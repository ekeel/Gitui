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

            let mut spans = vec![];

            // Add graph visualization
            if let Some(ref graph_info) = commit.graph_info {
                if !graph_info.graph_line.trim().is_empty() {
                    spans.push(Span::styled(
                        format!("{}│ ", graph_info.graph_line),
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    // Fallback if graph is empty
                    spans.push(Span::styled(
                        "● │ ",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            } else {
                // No graph info, show basic marker
                spans.push(Span::styled(
                    "● │ ",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            // Add commit info
            spans.push(Span::styled(
                format!("{} ", commit.id),
                Style::default().fg(Color::Yellow),
            ));

            // Add branch labels
            if !commit.branches.is_empty() {
                for branch_name in &commit.branches {
                    spans.push(Span::styled(
                        format!("({}) ", branch_name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }

            spans.push(Span::raw(format!("{} ", commit.date)));
            spans.push(Span::styled(
                commit.author.clone(),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::raw(format!(" - {}", commit.message)));

            let content = vec![Line::from(spans)];

            ListItem::new(content).style(style)
        })
        .collect();

    let commits_list = List::new(commits).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Commit History")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(commits_list, area);
}
