use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render_files(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left side: file list
    let files: Vec<ListItem> = app
        .files_state
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == app.files_state.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match file.status.trim() {
                "A" => Color::Green,
                "M" | " M" => Color::Yellow,
                "D" | " D" => Color::Red,
                "??" => Color::Blue,
                _ => Color::White,
            };

            let content = Line::from(vec![
                Span::styled(format!("{} ", file.status), Style::default().fg(status_color)),
                Span::raw(&file.path),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let files_list = List::new(files).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Files")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(files_list, chunks[0]);

    // Right side: diff view
    let diff_text = app
        .files_state
        .current_diff
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Select a file to view diff");

    let diff_lines: Vec<Line> = diff_text
        .lines()
        .map(|line| {
            let style = if line.starts_with('+') {
                Style::default().fg(Color::Green)
            } else if line.starts_with('-') {
                Style::default().fg(Color::Red)
            } else if line.starts_with("@@") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(Span::styled(line, style))
        })
        .collect();

    let diff_paragraph = Paragraph::new(diff_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Diff")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(diff_paragraph, chunks[1]);
}
