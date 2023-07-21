use std::io::Write;

use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Alignment, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::ui::theme::Theme;
use crate::view_model;

pub fn draw(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    state: &view_model::InputViewModel,
    chunk: Rect,
    theme: &Theme,
) {
    let inner_width = (chunk.width - 2) as usize;

    let input = state.input().iter().collect::<String>();
    let input = split_each(input, inner_width)
        .into_iter()
        .map(|line| Spans::from(vec![Span::raw(line)]))
        .collect::<Vec<_>>();

    let input_panel = Paragraph::new(input)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Input Your message", Style::default().add_modifier(Modifier::BOLD))),
        )
        .style(Style::default().fg(theme.panel_border_color))
        .alignment(Alignment::Left);

    frame.render_widget(input_panel, chunk);

    let input_cursor = ui_input_cursor(state, inner_width);
    frame.set_cursor(chunk.x + 1 + input_cursor.0, chunk.y + 1 + input_cursor.1)
}

pub fn split_each(input: String, width: usize) -> Vec<String> {
    let mut split = Vec::with_capacity(input.width() / width);
    let mut row = String::new();

    let mut index = 0;

    for current_char in input.chars() {
        if (index != 0 && index == width) || index + current_char.width().unwrap_or(0) > width {
            split.push(row.drain(..).collect());
            index = 0;
        }

        row.push(current_char);
        index += current_char.width().unwrap_or(0);
    }
    // leftover
    if !row.is_empty() {
        split.push(row.drain(..).collect());
    }
    split
}

pub fn ui_input_cursor(state: &view_model::InputViewModel, width: usize) -> (u16, u16) {
    let mut position = (0, 0);

    for current_char in state.text.iter().take(state.cursor) {
        let char_width = unicode_width::UnicodeWidthChar::width(*current_char).unwrap_or(0);

        position.0 += char_width;

        match position.0.cmp(&width) {
            std::cmp::Ordering::Equal => {
                position.0 = 0;
                position.1 += 1;
            }
            std::cmp::Ordering::Greater => {
                // Handle a char with width > 1 at the end of the row
                // width - (char_width - 1) accounts for the empty column(s) left behind
                position.0 -= width - (char_width - 1);
                position.1 += 1;
            }
            _ => (),
        }
    }

    (position.0 as u16, position.1 as u16)
}

