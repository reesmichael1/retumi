use tui_realm_stdlib::Label;
use tuirealm::props::Color;
use tuirealm::{Component, Event, MockComponent, NoUserEvent};

use crate::ui::Msg;

#[derive(MockComponent)]
pub struct ErrorBar {
    component: Label,
}

impl Default for ErrorBar {
    fn default() -> Self {
        Self {
            component: Label::default().foreground(Color::Red),
        }
    }
}

impl Component<Msg, NoUserEvent> for ErrorBar {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
