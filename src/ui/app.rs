use crate::ui::hello::HelloWorld;
use crate::ui::menu;
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};

const WINDOW_SIZE: f32 = 480.0;

/// Starts the application and opens the root window, centered on screen.
pub fn run() {
    Application::new().run(|cx: &mut App| {
        menu::init(cx);
        let bounds = Bounds::centered(None, size(px(WINDOW_SIZE), px(WINDOW_SIZE)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HelloWorld::new()),
        )
        .expect("failed to open the root window");
        cx.activate(true);
    });
}
