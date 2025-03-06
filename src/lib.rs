#![feature(macro_metavar_expr)]

use crossbeam::channel;
use html2text::render::PlainDecorator;
use html2text::{config::Config, RcDom};

use crate::error::RetumiError;
use crate::js::{EngineContext, JsMessage, WorkerMsg};

mod doc;
mod error;
mod js;

pub fn render(dom: &RcDom, config: &Config<PlainDecorator>) -> Result<String, RetumiError> {
    let tree = config.dom_to_render_tree(dom)?;
    let rendered = config.render_to_string(tree, 120)?;
    Ok(rendered)
}

pub async fn run_main() -> Result<(), RetumiError> {
    let config = html2text::config::plain();

    let file = std::fs::File::open("demo/hello.html")?;
    let mut dom = config.parse_html(file)?;

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

    let scripts = doc::extract_scripts(&dom);

    let mut context = EngineContext::new();
    js::exec(
        &mut dom,
        &mut context,
        msg_rx.clone(),
        worker_tx.clone(),
        doc::contents(&scripts[0]),
    );

    worker_tx.send(WorkerMsg::Shutdown)?;
    js_handle.join().unwrap()?;

    let rendered = render(&dom, &config)?;
    println!("{rendered}");
    Ok(())
}
