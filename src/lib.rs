#![feature(macro_metavar_expr)]

use crossbeam::channel;
use html5ever::tendril::TendrilSink;
use html5ever::ParseOpts;
use markup5ever_rcdom::RcDom;

use crate::error::RetumiError;
use crate::js::{EngineContext, JsMessage, WorkerMsg};

mod error;
mod js;

pub async fn run_main() -> Result<(), RetumiError> {
    let html = std::fs::read_to_string("demo/hello.html")?;
    let mut dom = html5ever::parse_document(RcDom::default(), ParseOpts::default()).one(html);

    let (msg_tx, msg_rx) = channel::unbounded::<JsMessage>();
    let (worker_tx, worker_rx) = channel::unbounded::<WorkerMsg>();

    {
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
            })?;
    }

    let mut context = EngineContext::new();
    let src = std::fs::read_to_string("demo/hello.js")?;
    js::exec(
        &mut dom,
        &mut context,
        msg_rx.clone(),
        worker_tx.clone(),
        String::from("let a = 1;"),
    )?;
    js::exec(
        &mut dom,
        &mut context,
        msg_rx.clone(),
        worker_tx.clone(),
        src,
    )?;

    worker_tx.send(WorkerMsg::Shutdown)?;

    Ok(())
}
