use crate::js::{JsMessage, WorkerMsg};

use crossbeam::channel::{RecvError, SendError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetumiError {
    #[error("error while sending message from JS worker thread to main thread")]
    WorkerSendError(#[from] SendError<WorkerMsg>),
    #[error("error while sending message from main thread to JS worker thread")]
    MessageSendError(#[from] SendError<JsMessage>),
    #[error("error while receiving message in JS worker thread")]
    RecvError(#[from] RecvError),
    #[error("error while reading a local file")]
    IOError(#[from] std::io::Error),
    #[error("error while initializing JavaScript")]
    JsInitializeError,
    #[error("error while initializing JavaScript: {0}")]
    JsExecError(String),
    #[error("error while serializing object to JSON")]
    SerializeError(#[from] serde_json::Error),
    #[error("could not find handle submitted by JS")]
    InvalidHandle,
}
