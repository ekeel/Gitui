use crossterm::event::{KeyCode, KeyEvent};
use anyhow::Result;

use crate::app::{App, View};
use crate::git::GitRepo;

pub fn handle_key_event(app: &mut App, key: KeyEvent, git_repo: &GitRepo) -> Result<()> {
    // Global key bindings
    match key.code {
        KeyCode::Char('q') if !app.show_commit_dialog => {
            app.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('1') if !app.show_commit_dialog => {
            app.switch_view(View::Files);
            refresh_files(app, git_repo)?;
            return Ok(());
        }
        KeyCode::Char('2') if !app.show_commit_dialog => {
            app.switch_view(View::History);
            refresh_history(app, git_repo)?;
            return Ok(());
        }
        KeyCode::Char('3') if !app.show_commit_dialog => {
            app.switch_view(View::Branches);
            refresh_branches(app, git_repo)?;
            return Ok(());
        }
        KeyCode::Char('r') if !app.show_commit_dialog => {
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

    // Navigation
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_item();
            if app.current_view == View::Files && !app.files_state.files.is_empty() {
                update_file_diff(app, git_repo)?;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
            if app.current_view == View::Files && !app.files_state.files.is_empty() {
                update_file_diff(app, git_repo)?;
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
            // Push
            app.set_status("Pushing...".to_string());
            match git_repo.push() {
                Ok(_) => app.set_status("Pushed successfully".to_string()),
                Err(e) => app.set_status(format!("Push failed: {}", e)),
            }
        }
        KeyCode::Char('p') => {
            // Pull
            app.set_status("Pulling...".to_string());
            match git_repo.pull() {
                Ok(_) => {
                    app.set_status("Pulled successfully".to_string());
                    refresh_current_view(app, git_repo)?;
                }
                Err(e) => app.set_status(format!("Pull failed: {}", e)),
            }
        }
        KeyCode::Char('S') => {
            // Sync (pull + push)
            app.set_status("Syncing...".to_string());
            match git_repo.sync() {
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
        app.files_state.selected = app.files_state.selected.min(app.files_state.files.len() - 1);
        update_file_diff(app, git_repo)?;
    } else {
        app.files_state.current_diff = None;
    }
    Ok(())
}

fn refresh_history(app: &mut App, git_repo: &GitRepo) -> Result<()> {
    app.history_state.commits = git_repo.get_commits(100)?;
    if !app.history_state.commits.is_empty() {
        app.history_state.selected = app.history_state.selected.min(app.history_state.commits.len() - 1);
    }
    Ok(())
}

fn refresh_branches(app: &mut App, git_repo: &GitRepo) -> Result<()> {
    app.branches_state.branches = git_repo.get_branches()?;
    app.branches_state.current_branch = git_repo.get_current_branch()?;
    if !app.branches_state.branches.is_empty() {
        app.branches_state.selected = app.branches_state.selected.min(app.branches_state.branches.len() - 1);
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
