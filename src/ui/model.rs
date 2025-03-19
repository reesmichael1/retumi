use std::time::Duration;

use crate::error::RetumiError;
use crate::event::{HttpClient, RetumiEvent};
use crate::js::{JsMessage, WorkerMsg};

use crossbeam::channel::{Receiver, Sender};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{PropPayload, PropValue};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{
    Application, AttrValue, Attribute, EventListenerCfg, Sub, SubClause, SubEventClause, Update,
};

use super::components::{Closer, ErrorBar, Page, UrlBar};
use super::{Id, Msg};

pub struct Model<T>
where
    T: TerminalAdapter,
{
    pub app: Application<Id, Msg, RetumiEvent>,
    pub quit: bool,
    pub redraw: bool,
    pub terminal: TerminalBridge<T>,
    http_tx: Sender<Msg>,
    has_error: bool,
}

impl Model<CrosstermTerminalAdapter> {
    pub fn new(msg_rx: Receiver<JsMessage>, worker_tx: Sender<WorkerMsg>) -> Self {
        let (http_tx, http_rx) = crossbeam::channel::bounded(16);
        let (content_tx, content_rx) = crossbeam::channel::bounded(16);

        let mut app = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(10), 10)
                .add_port(
                    Box::new(HttpClient::new(http_rx, content_tx)),
                    Duration::from_millis(10),
                    10,
                ),
        );

        assert!(app
            .mount(Id::UrlBar, Box::new(UrlBar::default()), vec![])
            .is_ok());
        assert!(app
            .mount(
                Id::Page,
                Box::new(Page::new(content_rx, msg_rx, worker_tx)),
                vec![Sub::new(
                    SubEventClause::User(RetumiEvent::PageReady),
                    SubClause::Always
                )]
            )
            .is_ok());
        assert!(app
            .mount(Id::ErrorBar, Box::new(ErrorBar::default()), vec![])
            .is_ok());
        assert!(app
            .mount(
                Id::Closer,
                Box::new(Closer::default()),
                vec![Sub::new(
                    SubEventClause::Keyboard(KeyEvent {
                        code: Key::Esc,
                        modifiers: KeyModifiers::NONE,
                    }),
                    SubClause::Always
                )]
            )
            .is_ok());
        assert!(app.active(&Id::UrlBar).is_ok());

        Self {
            app,
            quit: false,
            redraw: true,
            terminal: TerminalBridge::init_crossterm().expect("failed to initialize terminal"),
            http_tx,
            has_error: false,
        }
    }

    pub fn run(&mut self) -> Result<(), RetumiError> {
        self.terminal.enable_raw_mode()?;
        self.terminal.enter_alternate_screen()?;

        while !self.quit {
            match self.app.tick(tuirealm::PollStrategy::Once) {
                Err(err) => {
                    eprintln!("{err}");
                    break;
                }
                Ok(messages) => {
                    if messages.len() > 0 {
                        self.redraw = true;
                        for msg in messages.into_iter() {
                            let mut msg = Some(msg);
                            while msg.is_some() {
                                msg = self.update(msg);
                            }
                        }
                    }
                }
            }

            if self.redraw {
                self.redraw = false;
                self.view();
            }
        }

        self.terminal.leave_alternate_screen()?;
        self.terminal.disable_raw_mode()?;

        Ok(())
    }

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

    fn maybe_error(&self, res: Result<(), RetumiError>) -> Option<Msg> {
        match res {
            Ok(_) => None,
            Err(err) => Some(Msg::FillError(err.to_string())),
        }
    }

    fn do_load_page(&mut self, url: String) -> Result<(), RetumiError> {
        self.http_tx
            .send(Msg::UrlSubmit(url))
            .map_err(|_| RetumiError::ChannelError)
    }
}

impl Update<Msg> for Model<CrosstermTerminalAdapter> {
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
                    let res = self.do_load_page(url);
                    self.maybe_error(res)
                }
                Msg::PageLoad(contents) => {
                    assert!(self.app.active(&Id::Page).is_ok());
                    self.has_error = false;
                    assert!(self
                        .app
                        .attr(
                            &Id::Page,
                            Attribute::Text,
                            AttrValue::Payload(PropPayload::Vec(
                                contents.into_iter().map(PropValue::TextSpan).collect()
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
