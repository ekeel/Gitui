use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

pub fn render_branches(f: &mut Frame, app: &App, area: Rect) {
    let branches: Vec<ListItem> = app
        .branches_state
        .branches
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            let style = if i == app.branches_state.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if branch.is_current { "* " } else { "  " };
            let branch_style = if branch.is_current {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let content = Line::from(vec![
                Span::styled(prefix, branch_style),
                Span::styled(&branch.name, branch_style),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!("Branches (Current: {})", app.branches_state.current_branch);
    let branches_list = List::new(branches).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(branches_list, area);
}
