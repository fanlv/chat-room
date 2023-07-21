use std::io::Stdout;
use std::sync::Arc;

use crossterm::event;
use crossterm::event::KeyCode;
use log::Level;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::{Receiver, Sender};

use sophia_core::command;
use sophia_core::errors::Result;
use sophia_net::quic;

use crate::config;
use crate::controller::Caller;
use crate::controller::Controller;
use crate::ui::AppView;
use crate::view_model::AppViewModel;

pub async fn run(conf: config::Config) -> Result<()> {
    let (sender, receiver) = mpsc::channel::<Arc<RwLock<AppViewModel>>>(1);

    tokio::spawn(async move {
        let _ = io_run_loop(conf, sender).await;
    });

    let views = AppView::new(std::io::stdout())?;
    ui_run_loop(views, receiver).await;

    Ok(())
}


pub async fn ui_run_loop(mut views: AppView<Stdout>, mut receiver: Receiver<Arc<RwLock<AppViewModel>>>) {
    while let Some(state) = receiver.recv().await {
        let _ = views.render(state).await;
    }
}

pub async fn io_run_loop(conf: config::Config, sender: Sender<Arc<RwLock<AppViewModel>>>) -> Result<()> {
    let mut controller = Controller::new(sender, conf.clone());


    let mut quic_cli = quic::Client::new();
    let cli = quic_cli.with_cert_path(conf.cert_path)
        .with_application_level_protocols(conf.application_level_protocols)
        .with_server_addr(conf.server_addr)
        .with_server_name(conf.server_name);

    let login = command::Login {
        user_name: conf.user_name,
        chat_id: conf.chat_id,
        password: conf.password,
    };


    let controller1 = controller.clone();
    tokio::spawn(async move {
        keyboard_event(controller1).await;
    });


    controller.log(Level::Info, format!("TIPS: Press the 'ESC' key to EXIT")).await;
    loop {
        if controller.stop_accept_stream().await {
            break;
        }
        controller.log(Level::Info, format!("connecting...")).await;

        let res = cli.connect().await;
        if let Err(e) = res {
            controller.log(Level::Error, format!("connect failed, will retry ...  {}", e.to_string())).await;

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        }

        let conn = res.unwrap();
        controller.set_conn(conn.clone()).await;

        controller.log(Level::Info, format!("attempting to log in with username ({}) to the chat room ({}), please wait"
                                            , login.user_name, login.chat_id)).await;


        let res = controller.login(login.clone()).await;
        if let Err(e) = res {
            controller.log(Level::Error, format!("login failed , err = {}", e)).await;

            break;
        }

        let session_id = res.unwrap();
        controller.log(Level::Info, format!("login success.")).await;

        {
            let mut s = controller.session_id.write().await;
            *s = session_id;
        }


        // let res = accept_request(conn, controller).await;
        let res = conn.accept_request(controller.clone()).await;
        if let Err(e) = res {
            controller.log(Level::Error,
                           format!("accept_request_loop failed = {}", e.to_string())).await;
        }
    }

    Ok(())
}

async fn keyboard_event(controller: Controller) {
    loop {
        let result = event::read();
        if result.is_err() {
            controller.log(Level::Error, format!("event::read err : {}", result.err().unwrap())).await;

            continue;
        }

        let ev = result.unwrap();
        match ev {
            event::Event::Key(event::KeyEvent { code, .. }) => {
                if code == KeyCode::Esc {
                    exit_app(&controller).await;
                    return;
                }

                handle_key(code, &controller).await;
            }
            event::Event::Resize(_, _) => {
                // controller.log(Level::Info, format!("resize to {}x{}", w, h)).await;
                controller.refresh().await;
            }
            _ => ()
        }
    }
}

async fn exit_app(controller: &Controller) {
    let mut exit_app = controller.exit_app.write().await;
    *exit_app = true;
    controller.log(Level::Info, "bye~~".to_string()).await;
    let conn = controller.opt_conn().await;
    if conn.is_none() {
        return;
    }

    conn.as_ref().unwrap().closed().await;
}

async fn handle_key(code: KeyCode, controller: &Controller) {
    // controller.log(Level::Info, format!("key code {:?} ", code)).await;

    match code {
        KeyCode::Char(character) => {
            controller.input_write(character).await;
        }
        KeyCode::Delete => {
            controller.input_remove().await;
        }
        KeyCode::Backspace => {
            controller.input_remove_previous().await;
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End => {
            controller.input_move_cursor(code).await;
        }
        KeyCode::Up | KeyCode::Down | KeyCode::PageUp => {
            controller.messages_scroll(code).await;
        }
        KeyCode::Enter => {
            send_message(&controller).await;
        }
        _ => {}
    }

    controller.refresh().await;
}

const MAX_MSG_LEN: usize = 1024;

async fn send_message(ctrl: &Controller) {
    let vm = ctrl.get_view_model().await;
    if vm.input_vm.text.len() == 0 {
        return;
    }

    if vm.input_vm.text.len() > MAX_MSG_LEN {
        ctrl.log(Level::Warn, format!("msg len must lest lan {}", MAX_MSG_LEN)).await;
        return;
    }

    let msg: String = vm.input_vm.text.iter().collect();
    let chat_id = vm.conf.chat_id;
    let res = ctrl.send_msg(&msg, chat_id).await;
    if let Err(e) = res {
        ctrl.log(Level::Error, format!("send msg error : {}", e)).await;
        return;
    }

    ctrl.clean_input().await;
}



