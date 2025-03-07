mod components;
mod model;

pub use model::Model;

#[derive(Debug, PartialEq)]
pub enum Msg {
    None,
    Quit,
    UrlBlur,
    UrlSubmit(String),
    PageLoad(String),
    FillError(String),
    PageBlur,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    UrlBar,
    ErrorBar,
    Page,
}
