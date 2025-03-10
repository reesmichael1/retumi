use tui_realm_stdlib::Phantom;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::{Component, Event, MockComponent};

use crate::event::RetumiEvent;
use crate::ui::Msg;

#[derive(Default, MockComponent)]
pub struct Closer {
    component: Phantom,
}

impl Component<Msg, RetumiEvent> for Closer {
    fn on(&mut self, ev: Event<RetumiEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::Quit),
            _ => None,
        }
    }
}
