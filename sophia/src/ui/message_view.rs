use std::io::Write;
use std::sync::Arc;

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use log::info;
use tokio::sync::RwLock;
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Alignment, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::{config, view_model};
use crate::ui::theme::Theme;
use crate::view_model::{AppViewModel, Message, SomeUser};

pub fn draw(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    conf: &config::Config,
    state: &view_model::ChatMessageViewModel,
    chunk: Rect,
    theme: &Theme,
) {
    if !state.messages.is_empty() {
        info!("last message content = {} , {}" ,state.messages.len(), state.messages.last().unwrap().content);
    }

    let scroll_messages_view_pos = state.scroll_pos;
    let mut msg_list: Vec<Spans> = Vec::new();


    for msg in state.messages.iter() {
        let date = get_time_string_with_custom(msg.time, "%H:%M:%S");

        match &msg.user {
            SomeUser::User(u) => {
                let idx = u.login_time as usize;
                let mut color = theme.message_colors[idx % theme.message_colors.len()];

                let remote = format!(" ({}) ", u.address);
                let mut name = u.user_name.to_string();

                if name == conf.user_name {
                    name = format!("{}(me)", conf.user_name);
                    color = theme.my_user_color
                }

                msg_list.push(
                    Spans::from(vec![
                        Span::styled(date, Style::default().fg(theme.date_color)),
                        Span::styled(remote, Style::default().fg(theme.address_color)),
                        Span::styled(format!("{} :", name), Style::default().fg(color)),
                    ]));

                msg_list.push(
                    Spans::from(vec![
                        Span::raw(format!("  {}", &msg.content)),
                    ]));
            }
            SomeUser::System => {
                msg_list.push(
                    Spans::from(vec![
                        Span::styled(date, Style::default().fg(theme.date_color)),
                        Span::styled(format!("  {}", &msg.content), Style::default().fg(theme.system_info_color.0)),
                    ]));
            }
        };
    }


    let msg_list_panel = Paragraph::new(msg_list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(format!("Chat Room : {}", conf.chat_id),
                                    Style::default().add_modifier(Modifier::BOLD))),
        )
        .style(Style::default().fg(theme.panel_border_color))
        .alignment(Alignment::Left)
        .scroll((scroll_messages_view_pos as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(msg_list_panel, chunk);
}

fn get_time_string_with_custom(timestamp: i64, str: &str) -> String {
    // 将 i64 时间戳转换为 NaiveDateTime
    let naive_datetime = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    // 将 NaiveDateTime 转换为 DateTime<Utc>
    let datetime_utc = DateTime::<Utc>::from_utc(naive_datetime, Utc);
    // 使用 with_timezone 方法将 DateTime<Utc> 转换为 DateTime<Local>
    let datetime_local: DateTime<Local> = datetime_utc.with_timezone(&Local);
    let date = datetime_local.format(str).to_string();

    date
}


pub async fn adjust_scroll_pos(state: Arc<RwLock<AppViewModel>>, rect: Rect) {
    if rect.height == 0 || rect.width == 0 {
        return;
    }

    let mut state = state.write().await;
    let scroll_messages_view_pos = calculate_scroll_pos(&mut state, &rect);
    if scroll_messages_view_pos > 0 {
        state.msg_vm.scroll_pos = scroll_messages_view_pos
    }
}

pub fn calculate_scroll_pos(state: &mut AppViewModel, chunk: &Rect) -> usize {
    if state.msg_vm.messages.is_empty() {
        return 0;
    }


    let curr_pos = state.msg_vm.scroll_pos;
    let mut new_pos: usize = 0;
    let height = chunk.height as usize - 4;
    let width = chunk.width as usize - 2;
    let lines: usize = calculate_message_lines(width, &state.msg_vm.messages);

    if lines > height && curr_pos < lines - height {
        new_pos = lines - height;
    } else if lines > 2 && curr_pos >= lines - 2 {
        return lines - 2;
    }

    let mut need_update_pos = false;
    if state.msg_vm.scroll_to_pos > 0
        && state.msg_vm.scroll_to_pos.le(&state.msg_vm.messages.len()) {
        state.msg_vm.scroll_to_pos = 0;
        need_update_pos = true;
    }

    if need_update_pos {
        new_pos
    } else {
        0
    }
}

fn calculate_message_lines(width: usize, message_list: &Vec<Message>) -> usize {
    let mut lines: usize = 0;
    for message in message_list {
        if let SomeUser::User(_) = message.user.clone() {
            let content = format!("  {}", &message.content);
            let mut content_with: usize = 0;
            for c in content.chars() {
                let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
                content_with += char_width;
            }

            let result = content_with as f64 / width as f64;
            let ceil_result = result.ceil();
            let result_usize = ceil_result as usize;

            lines += result_usize + 1
        } else {
            lines += 1;
        }
    }

    lines
}