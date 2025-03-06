use crate::js::{JsMessage, WorkerMsg};

use crossbeam::channel::{RecvError, SendError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetumiError {
    #[error("error while sending message from JS worker thread to main thread")]
    WorkerSendError(#[from] SendError<WorkerMsg>),
    #[error("error while sending message from main thread to JS worker thread: {0}")]
    MessageSendError(#[from] SendError<JsMessage>),
    #[error("error while receiving message: {0}")]
    RecvError(#[from] RecvError),
    #[error("error while reading a local file: {0}")]
    IOError(#[from] std::io::Error),
    #[error("error while initializing JavaScript: {0}")]
    JsInitializeError(String),
    #[error("error while executing JavaScript: {0}")]
    JsExecError(String),
    #[error("error while serializing object to JSON: {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("error while rendering: {0}")]
    RenderError(#[from] html2text::Error),
}
