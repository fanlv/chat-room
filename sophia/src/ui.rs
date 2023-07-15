use std::io::Write;

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use crossterm::ExecutableCommand;
use crossterm::terminal::{self};
use log::{info, Level};
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::Terminal;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use sophia_core::errno_new;
use sophia_core::errors::Result;

use crate::controller::{AppViewModel, MessageViewModel, SomeUser};

pub struct UIViews<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
    rect: Rect,
}


impl<W: Write> UIViews<W> {
    pub fn new(mut out: W) -> Result<UIViews<W>> {
        terminal::enable_raw_mode()
            .map_err(|e| errno_new!("terminal::enable_raw_mode failed =  {}",e))?;

        out.execute(terminal::EnterAlternateScreen)
            .map_err(|e| errno_new!("out.execute failed =  {}",e))?;

        Ok(UIViews {
            terminal: Terminal::new(CrosstermBackend::new(out))?,
            rect: Rect::default(),
        })
    }

    pub async fn render(&mut self, mut state: AppViewModel) -> Result<()> {
        let (_, message_chunks) = layout(self.rect);

        reset_scroll_messages_view_pos(&mut state, message_chunks[0]).await;
        self.terminal.draw(|frame| {
            self.rect = frame.size();
            draw(state, frame)
        })?;

        Ok(())
    }
}

impl<W: Write> Drop for UIViews<W> {
    fn drop(&mut self) {
        self.terminal.backend_mut()
            .execute(terminal::LeaveAlternateScreen)
            .expect("Could not execute to stdout");
        terminal::disable_raw_mode().expect("Terminal doesn't support to disable raw mode");
        if std::thread::panicking() {
            eprintln!(
                "to log the error you can redirect std error to a file, example: sphia 2> sphia_log",
            );
        }
    }
}


pub struct Theme {
    pub message_colors: Vec<Color>,
    pub my_user_color: Color,
    pub date_color: Color,
    pub address_color: Color,
    pub system_info_color: (Color, Color),
    pub panel_border_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl Theme {
    fn dark_theme() -> Self {
        Self {
            message_colors: vec![
                Color::Yellow,
                Color::Magenta,
                Color::Rgb(255, 0, 255),
                Color::Rgb(0, 255, 255),
                Color::Rgb(255, 165, 0),
                Color::Rgb(153, 50, 205),
                Color::Rgb(153, 50, 205),
                Color::Rgb(255, 215, 255)],
            my_user_color: Color::Green,
            date_color: Color::DarkGray,
            address_color: Color::DarkGray,
            system_info_color: (Color::LightRed, Color::LightCyan),
            panel_border_color: Color::White,
        }
    }

    fn light_theme() -> Self {
        Self {
            message_colors: vec![Color::Black],
            my_user_color: Color::Green,
            date_color: Color::Black,
            address_color: Color::Black,
            system_info_color: (Color::LightRed, Color::LightCyan),
            panel_border_color: Color::Black,
        }
    }
}


pub fn layout(chunk: Rect) -> (Vec<Rect>, Vec<Rect>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(33)].as_ref())
        .split(chunk);

    let message_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(5), Constraint::Length(10)].as_ref())
        .split(chunks[0]);

    (chunks, message_chunks)
}

fn draw(state: AppViewModel, frame: &mut Frame<CrosstermBackend<impl Write>>) {
    let (chunks, message_chunks) = layout(frame.size());

    // info!("[draw] message len = {}", state.message_list.len());

    let mut theme = Theme::default();
    if state.conf.theme != "dark" {
        theme = Theme::light_theme();
    }

    draw_message_list_panel(frame, &state, message_chunks[0], &theme);
    draw_input_panel(frame, &state, message_chunks[1], &theme);
    draw_log_panel(frame, &state, message_chunks[2], &theme);
    draw_user_list_panel(frame, &state, chunks[1], &theme);
}


