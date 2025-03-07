use crossbeam::channel::{Receiver, Sender};
use html2text::config::Config;
use html2text::render::PlainDecorator;
use html2text::RcDom;

use crate::doc;
use crate::error::RetumiError;
use crate::js;
use crate::js::{EngineContext, JsMessage, WorkerMsg};

fn render(dom: &RcDom, config: &Config<PlainDecorator>) -> Result<String, RetumiError> {
    let tree = config.dom_to_render_tree(dom)?;
    let rendered = config.render_to_string(tree, 120)?;
    Ok(rendered)
}

pub fn browse(
    path: String,
    msg_rx: Receiver<JsMessage>,
    worker_tx: Sender<WorkerMsg>,
) -> Result<String, RetumiError> {
    let config = html2text::config::plain();
    let file = std::fs::File::open(path)?;
    let mut dom = config.parse_html(file)?;

    let scripts = doc::extract_scripts(&dom);

    let mut context = EngineContext::new();
    js::exec(
        &mut dom,
        &mut context,
        msg_rx.clone(),
        worker_tx.clone(),
        doc::contents(&scripts[0]),
    );

    let rendered = render(&dom, &config)?;
    Ok(rendered)
}
