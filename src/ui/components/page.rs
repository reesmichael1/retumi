use crossbeam::channel::Receiver;
use tui_realm_stdlib::Textarea;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::{Component, Event, MockComponent};

use crate::error::RetumiError;
use crate::event::RetumiEvent;
use crate::ui::Msg;

#[derive(MockComponent)]
pub struct Page {
    component: Textarea,
    rx: Receiver<Result<String, RetumiError>>,
}

impl Component<Msg, RetumiEvent> for Page {
    fn on(&mut self, ev: Event<RetumiEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::Quit),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(Msg::PageBlur),
            Event::User(RetumiEvent::PageReady) => {
                let contents = self.rx.recv().unwrap();
                match contents {
                    Ok(page) => Some(Msg::PageLoad(page)),
                    Err(err) => Some(Msg::FillError(err.to_string())),
                }
            }
            _ => None,
        }
    }
}

impl Page {
    pub fn new(rx: Receiver<Result<String, RetumiError>>) -> Self {
        Self {
            component: Textarea::default(),
            rx,
        }
    }
}