fn draw_log_panel(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    state: &AppViewModel,
    chunk: Rect,
    theme: &Theme,
) {
    let logs = state.logs
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


fn draw_user_list_panel(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    state: &AppViewModel,
    chunk: Rect,
    theme: &Theme) {
    let mut users: Vec<Spans> = Vec::new();

    for user in state.user_list.iter() {
        let idx = user.login_time as usize;
        let mut color = theme.message_colors[idx % theme.message_colors.len()];
        let mut user_name = user.user_name.to_string();
        if user_name == state.conf.user_name {
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


fn draw_message_list_panel(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    state: &AppViewModel,
    chunk: Rect,
    theme: &Theme,
) {
    if !state.message_list.is_empty() {
        info!("last message content = {} , {}" ,state.message_list.len(), state.message_list.last().unwrap().content);
    }

    let scroll_messages_view_pos = state.reset_pos;
    let mut msg_list: Vec<Spans> = Vec::new();


    for msg in state.message_list.iter() {
        let date = get_time_string_with_custom(msg.time, "%H:%M:%S");

        match &msg.user {
            SomeUser::User(u) => {
                let idx = u.login_time as usize;
                let mut color = theme.message_colors[idx % theme.message_colors.len()];

                let remote = format!(" ({}) ", u.address);
                let mut name = u.user_name.to_string();

                if name == state.conf.user_name {
                    name = format!("{}(me)", state.conf.user_name);
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
                .title(Span::styled(format!("Chat Room : {}", state.conf.chat_id),
                                    Style::default().add_modifier(Modifier::BOLD))),
        )
        .style(Style::default().fg(theme.panel_border_color))
        .alignment(Alignment::Left)
        .scroll((scroll_messages_view_pos as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(msg_list_panel, chunk);
}

fn draw_input_panel(
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    state: &AppViewModel,
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

pub fn ui_input_cursor(state: &AppViewModel, width: usize) -> (u16, u16) {
    let mut position = (0, 0);

    for current_char in state.input.iter().take(state.input_cursor) {
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

pub async fn reset_scroll_messages_view_pos(state: &mut AppViewModel, rect: Rect) {
    if rect.height == 0 || rect.width == 0 {
        return;
    }
    let mut scroll_messages_view_pos = calculate_message_scroll_pos(state, &rect).await;
    if scroll_messages_view_pos > 0 {
        let mut current_pos = state.scroll_messages_view_pos.write().await;
        *current_pos = scroll_messages_view_pos;
    } else {
        scroll_messages_view_pos = state.scroll_messages_view_pos.read().await.clone();
    }

    state.reset_pos = scroll_messages_view_pos;
}

pub async fn calculate_message_scroll_pos(state: &AppViewModel, chunk: &Rect) -> usize {
    if state.message_list.is_empty() {
        return 0;
    }


    let curr_pos;
    {
        curr_pos = state.scroll_messages_view_pos.read().await.clone();
    }

    let mut new_pos: usize = 0;
    let height = chunk.height as usize - 4;
    let width = chunk.width as usize - 2;

    let lines: usize = calculate_message_lines(width, &state.message_list);
    if lines > height && curr_pos < lines - height {
        new_pos = lines - height;
    } else if lines > 2 && curr_pos >= lines - 2 {
        return lines - 2;
    }

    let mut need_update_pos = false;
    {
        let mut auto_set_scroll_messages_view_pos = state.auto_set_scroll_messages_view_pos.write().await;
        if *auto_set_scroll_messages_view_pos > 0 && auto_set_scroll_messages_view_pos.le(&state.message_list.len()) {
            *auto_set_scroll_messages_view_pos = 0;
            need_update_pos = true;
        }
    }

    if need_update_pos {
        info!("curr_pos = {} , new_pos =  {} , lines = {}  , height = {}",curr_pos ,new_pos,lines  ,height);
        new_pos
    } else {
        0
    }
}

fn calculate_message_lines(width: usize, message_list: &Vec<MessageViewModel>) -> usize {
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
