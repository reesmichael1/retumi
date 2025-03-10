//use super::Label;

use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, Color, InputType};
use tuirealm::{Component, Event, MockComponent};

use crate::event::RetumiEvent;
use crate::ui::Msg;

#[derive(MockComponent)]
pub struct UrlBar {
    component: Input,
}

impl Component<Msg, RetumiEvent> for UrlBar {
    fn on(&mut self, ev: Event<RetumiEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::UrlBlur),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::Quit),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let url = self.component.states.get_value();
                return Some(Msg::UrlSubmit(url));
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Default for UrlBar {
    fn default() -> Self {
        Self {
            component: Input::default()
                .foreground(Color::Blue)
                .title("URL", Alignment::Center)
                .input_type(InputType::Text),
        }
    }
}
