use crate::feature::section::Section;
use crate::feature::section::source::SectionSource;
use crate::ui::menu;
use crate::ui::split_pane::SplitPane;
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};
use std::sync::Arc;

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 720.0;

/// Starts the application and opens the root window, centered on screen.
/// `sections` come from `source`, which also supplies their content;
/// `no_sections_message` is shown when there are none.
pub fn run(sections: Vec<Section>, source: Arc<dyn SectionSource>, no_sections_message: String) {
    Application::new().run(move |cx: &mut App| {
        menu::init(cx);
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(|cx| SplitPane::new(sections, source, no_sections_message, cx)),
            )
            .expect("failed to open the root window");
        cx.on_action(move |_: &menu::Refresh, cx: &mut App| {
            // Menu-triggered actions are dispatched from inside an update of
            // this same window, so updating it again here must be deferred
            // until that update finishes — otherwise it fails as a reentrant
            // borrow.
            cx.defer(move |cx| {
                window
                    .update(cx, |pane, _window, cx| pane.refresh_selected(cx))
                    .ok();
            });
        });
        cx.activate(true);
    });
}
