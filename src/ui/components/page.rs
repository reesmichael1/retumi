use crossbeam::channel::{Receiver, Sender};
use tui_realm_stdlib::Textarea;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::{Component, Event, MockComponent};

use crate::browser::Browser;
use crate::event::RetumiEvent;
use crate::js::{JsMessage, WorkerMsg};
use crate::ui::Msg;

#[derive(MockComponent)]
pub struct Page {
    component: Textarea,
    rx: Receiver<Option<String>>,
    browser: Browser,
}

impl Component<Msg, RetumiEvent> for Page {
    fn on(&mut self, ev: Event<RetumiEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(Msg::PageBlur),
            Event::Keyboard(KeyEvent {
                code: Key::Char(' '),
                ..
            }) => {
                let contents = self.browser.cycle_link().unwrap();
                Some(Msg::PageLoad(contents))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.browser.get_active_link().map(Msg::UrlSubmit),
            Event::User(RetumiEvent::PageReady) => {
                let msg = self.rx.recv().unwrap();
                if let Some(contents) = msg {
                    match self.browser.render_contents(&contents) {
                        Ok(page) => Some(Msg::PageLoad(page)),
                        Err(err) => Some(Msg::FillError(err.to_string())),
                    }
                } else {
                    Some(Msg::FillError("could not load URL".to_string()))
                }
            }
            _ => None,
        }
    }
}

impl Page {
    pub fn new(
        rx: Receiver<Option<String>>,
        msg_rx: Receiver<JsMessage>,
        worker_tx: Sender<WorkerMsg>,
    ) -> Self {
        Self {
            component: Textarea::default(),
            rx,
            browser: Browser::new(msg_rx, worker_tx),
        }
    }
}
