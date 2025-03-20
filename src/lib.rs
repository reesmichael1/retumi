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

// Based on https://ratatui.rs/recipes/apps/log-with-tracing/
pub(crate) mod tracelog {
    /// Similar to the `std::dbg!` macro, but generates `tracing` events rather
    /// than printing to stdout.
    ///
    /// By default, the verbosity level for the generated events is `DEBUG`, but
    /// this can be customized.
    #[macro_export]
    macro_rules! trace_dbg {
        (target: $target:expr, level: $level:expr, $ex:expr) => {{
            match $ex {
                value => {
                    tracing::event!(target: $target, $level, ?value, stringify!($ex));
                    value
                }
            }
        }};
        (level: $level:expr, $ex:expr) => {
            trace_dbg!(target: module_path!(), level: $level, $ex)
        };
        (target: $target:expr, $ex:expr) => {
            trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
        };
        ($ex:expr) => {
            trace_dbg!(level: tracing::Level::DEBUG, $ex)
        };
    }

    pub(crate) use trace_dbg;
}

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
                    tracelog::trace_dbg!(&err);
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
