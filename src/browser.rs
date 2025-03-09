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

pub async fn browse(
    url: String,
    msg_rx: Receiver<JsMessage>,
    worker_tx: Sender<WorkerMsg>,
) -> Result<String, RetumiError> {
    let config = html2text::config::plain();
    let content = reqwest::get(url).await?.text().await.unwrap();
    let mut dom = config.parse_html(std::io::Cursor::new(content))?;

    let mut context = EngineContext::new();
    let scripts = doc::extract_scripts(&dom);
    for script in scripts {
        js::exec(
            &mut dom,
            &mut context,
            msg_rx.clone(),
            worker_tx.clone(),
            doc::contents(&script),
        );
    }

    let rendered = render(&dom, &config)?;
    Ok(rendered)
}
