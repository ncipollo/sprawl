use crate::feature::config;
use crate::ui::menu;
use crate::ui::split_pane::SplitPane;
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 720.0;

/// Starts the application and opens the root window, centered on screen.
pub fn run() {
    let sections = config::startup_sections();
    Application::new().run(|cx: &mut App| {
        menu::init(cx);
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| SplitPane::new(sections, cx)),
        )
        .expect("failed to open the root window");
        cx.activate(true);
    });
}
