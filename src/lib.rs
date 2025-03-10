#![feature(macro_metavar_expr)]

use crossbeam::channel;

use crate::error::RetumiError;
use crate::js::{JsMessage, WorkerMsg};

mod browser;
mod doc;
mod error;
mod event;
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
    model.run()?;

    worker_tx.send(WorkerMsg::Shutdown)?;
    js_handle.join().unwrap()?;

    Ok(())
}
