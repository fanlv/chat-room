use std::io::Write;
use std::sync::Arc;

use crossterm::ExecutableCommand;
use crossterm::terminal::{self};
use tokio::sync::RwLock;
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::Terminal;

use sophia_core::errno_new;
use sophia_core::errors::Result;

use crate::ui::{input_view, log_view, message_view, user_list_view};
use crate::ui::theme::Theme;
use crate::view_model::AppViewModel;

pub struct AppView<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
    rect: Rect,
}


impl<W: Write> AppView<W> {
    pub fn new(mut out: W) -> Result<AppView<W>> {
        terminal::enable_raw_mode()
            .map_err(|e| errno_new!("terminal::enable_raw_mode failed =  {}",e))?;

        out.execute(terminal::EnterAlternateScreen)
            .map_err(|e| errno_new!("out.execute failed =  {}",e))?;

        Ok(AppView {
            terminal: Terminal::new(CrosstermBackend::new(out))?,
            rect: Rect::default(),
        })
    }

    pub async fn render(&mut self, state: Arc<RwLock<AppViewModel>>) -> Result<()> {
        let (_, message_chunks) = layout(self.rect);
        message_view::adjust_scroll_pos(state.clone(), message_chunks[0]).await;

        let state = state.read().await.clone();
        self.terminal.draw(|frame| {
            self.rect = frame.size();
            draw(state, frame)
        })?;

        Ok(())
    }
}

impl<W: Write> Drop for AppView<W> {
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


fn draw(state: AppViewModel, frame: &mut Frame<CrosstermBackend<impl Write>>) {
    let (chunks, message_chunks) = layout(frame.size());

    let mut theme = Theme::default();
    if state.conf.theme != "dark" {
        theme = Theme::light_theme();
    }

    message_view::draw(frame, &state.conf, &state.msg_vm, message_chunks[0], &theme);
    input_view::draw(frame, &state.input_vm, message_chunks[1], &theme);
    log_view::draw(frame, &state.log_vm, message_chunks[2], &theme);
    user_list_view::draw(frame, &state.conf, &state.user_vm, chunks[1], &theme);
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

