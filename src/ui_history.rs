use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

pub fn render_history(f: &mut Frame, app: &App, area: Rect) {
    let commits: Vec<ListItem> = app
        .history_state
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let style = if i == app.history_state.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let content = vec![Line::from(vec![
                Span::styled(format!("{} ", commit.id), Style::default().fg(Color::Yellow)),
                Span::raw(format!("{} ", commit.date)),
                Span::styled(commit.author.clone(), Style::default().fg(Color::Green)),
                Span::raw(format!(" - {}", commit.message)),
            ])];

            ListItem::new(content).style(style)
        })
        .collect();

    let commits_list = List::new(commits)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Commit History")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(commits_list, area);
}
