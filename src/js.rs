use std::rc::Rc;

use boa_engine::{Context, JsValue, NativeFunction, Source};
use crossbeam::channel::{Receiver, Sender};
use html5ever::local_name;
use html5ever::tree_builder::TreeSink;
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom};

use crate::error::RetumiError;

#[derive(Debug, Clone)]
pub enum JsMessage {
    Print(String),
    GetElementById(String),
    QuerySelector(String),
    GetAttribute(usize, String),
    SetAttribute(usize, String, String),
    Done,
}

#[derive(Debug, Clone)]
pub enum WorkerMsg {
    Response(serde_json::Value),
    Execute(String),
    Shutdown,
}

pub struct EngineContext {
    handles: Vec<(Rc<Node>, usize)>,
}

impl<'a> EngineContext {
    pub fn new() -> Self {
        Self { handles: vec![] }
    }

    pub fn get_handle(&mut self, dom: &RcDom, node: &Handle) -> usize {
        for (n, h) in &self.handles {
            if dom.same_node(n, node) {
                return *h;
            }
        }

        let count = self.handles.len();
        self.handles.push((node.clone(), count));
        return count;
    }

    pub fn get_element(&self, handle: usize) -> Option<Handle> {
        for (n, h) in &self.handles {
            if handle == *h {
                return Some(n.clone());
            }
        }

        None
    }

    pub fn get_element_mut(&mut self, handle: usize) -> Option<&mut Handle> {
        for (ref mut n, h) in &mut self.handles {
            if handle == *h {
                return Some(n);
            }
        }

        None
    }
}

fn jsval_to_int(val: &JsValue) -> usize {
    val.as_number().unwrap() as usize
}

fn jsval_to_string(val: &JsValue) -> String {
    val.as_string().unwrap().to_std_string_escaped()
}

fn initialize_context(
    rx: Receiver<WorkerMsg>,
    tx: Sender<JsMessage>,
) -> Result<Context, RetumiError> {
    let mut ctx = Context::default();

    macro_rules! js_func {
        ($name: expr, $signal: expr, $($chain: expr),*) => {
            let tx = tx.clone();
            let rx = rx.clone();
            unsafe {
                ctx.register_global_builtin_callable(
                    $name.into(),
                    ${count($chain)},
                    NativeFunction::from_closure(move |_this, args, ctx| {
                        tx.send($signal($($chain(args.get(${index()}).unwrap())),*)).unwrap();

                        let response = rx.recv().unwrap();
                        match response {
                            WorkerMsg::Response(res) => {
                                Ok(JsValue::from_json(&res, ctx).unwrap())
                            },
                            _ => unreachable!(),
                        }
                    }),
                )
                .map_err(|_| RetumiError::JsInitializeError)?;
            }
        };
    }

    js_func!("logInner", JsMessage::Print, jsval_to_string);
    js_func!(
        "getElementByIdInner",
        JsMessage::GetElementById,
        jsval_to_string
    );
    js_func!(
        "querySelectorInner",
        JsMessage::QuerySelector,
        jsval_to_string
    );
    js_func!(
        "getAttributeInner",
        JsMessage::GetAttribute,
        jsval_to_int,
        jsval_to_string
    );
    js_func!(
        "setAttributeInner",
        JsMessage::SetAttribute,
        jsval_to_int,
        jsval_to_string,
        jsval_to_string
    );

    let runtime_js = "
    console = { log: function(x) { logInner(String(x)) } }

    const Node = class {
        constructor(handle) {
            this.handle = handle;
        }

        getAttribute(attr) {
            return getAttributeInner(this.handle, attr);
        }

        setAttribute(attr, val) {
            return setAttributeInner(this.handle, attr, val);
        }
    }

    document = {
        querySelectorAll: function(s) {
            var handles = querySelectorInner(s);
            return handles.map(h => new Node(h));
        },

        getElementById: function(id) {
            const handle = getElementByIdInner(id);
            if (handle == null) {
                return null;
            } else {
                return new Node(handle);
            }
        }
    }";

    ctx.eval(Source::from_bytes(runtime_js))
        .map_err(|_| RetumiError::JsInitializeError)?;

    Ok(ctx)
}

