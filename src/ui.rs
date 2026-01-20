use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, View};
use crate::ui_branches::render_branches;
use crate::ui_files::render_files;
use crate::ui_history::render_history;

pub fn render_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    // Render header
    render_header(f, app, chunks[0]);

    // Render main content based on current view
    match app.current_view {
        View::Files => render_files(f, app, chunks[1]),
        View::History => render_history(f, app, chunks[1]),
        View::Branches => render_branches(f, app, chunks[1]),
    }

    // Render footer
    render_footer(f, app, chunks[2]);

    // Render commit dialog if active
    if app.show_commit_dialog {
        render_commit_dialog(f, app);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let title = vec![
        Span::styled("GitUI", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(
            format!("Branch: {}", app.branches_state.current_branch),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled("Views: ", Style::default().fg(Color::White)),
        Span::styled("[1]", get_view_style(app, View::Files)),
        Span::raw(" Files "),
        Span::styled("[2]", get_view_style(app, View::History)),
        Span::raw(" History "),
        Span::styled("[3]", get_view_style(app, View::Branches)),
        Span::raw(" Branches"),
    ];

    let header = Paragraph::new(Line::from(title))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(header, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.current_view {
        View::Files => {
            "↑/↓:Navigate | s:Stage | a:Stage All | c:Commit | p:Pull | P:Push | S:Sync | r:Refresh | q:Quit"
        }
        View::History => {
            "↑/↓:Navigate | r:Refresh | q:Quit"
        }
        View::Branches => {
            "↑/↓:Navigate | Enter:Checkout | r:Refresh | q:Quit"
        }
    };

    let mut footer_lines = vec![Line::from(Span::styled(
        help_text,
        Style::default().fg(Color::Gray),
    ))];

    if let Some(status) = &app.status_message {
        footer_lines.insert(
            0,
            Line::from(Span::styled(
                status,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
        );
    }

    let footer = Paragraph::new(footer_lines)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(footer, area);
}

fn render_commit_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());

    let block = Block::default()
        .title("Commit Message (Enter to commit, Esc to cancel)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let text = Paragraph::new(app.commit_message.as_str())
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(Clear, area);
    f.render_widget(text, area);
}

fn get_view_style(app: &App, view: View) -> Style {
    if app.current_view == view {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
