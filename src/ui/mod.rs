mod components;
mod model;

pub use model::Model;
use tuirealm::props::TextSpan;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    None,
    Quit,
    UrlBlur,
    UrlSubmit(String),
    PageLoad(Vec<TextSpan>),
    FillError(String),
    PageBlur,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    UrlBar,
    ErrorBar,
    Page,
    Closer,
}
