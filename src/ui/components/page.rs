use tui_realm_stdlib::Textarea;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::ui::Msg;

#[derive(MockComponent)]
pub struct Page {
    component: Textarea,
}

impl Component<Msg, NoUserEvent> for Page {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::Quit),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::PageBlur),
            _ => {}
        }
        None
    }
}

impl Default for Page {
    fn default() -> Self {
        Self {
            component: Textarea::default(),
        }
    }
}
