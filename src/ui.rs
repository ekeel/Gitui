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
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
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

    // Render branch creation dialog if active
    if app.show_branch_dialog {
        render_branch_dialog(f, app);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let title = vec![
        Span::styled(
            "GitUI",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
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
            "↑/↓:Navigate | PgUp/PgDn:Scroll | s:Stage | a:Stage All | c:Commit | p:Pull | P:Push | S:Sync | r:Refresh | q:Quit"
        }
        View::History => {
            "↑/↓:Navigate | r:Refresh | q:Quit"
        }
        View::Branches => {
            "↑/↓:Navigate | n:New Branch | Enter:Checkout | r:Refresh | q:Quit"
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
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
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

fn render_branch_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, f.area());

    f.render_widget(Clear, area);

    if app.branch_creation.selecting_base {
        // Show base branch selection
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(area);

        let branches: Vec<ratatui::widgets::ListItem> = app
            .branches_state
            .branches
            .iter()
            .enumerate()
            .map(|(i, branch)| {
                let style = if i == app.branch_creation.base_branch_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if branch.is_current { "* " } else { "  " };
                let content = Line::from(vec![Span::raw(prefix), Span::raw(&branch.name)]);

                ratatui::widgets::ListItem::new(content).style(style)
            })
            .collect();

        let list = ratatui::widgets::List::new(branches).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Base Branch (Enter to confirm, Esc to cancel)")
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(list, chunks[0]);
    } else {
        // Show branch name input
        let base_branch = app
            .branches_state
            .branches
            .get(app.branch_creation.base_branch_selected)
            .map(|b| b.name.as_str())
            .unwrap_or(&app.branches_state.current_branch);

        let title = format!(
            "Create Branch from '{}' (Tab to change base, Enter to create, Esc to cancel)",
            base_branch
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let text = Paragraph::new(app.branch_creation.new_branch_name.as_str())
            .block(block)
            .style(Style::default().fg(Color::White));

        f.render_widget(text, area);
    }
}

fn get_view_style(app: &App, view: View) -> Style {
    if app.current_view == view {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
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
