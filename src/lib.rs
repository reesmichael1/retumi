#![feature(macro_metavar_expr)]

use crossbeam::channel;
use tuirealm::Update;

use crate::error::RetumiError;
use crate::js::{JsMessage, WorkerMsg};

mod browser;
mod doc;
mod error;
mod js;
mod ui;

pub async fn run_main() -> Result<(), RetumiError> {
    let (msg_tx, msg_rx) = channel::unbounded::<JsMessage>();
    let (worker_tx, worker_rx) = channel::unbounded::<WorkerMsg>();

    let js_handle = {
        let rx = worker_rx.clone();
        let tx = msg_tx.clone();
        std::thread::Builder::new()
            .name(String::from("js_context"))
            .spawn(move || {
                if let Err(err) = js::run_worker(rx.clone(), tx.clone()) {
                    log::error!("{err}");
                    return Err(err);
                }

                Ok(())
            })?
    };

    let mut model = ui::Model::new(msg_rx, worker_tx.clone());
    model.terminal.enable_raw_mode().unwrap();
    model.terminal.enter_alternate_screen().unwrap();

    while !model.quit {
        if !model.tok_rx.is_empty() {
            model.redraw = true;
            let mut msg = Some(model.tok_rx.recv().await.unwrap());
            while msg.is_some() {
                msg = model.update(msg);
            }
        }

        match model.app.tick(tuirealm::PollStrategy::Once) {
            Err(err) => {
                eprintln!("{err}");
                break;
            }
            Ok(messages) => {
                if messages.len() > 0 {
                    model.redraw = true;
                    for msg in messages.into_iter() {
                        let mut msg = Some(msg);
                        while msg.is_some() {
                            msg = model.update(msg);
                        }
                    }
                }
            }
        }
        if model.redraw {
            model.redraw = false;
            model.view();
        }
    }

    let _ = model.terminal.leave_alternate_screen();
    let _ = model.terminal.disable_raw_mode();

    worker_tx.send(WorkerMsg::Shutdown)?;
    js_handle.join().unwrap()?;

    Ok(())
}
