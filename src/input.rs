use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::app::{App, View};
use crate::git::GitRepo;

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> Result<()> {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            if app.current_view == View::Files {
                app.scroll_diff_down();
            }
        }
        MouseEventKind::ScrollUp => {
            if app.current_view == View::Files {
                app.scroll_diff_up();
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_key_event(app: &mut App, key: KeyEvent, git_repo: &GitRepo) -> Result<()> {
    // Global key bindings (only when no dialog is open)
    match key.code {
        KeyCode::Char('q')
            if !app.show_commit_dialog && !app.show_branch_dialog && !app.show_delete_confirm =>
        {
            app.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('1')
            if !app.show_commit_dialog && !app.show_branch_dialog && !app.show_delete_confirm =>
        {
            app.switch_view(View::Files);
            refresh_files(app, git_repo)?;
            return Ok(());
        }
        KeyCode::Char('2')
            if !app.show_commit_dialog && !app.show_branch_dialog && !app.show_delete_confirm =>
        {
            app.switch_view(View::History);
            refresh_history(app, git_repo)?;
            return Ok(());
        }
        KeyCode::Char('3')
            if !app.show_commit_dialog && !app.show_branch_dialog && !app.show_delete_confirm =>
        {
            app.switch_view(View::Branches);
            refresh_branches(app, git_repo)?;
            return Ok(());
        }
        KeyCode::Char('r')
            if !app.show_commit_dialog && !app.show_branch_dialog && !app.show_delete_confirm =>
        {
            refresh_current_view(app, git_repo)?;
            app.set_status("Refreshed".to_string());
            return Ok(());
        }
        _ => {}
    }

    // Commit dialog handling
    if app.show_commit_dialog {
        match key.code {
            KeyCode::Esc => {
                app.show_commit_dialog = false;
                app.commit_message.clear();
            }
            KeyCode::Enter => {
                if !app.commit_message.trim().is_empty() {
                    match git_repo.commit(&app.commit_message) {
                        Ok(_) => {
                            app.set_status("Committed successfully".to_string());
                            app.commit_message.clear();
                            app.show_commit_dialog = false;
                            refresh_files(app, git_repo)?;
                        }
                        Err(e) => {
                            app.set_status(format!("Commit failed: {}", e));
                        }
                    }
                }
            }
            KeyCode::Char(c) => {
                app.commit_message.push(c);
            }
            KeyCode::Backspace => {
                app.commit_message.pop();
            }
            _ => {}
        }
        return Ok(());
    }

    // Branch creation dialog handling
    if app.show_branch_dialog {
        if app.branch_creation.selecting_base {
            // Selecting base branch
            match key.code {
                KeyCode::Esc => {
                    app.show_branch_dialog = false;
                    app.branch_creation.new_branch_name.clear();
                    app.branch_creation.selecting_base = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.branch_creation.base_branch_selected > 0 {
                        app.branch_creation.base_branch_selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !app.branches_state.branches.is_empty() {
                        app.branch_creation.base_branch_selected =
                            (app.branch_creation.base_branch_selected + 1)
                                .min(app.branches_state.branches.len() - 1);
                    }
                }
                KeyCode::Enter => {
                    // Confirm base branch selection, go back to name entry
                    app.branch_creation.selecting_base = false;
                }
                _ => {}
            }
        } else {
            // Entering branch name
            match key.code {
                KeyCode::Esc => {
                    app.show_branch_dialog = false;
                    app.branch_creation.new_branch_name.clear();
                }
                KeyCode::Enter => {
                    if !app.branch_creation.new_branch_name.trim().is_empty() {
                        let base_branch = &app
                            .branches_state
                            .branches
                            .get(app.branch_creation.base_branch_selected)
                            .map(|b| b.name.clone())
                            .unwrap_or_else(|| app.branches_state.current_branch.clone());

                        let branch_name = app.branch_creation.new_branch_name.clone();
                        match git_repo.create_branch(&branch_name, base_branch) {
                            Ok(_) => {
                                // Push the new branch to remote
                                app.set_status(format!("Pushing branch to remote..."));
                                let _ = disable_raw_mode();
                                let push_result = git_repo.push_branch(&branch_name);
                                let _ = enable_raw_mode();

                                match push_result {
                                    Ok(_) => {
                                        app.set_status(format!(
                                            "Created and pushed branch: {}",
                                            branch_name
                                        ));
                                    }
                                    Err(e) => {
                                        app.set_status(format!(
                                            "Created branch locally but failed to push: {}",
                                            e
                                        ));
                                    }
                                }
                                app.branch_creation.new_branch_name.clear();
                                app.show_branch_dialog = false;
                                refresh_branches(app, git_repo)?;
                            }
                            Err(e) => {
                                app.set_status(format!("Failed to create branch: {}", e));
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    app.branch_creation.new_branch_name.push(c);
                }
                KeyCode::Backspace => {
                    app.branch_creation.new_branch_name.pop();
                }
                KeyCode::Tab => {
                    // Switch to base branch selection
                    app.branch_creation.selecting_base = true;
                }
                _ => {}
            }
        }
        return Ok(());
    }

    // Delete confirmation dialog handling
    if app.show_delete_confirm {
        match key.code {
            KeyCode::Esc => {
                app.show_delete_confirm = false;
                app.delete_confirmation.clear();
                app.branch_to_delete = None;
            }
            KeyCode::Enter => {
                let confirmation = app.delete_confirmation.trim().to_lowercase();
                if confirmation == "y" || confirmation == "yes" {
                    if let Some(branch_name) = &app.branch_to_delete {
                        match git_repo.delete_branch(branch_name) {
                            Ok(_) => {
                                app.set_status(format!("Deleted branch: {}", branch_name));
                                app.show_delete_confirm = false;
                                app.delete_confirmation.clear();
                                app.branch_to_delete = None;
                                refresh_branches(app, git_repo)?;
                            }
                            Err(e) => {
                                app.set_status(format!("Failed to delete branch: {}", e));
                            }
                        }
                    }
                } else {
                    app.set_status("Delete cancelled".to_string());
                    app.show_delete_confirm = false;
                    app.delete_confirmation.clear();
                    app.branch_to_delete = None;
                }
            }
            KeyCode::Char(c) => {
                app.delete_confirmation.push(c);
            }
            KeyCode::Backspace => {
                app.delete_confirmation.pop();
            }
            _ => {}
        }
        return Ok(());
    }

    // Navigation
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_item();
            if app.current_view == View::Files && !app.files_state.files.is_empty() {
                app.reset_diff_scroll();
                update_file_diff(app, git_repo)?;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
            if app.current_view == View::Files && !app.files_state.files.is_empty() {
                app.reset_diff_scroll();
                update_file_diff(app, git_repo)?;
            }
        }
        KeyCode::PageUp => {
            if app.current_view == View::Files {
                for _ in 0..10 {
                    app.scroll_diff_up();
                }
            }
        }
        KeyCode::PageDown => {
            if app.current_view == View::Files {
                for _ in 0..10 {
                    app.scroll_diff_down();
                }
            }
        }
        _ => {}
    }

    // View-specific key bindings
    match app.current_view {
        View::Files => handle_files_keys(app, key, git_repo)?,
        View::Branches => handle_branches_keys(app, key, git_repo)?,
        View::History => {}
    }

    Ok(())
}

fn handle_files_keys(app: &mut App, key: KeyEvent, git_repo: &GitRepo) -> Result<()> {
    match key.code {
        KeyCode::Char('s') => {
            // Stage selected file
            if let Some(file) = app.files_state.files.get(app.files_state.selected) {
                match git_repo.stage_file(&file.path) {
                    Ok(_) => {
                        app.set_status(format!("Staged: {}", file.path));
                        refresh_files(app, git_repo)?;
                    }
                    Err(e) => {
                        app.set_status(format!("Failed to stage: {}", e));
                    }
                }
            }
        }
        KeyCode::Char('a') => {
            // Stage all files
            match git_repo.stage_all() {
                Ok(_) => {
                    app.set_status("Staged all files".to_string());
                    refresh_files(app, git_repo)?;
                }
                Err(e) => {
                    app.set_status(format!("Failed to stage all: {}", e));
                }
            }
        }
        KeyCode::Char('c') => {
            // Show commit dialog
            app.show_commit_dialog = true;
            app.commit_message.clear();
        }
        KeyCode::Char('P') => {
            // Push - temporarily restore terminal for credential prompts
            app.set_status("Pushing...".to_string());
            let _ = disable_raw_mode();
            let result = git_repo.push();
            let _ = enable_raw_mode();

            match result {
                Ok(_) => app.set_status("Pushed successfully".to_string()),
                Err(e) => app.set_status(format!("Push failed: {}", e)),
            }
        }
        KeyCode::Char('p') => {
            // Pull - temporarily restore terminal for credential prompts
            app.set_status("Pulling...".to_string());
            let _ = disable_raw_mode();
            let result = git_repo.pull();
            let _ = enable_raw_mode();

            match result {
                Ok(_) => {
                    app.set_status("Pulled successfully".to_string());
                    refresh_current_view(app, git_repo)?;
                }
                Err(e) => app.set_status(format!("Pull failed: {}", e)),
            }
        }
        KeyCode::Char('S') => {
            // Sync (pull + push) - temporarily restore terminal
            app.set_status("Syncing...".to_string());
            let _ = disable_raw_mode();
            let result = git_repo.sync();
            let _ = enable_raw_mode();

            match result {
                Ok(_) => {
                    app.set_status("Synced successfully".to_string());
                    refresh_current_view(app, git_repo)?;
                }
                Err(e) => app.set_status(format!("Sync failed: {}", e)),
            }
        }
        KeyCode::Enter => {
            // Update diff for selected file
            update_file_diff(app, git_repo)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_branches_keys(app: &mut App, key: KeyEvent, git_repo: &GitRepo) -> Result<()> {
    match key.code {
        KeyCode::Char('n') => {
            // Open branch creation dialog
            app.show_branch_dialog = true;
            app.branch_creation.new_branch_name.clear();
            app.branch_creation.selecting_base = false;
            // Set default base to current branch index
            if let Some(pos) = app
                .branches_state
                .branches
                .iter()
                .position(|b| b.is_current)
            {
                app.branch_creation.base_branch_selected = pos;
            }
        }
        KeyCode::Char('d') => {
            // Open delete confirmation dialog
            if let Some(branch) = app.branches_state.branches.get(app.branches_state.selected) {
                if !branch.is_current {
                    app.show_delete_confirm = true;
                    app.branch_to_delete = Some(branch.name.clone());
                    app.delete_confirmation.clear();
                } else {
                    app.set_status("Cannot delete the current branch".to_string());
                }
            }
        }
        KeyCode::Enter | KeyCode::Char('o') => {
            // Checkout selected branch
            if let Some(branch) = app.branches_state.branches.get(app.branches_state.selected) {
                if !branch.is_current {
                    match git_repo.checkout_branch(&branch.name) {
                        Ok(_) => {
                            app.set_status(format!("Checked out: {}", branch.name));
                            refresh_branches(app, git_repo)?;
                            refresh_files(app, git_repo)?;
                        }
                        Err(e) => {
                            app.set_status(format!("Checkout failed: {}", e));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn update_file_diff(app: &mut App, git_repo: &GitRepo) -> Result<()> {
    if let Some(file) = app.files_state.files.get(app.files_state.selected) {
        match git_repo.get_diff_for_file(&file.path) {
            Ok(diff) => {
                app.files_state.current_diff = Some(diff);
            }
            Err(e) => {
                app.files_state.current_diff = Some(format!("Error getting diff: {}", e));
            }
        }
    }
    Ok(())
}

fn refresh_files(app: &mut App, git_repo: &GitRepo) -> Result<()> {
    app.files_state.files = git_repo.get_status()?;
    if !app.files_state.files.is_empty() {
        app.files_state.selected = app
            .files_state
            .selected
            .min(app.files_state.files.len() - 1);
        update_file_diff(app, git_repo)?;
    } else {
        app.files_state.current_diff = None;
    }
    Ok(())
}

fn refresh_history(app: &mut App, git_repo: &GitRepo) -> Result<()> {
    app.history_state.commits = git_repo.get_commits(100)?;
    if !app.history_state.commits.is_empty() {
        app.history_state.selected = app
            .history_state
            .selected
            .min(app.history_state.commits.len() - 1);
    }
    Ok(())
}

fn refresh_branches(app: &mut App, git_repo: &GitRepo) -> Result<()> {
    app.branches_state.branches = git_repo.get_branches()?;
    app.branches_state.current_branch = git_repo.get_current_branch()?;
    if !app.branches_state.branches.is_empty() {
        app.branches_state.selected = app
            .branches_state
            .selected
            .min(app.branches_state.branches.len() - 1);
    }
    Ok(())
}

fn refresh_current_view(app: &mut App, git_repo: &GitRepo) -> Result<()> {
    match app.current_view {
        View::Files => refresh_files(app, git_repo)?,
        View::History => refresh_history(app, git_repo)?,
        View::Branches => refresh_branches(app, git_repo)?,
    }
    Ok(())
}
