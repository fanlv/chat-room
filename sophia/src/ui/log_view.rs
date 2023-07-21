use std::io::Write;

use log::Level;
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::theme::Theme;
use crate::view_model;

pub fn draw(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    state: &view_model::LogViewModel,
    chunk: Rect,
    theme: &Theme,
) {
    let logs = state.contents
        .iter()
        .rev()
        .map(|log| {
            // Color::Blue, Color::Yellow, Color::Cyan, Color::Magenta
            let date = log.time.format("%m-%d %H:%M:%S ").to_string();
            let color = match log.level {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Blue,
                Level::Trace => Color::Cyan,
            };


            Spans::from(vec![
                Span::styled(date, Style::default().fg(theme.date_color)),
                Span::styled(log.level.to_string(), Style::default().fg(color)),
                Span::raw(format!(" {}", log.content.to_string())),
            ])
        })
        .collect::<Vec<_>>();

    let log_panel = Paragraph::new(logs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Log Console", Style::default().add_modifier(Modifier::BOLD))),
        )
        .style(Style::default().fg(theme.panel_border_color))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(log_panel, chunk);
}
