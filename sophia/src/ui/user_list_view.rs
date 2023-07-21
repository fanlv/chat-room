use std::io::Write;

use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Alignment, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::{config, view_model};
use crate::ui::theme::Theme;

pub fn draw(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    conf: &config::Config,
    state: &view_model::UserViewModel,
    chunk: Rect,
    theme: &Theme) {
    let mut users: Vec<Spans> = Vec::new();

    for user in state.users.iter() {
        let idx = user.login_time as usize;
        let mut color = theme.message_colors[idx % theme.message_colors.len()];
        let mut user_name = user.user_name.to_string();
        if user_name == conf.user_name {
            user_name = format!("{}(me)", user_name);
            color = theme.my_user_color;
        }

        users.push(
            Spans::from(vec![
                Span::styled(user_name, Style::default().fg(color)),
                Span::styled(format!("-{}", &user.address), Style::default().fg(theme.address_color)),
            ])
        )
    }


    let user_list_panel = Paragraph::new(users)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Online User List", Style::default().add_modifier(Modifier::BOLD))),
        )
        .style(Style::default().fg(theme.panel_border_color))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(user_list_panel, chunk);
}
