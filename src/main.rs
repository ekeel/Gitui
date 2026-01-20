mod app;
mod git;
mod input;
mod ui;
mod ui_branches;
mod ui_files;
mod ui_history;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use app::App;
use git::GitRepo;
use input::{handle_key_event, handle_mouse_event};
use ui::render_ui;

fn main() -> Result<()> {
    // Get repository path from args or use current directory
    let repo_path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Open git repository
    let git_repo = GitRepo::open(&repo_path)?;
    // Test comment
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(repo_path);

    // Initialize data
    app.branches_state.current_branch = git_repo.get_current_branch()?;
    app.branches_state.branches = git_repo.get_branches()?;
    app.files_state.files = git_repo.get_status()?;
    if !app.files_state.files.is_empty() {
        if let Ok(diff) = git_repo.get_diff_for_file(&app.files_state.files[0].path) {
            app.files_state.current_diff = Some(diff);
        }
    }
    app.history_state.commits = git_repo.get_commits(100)?;

    // Setup panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        restore_terminal();
        original_hook(panic);
    }));

    // Main loop
    let result = run_app(&mut terminal, &mut app, &git_repo);

    // Restore terminal - always do this
    restore_terminal();

    // After restoring terminal, we can safely show errors
    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn restore_terminal() {
    // Disable raw mode
    let _ = disable_raw_mode();

    // Restore terminal state
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    git_repo: &GitRepo,
) -> Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key_event(app, key, git_repo)?;
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(app, mouse)?;
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
