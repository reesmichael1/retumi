use std::time::Duration;

use crate::js::{JsMessage, WorkerMsg};

use crossbeam::channel::{Receiver, Sender};
use tuirealm::event::NoUserEvent;
use tuirealm::props::{PropPayload, PropValue, TextSpan};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, AttrValue, Attribute, EventListenerCfg, Update};

use super::components::{ErrorBar, Page, UrlBar};
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
    pub tok_rx: tokio::sync::mpsc::Receiver<Msg>,
    tok_tx: tokio::sync::mpsc::Sender<Msg>,
    has_error: bool,
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
        assert!(app
            .mount(Id::ErrorBar, Box::new(ErrorBar::default()), vec![])
            .is_ok());
        assert!(app.active(&Id::UrlBar).is_ok());

        let (tok_tx, tok_rx) = tokio::sync::mpsc::channel(16);

        Self {
            app,
            quit: false,
            redraw: true,
            terminal: TerminalBridge::init_crossterm().expect("failed to initialize terminal"),
            msg_rx,
            worker_tx,
            tok_rx,
            tok_tx,
            has_error: false,
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
                let mut constraints = vec![Constraint::Length(3), Constraint::Fill(1)];
                if self.has_error {
                    constraints.push(Constraint::Length(1));
                }
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(&constraints)
                    .split(f.area());
                self.app.view(&Id::UrlBar, f, chunks[0]);
                self.app.view(&Id::Page, f, chunks[1]);

                if self.has_error {
                    self.app.view(&Id::ErrorBar, f, chunks[2]);
                }
            })
            .is_ok());
    }

    fn do_load_page(&mut self, url: String) {
        let tx = self.tok_tx.clone();
        let msg_rx = self.msg_rx.clone();
        let worker_tx = self.worker_tx.clone();
        tokio::spawn(async move {
            match browser::browse(url, msg_rx.clone(), worker_tx.clone()).await {
                Ok(contents) => tx.send(Msg::PageLoad(contents)).await.unwrap(),
                Err(err) => tx.send(Msg::FillError(err.to_string())).await.unwrap(),
            }
        });
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
                    self.do_load_page(url);
                    None
                }
                Msg::PageLoad(contents) => {
                    assert!(self.app.active(&Id::Page).is_ok());
                    self.has_error = false;
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
                Msg::FillError(err) => {
                    self.has_error = true;
                    assert!(self
                        .app
                        .attr(
                            &Id::ErrorBar,
                            Attribute::Text,
                            AttrValue::String(err.into()),
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
