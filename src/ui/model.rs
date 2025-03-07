use std::time::Duration;

use crate::js::{JsMessage, WorkerMsg};

use crossbeam::channel::{Receiver, Sender};
use tuirealm::event::NoUserEvent;
use tuirealm::props::{PropPayload, PropValue, TextSpan};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, AttrValue, Attribute, EventListenerCfg, Update};

use super::components::{Page, UrlBar};
use super::{Id, Msg};
use crate::browser;

pub struct Model<T>
where
    T: TerminalAdapter,
{
    pub app: Application<Id, Msg, NoUserEvent>,
    pub quit: bool,
    pub redraw: bool,
    pub terminal: TerminalBridge<T>,
    pub msg_rx: Receiver<JsMessage>,
    pub worker_tx: Sender<WorkerMsg>,
}

impl Model<CrosstermTerminalAdapter> {
    pub fn new(msg_rx: Receiver<JsMessage>, worker_tx: Sender<WorkerMsg>) -> Self {
        let mut app = Application::init(
            EventListenerCfg::default().crossterm_input_listener(Duration::from_millis(10), 10),
        );

        assert!(app
            .mount(Id::UrlBar, Box::new(UrlBar::default()), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Page, Box::new(Page::default()), vec![])
            .is_ok());
        assert!(app.active(&Id::UrlBar).is_ok());

        Self {
            app,
            quit: false,
            redraw: true,
            terminal: TerminalBridge::init_crossterm().expect("failed to initialize terminal"),
            msg_rx,
            worker_tx,
        }
    }
}

impl<T> Model<T>
where
    T: TerminalAdapter,
{
    pub fn view(&mut self) {
        assert!(self
            .terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Length(3), Constraint::Fill(1)])
                    .split(f.area());
                self.app.view(&Id::UrlBar, f, chunks[0]);
                self.app.view(&Id::Page, f, chunks[1]);
            })
            .is_ok());
    }
}

impl<T> Update<Msg> for Model<T>
where
    T: TerminalAdapter,
{
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            self.redraw = true;
            match msg {
                Msg::None => None,
                Msg::UrlBlur => {
                    assert!(self.app.active(&Id::Page).is_ok());
                    None
                }
                Msg::PageBlur => {
                    assert!(self.app.active(&Id::UrlBar).is_ok());
                    None
                }
                Msg::UrlSubmit(url) => {
                    let contents =
                        browser::browse(url, self.msg_rx.clone(), self.worker_tx.clone()).unwrap();
                    Some(Msg::PageLoad(contents))
                }
                Msg::PageLoad(contents) => {
                    assert!(self.app.active(&Id::Page).is_ok());
                    let lines: Vec<TextSpan> =
                        contents.lines().map(|s| s.to_string().into()).collect();
                    assert!(self
                        .app
                        .attr(
                            &Id::Page,
                            Attribute::Text,
                            AttrValue::Payload(PropPayload::Vec(
                                lines.iter().cloned().map(PropValue::TextSpan).collect(),
                            )),
                        )
                        .is_ok());
                    None
                }
                Msg::Quit => {
                    self.quit = true;
                    None
                }
            }
        } else {
            None
        }
    }
}