pub fn run(rx: Receiver<WorkerMsg>, tx: Sender<JsMessage>) -> Result<(), RetumiError> {
    let mut ctx = initialize_context(rx.clone(), tx.clone())?;

    loop {
        let msg = rx.recv()?;
        match msg {
            WorkerMsg::Execute(src) => {
                ctx.eval(Source::from_bytes(&src))
                    .map_err(|err| RetumiError::JsExecError(err.to_string()))?;
                tx.send(JsMessage::Done).unwrap();
            }
            WorkerMsg::Response(_) => {
                break Err(RetumiError::JsExecError(
                    "got unexpected response message".to_string(),
                ))
            }
            WorkerMsg::Shutdown => {
                tx.send(JsMessage::Done)?;
                break Ok(());
            }
        }
    }
}

pub fn exec(
    dom: &mut RcDom,
    js_state: &mut EngineContext,
    rx: Receiver<JsMessage>,
    tx: Sender<WorkerMsg>,
    code: String,
) -> Result<(), RetumiError> {
    tx.send(WorkerMsg::Execute(code)).unwrap();

    loop {
        match rx.recv()? {
            JsMessage::Print(msg) => {
                println!("{msg}");
                tx.send(WorkerMsg::Response(serde_json::to_value(None::<()>)?))?;
            }
            JsMessage::GetElementById(id) => {
                fn walker(
                    this: &mut EngineContext,
                    dom: &RcDom,
                    node: Rc<Node>,
                    sel: &str,
                ) -> Option<usize> {
                    match &node.data {
                        NodeData::Element { attrs, .. } => {
                            for attr in attrs.borrow().iter() {
                                if attr.name.local == local_name!("id") && attr.value == sel.into()
                                {
                                    return Some(this.get_handle(dom, &node));
                                }
                            }
                        }
                        _ => {}
                    }

                    for child in node.children.borrow().iter() {
                        if let Some(h) = walker(this, dom, child.clone(), sel) {
                            return Some(h);
                        }
                    }
                    None
                }

                if let Some(h) = walker(js_state, &dom, dom.document.clone(), &id) {
                    tx.send(WorkerMsg::Response(serde_json::to_value(h)?))?;
                } else {
                    tx.send(WorkerMsg::Response(serde_json::to_value(None::<()>)?))?;
                }
            }
            JsMessage::QuerySelector(tag) => {
                let mut result = Vec::new();

                fn walker(
                    this: &mut EngineContext,
                    dom: &RcDom,
                    node: Rc<Node>,
                    sel: &str,
                    result: &mut Vec<usize>,
                ) {
                    match &node.data {
                        NodeData::Element { name, .. } => {
                            if &name.local == sel {
                                result.push(this.get_handle(dom, &node));
                            }
                        }
                        _ => {}
                    }

                    for child in node.children.borrow().iter() {
                        walker(this, dom, child.clone(), sel, result);
                    }
                }

                walker(js_state, dom, dom.document.clone(), &tag, &mut result);
                tx.send(WorkerMsg::Response(serde_json::to_value(result)?))?;
            }
            JsMessage::GetAttribute(handle, name) => {
                let mut sent = false;
                if let Some(node) = js_state.get_element(handle) {
                    match &node.data {
                        NodeData::Element { attrs, .. } => {
                            for attr in attrs.borrow().iter() {
                                if attr.name.local == name {
                                    let val = String::from(&attr.value);
                                    sent = true;
                                    tx.send(WorkerMsg::Response(serde_json::to_value(val)?))?;
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    // TODO: actually have good error handling
                }

                if !sent {
                    tx.send(WorkerMsg::Response(serde_json::to_value(None::<()>)?))?;
                }
            }
            JsMessage::SetAttribute(handle, name, value) => {
                if let Some(node) = js_state.get_element_mut(handle) {
                    match &node.data {
                        NodeData::Element { attrs, .. } => {
                            for attr in attrs.borrow_mut().iter_mut() {
                                if attr.name.local == name {
                                    attr.value = value.into();
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                tx.send(WorkerMsg::Response(serde_json::to_value(None::<()>)?))?;
            }
            JsMessage::Done => break Ok(()),
        }
    }
}
