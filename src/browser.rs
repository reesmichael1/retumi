use crossbeam::channel::{Receiver, Sender};
use html2text::config::Config;
use html2text::render::{RichAnnotation, RichDecorator, TextDecorator};
use html2text::{Colour, RcDom};

use tuirealm::props::{Style, TextSpan};
use tuirealm::ratatui::style::Modifier;

use crate::doc;
use crate::error::RetumiError;
use crate::js;
use crate::js::{EngineContext, JsMessage, WorkerMsg};

#[derive(Clone, Debug)]
struct RetumiRenderer {
    decorator: RichDecorator,
    selected_link: Option<usize>,
    link_count: usize,
    selected_url: Option<String>,
}

impl RetumiRenderer {
    fn new() -> Self {
        Self {
            decorator: RichDecorator::new(),
            selected_link: None,
            link_count: 0,
            selected_url: None,
        }
    }
}

impl TextDecorator for RetumiRenderer {
    type Annotation = RichAnnotation;

    fn decorate_link_start(&mut self, url: &str) -> (String, Self::Annotation) {
        self.link_count += 1;
        if self.selected_link == Some(self.link_count) {
            let style = RichAnnotation::ActiveLink(url.to_string());
            let (str, _) = self.decorator.decorate_link_start(url);
            self.selected_url = Some(url.to_string());
            (str, style)
        } else {
            self.decorator.decorate_link_start(url)
        }
    }

    fn decorate_link_end(&mut self) -> String {
        self.decorator.decorate_link_end()
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        self.decorator.decorate_em_start()
    }

    fn decorate_em_end(&self) -> String {
        self.decorator.decorate_em_end()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        self.decorator.decorate_strong_start()
    }

    fn decorate_strong_end(&self) -> String {
        self.decorator.decorate_strong_end()
    }

    fn make_subblock_decorator(&self) -> Self {
        // We don't need to reassign self.decorator
        // because RichDecorator::make_subblock_decorator doesn't do anything.
        // We might eventually need to handle updates propogated by the subblock decorator
        self.clone()
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        self.decorator.ordered_item_prefix(i)
    }

    fn unordered_item_prefix(&self) -> String {
        self.decorator.unordered_item_prefix()
    }

    fn quote_prefix(&self) -> String {
        self.decorator.quote_prefix()
    }

    fn header_prefix(&self, level: usize) -> String {
        self.decorator.header_prefix(level)
    }

    fn decorate_image(&mut self, src: &str, title: &str) -> (String, Self::Annotation) {
        self.decorator.decorate_image(src, title)
    }

    fn decorate_preformat_cont(&self) -> Self::Annotation {
        self.decorator.decorate_preformat_cont()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {
        self.decorator.decorate_preformat_first()
    }

    fn decorate_code_end(&self) -> String {
        self.decorator.decorate_code_end()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        self.decorator.decorate_code_start()
    }

    fn decorate_strikeout_end(&self) -> String {
        self.decorator.decorate_strikeout_end()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        self.decorate_code_start()
    }
}

fn to_style(tags: &[RichAnnotation]) -> Option<Style> {
    let mut style = Style::default()
        .fg(tuirealm::props::Color::White)
        .bg(tuirealm::props::Color::Black);
    let mut applied = false;

    for ann in tags {
        match *ann {
            RichAnnotation::Default => {}
            RichAnnotation::Link(_) => {
                applied = true;
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            RichAnnotation::ActiveLink(_) => {
                applied = true;
                style = style
                    .bg(tuirealm::props::Color::Blue)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
            }
            RichAnnotation::Strong => {
                applied = true;
                style = style
                    .add_modifier(Modifier::BOLD)
                    .fg(tuirealm::props::Color::Green);
            }
            RichAnnotation::BgColour(Colour { r, g, b }) => {
                applied = true;
                style = style.bg(tuirealm::props::Color::Rgb(r, g, b));
            }
            _ => {}
        }
    }

    if applied {
        Some(style)
    } else {
        None
    }
}

pub struct Browser {
    dom: RcDom,
    pub current_link: Option<usize>,
    config: Config<RetumiRenderer>,
    msg_rx: Receiver<JsMessage>,
    worker_tx: Sender<WorkerMsg>,
}

impl Browser {
    pub fn new(msg_rx: Receiver<JsMessage>, worker_tx: Sender<WorkerMsg>) -> Self {
        Self {
            dom: RcDom::default(),
            current_link: None,
            config: html2text::config::with_decorator(RetumiRenderer::new()),
            msg_rx,
            worker_tx,
        }
    }

    pub fn render_contents(&mut self, contents: &str) -> Result<Vec<TextSpan>, RetumiError> {
        let mut dom = self.config.parse_html(std::io::Cursor::new(contents))?;

        let mut context = EngineContext::new();
        let scripts = doc::extract_scripts(&dom);
        for script in scripts {
            js::exec(
                &mut dom,
                &mut context,
                self.msg_rx.clone(),
                self.worker_tx.clone(),
                doc::contents(&script),
            );
        }

        self.dom = dom;
        // Reset the rendering params so that we start with a clean page
        self.current_link = None;
        self.config.decorator = RetumiRenderer::new();
        self.render()
    }

    pub fn get_active_link(&mut self) -> Option<String> {
        self.config.decorator.selected_url.clone()
    }

    pub fn cycle_link(&mut self) -> Result<Vec<TextSpan>, RetumiError> {
        if let Some(link) = self.current_link {
            if link == self.config.decorator.link_count {
                self.current_link = Some(1);
            } else {
                self.current_link = Some(link + 1);
            }
        } else if self.config.decorator.link_count > 0 {
            self.current_link = Some(1);
        }

        self.render()
    }

    fn render(&mut self) -> Result<Vec<TextSpan>, RetumiError> {
        self.config.decorator = RetumiRenderer::new();
        self.config.decorator.selected_link = self.current_link;

        let tree = self.config.dom_to_render_tree(&self.dom)?;
        let (rendered, dec) = self.config.render_to_lines_and_dec(tree, 120)?;
        self.config.decorator = dec;

        let mut result = Vec::new();

        for line in rendered {
            let strings = line.tagged_strings();
            for ts in strings {
                if let Some(style) = to_style(&ts.tag) {
                    let span = TextSpan {
                        content: ts.s.clone(),
                        fg: style.fg.unwrap(),
                        bg: style.bg.unwrap(),
                        modifiers: style.add_modifier,
                    };
                    result.push(span);
                } else {
                    result.push(TextSpan::new(ts.s.clone()));
                }
            }
        }

        Ok(result)
    }
}
