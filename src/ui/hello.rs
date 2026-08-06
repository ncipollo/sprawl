use crate::feature::greeting;
use gpui::{Context, IntoElement, SharedString, Window, div, prelude::*, rgb};

/// The root view: a single centered greeting.
pub struct HelloWorld {
    message: SharedString,
}

impl HelloWorld {
    pub fn new() -> Self {
        Self {
            message: greeting::message().into(),
        }
    }
}

impl Default for HelloWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .justify_center()
            .items_center()
            .bg(rgb(0x1e1e1e))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(self.message.clone())
    }
}
