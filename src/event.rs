use crate::browser;
use crate::error::RetumiError;
use crate::js::{JsMessage, WorkerMsg};
use crate::ui::Msg;

use crossbeam::channel::{Receiver, Sender};
use tokio::runtime::Runtime;
use tuirealm::listener::{ListenerResult, Poll};
use tuirealm::Event;

#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub enum RetumiEvent {
    PageReady,
}

pub struct HttpClient {
    rx: Receiver<Msg>,
    tok_tx: tokio::sync::mpsc::Sender<Msg>,
    runtime: Runtime,
}

impl HttpClient {
    pub fn new(
        rx: Receiver<Msg>,
        tx: Sender<Result<String, RetumiError>>,
        msg_rx: Receiver<JsMessage>,
        worker_tx: Sender<WorkerMsg>,
    ) -> Self {
        let (tok_tx, mut tok_rx) = tokio::sync::mpsc::channel(16);

        {
            let msg_rx = msg_rx.clone();
            let worker_tx = worker_tx.clone();
            tokio::spawn(async move {
                loop {
                    let msg = tok_rx.recv().await.unwrap();
                    match msg {
                        Msg::UrlSubmit(url) => {
                            let page =
                                browser::browse(url, msg_rx.clone(), worker_tx.clone()).await;
                            tx.send(page).unwrap();
                        }
                        Msg::Quit => break,
                        _ => {}
                    }
                }
            });
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .build()
            .expect("Failed to initialize Tokio runtime");

        Self {
            rx,
            tok_tx,
            runtime,
        }
    }

    pub async fn start_page_load(&self, url: String) {
        let tx = self.tok_tx.clone();
        tx.send(Msg::UrlSubmit(url)).await.unwrap();
    }
}

impl Drop for HttpClient {
    fn drop(&mut self) {
        let tx = self.tok_tx.clone();
        self.runtime.block_on(async move {
            tx.send(Msg::Quit)
                .await
                .expect("could not send termination message to I/O thread");
        });
    }
}

impl Poll<RetumiEvent> for HttpClient {
    fn poll(&mut self) -> ListenerResult<Option<Event<RetumiEvent>>> {
        if !self.rx.is_empty() {
            let msg = self
                .rx
                .recv()
                .map_err(|_| tuirealm::ListenerError::PollFailed)?;
            match msg {
                Msg::UrlSubmit(url) => {
                    self.runtime.block_on(async {
                        self.start_page_load(url).await;
                    });
                    Ok(Some(Event::User(RetumiEvent::PageReady)))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}
