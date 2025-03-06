use html2text::markup5ever_rcdom::{Handle, NodeData, RcDom};
use html5ever::local_name;

pub fn extract_scripts(dom: &RcDom) -> Vec<Handle> {
    let mut result = vec![];

    fn walker(node: Handle, result: &mut Vec<Handle>) {
        match &node.data {
            NodeData::Element { name, .. } => {
                if name.local == local_name!("script") {
                    result.push(node.clone());
                }
            }
            _ => {}
        }

        for child in node.children.borrow().iter() {
            walker(child.clone(), result);
        }
    }

    walker(dom.document.clone(), &mut result);
    result
}

pub fn contents(script: &Handle) -> String {
    for child in script.children.borrow().iter() {
        match &child.data {
            NodeData::Text { contents } => {
                let s: String = contents.borrow().clone().into();
                return s;
            }
            _ => todo!(),
        }
    }

    String::new()
}
