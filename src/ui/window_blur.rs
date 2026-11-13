//! Backdrop blur for the root window. gpui's own `Blurred` appearance relies
//! on an `NSVisualEffectView` that does not blur on recent macOS, so on macOS
//! we also ask the WindowServer for a blur directly.

use gpui::Window;

/// Blurs whatever is behind `window`. A no-op off macOS.
pub fn apply(window: &Window) {
    #[cfg(target_os = "macos")]
    macos::apply(window);
    #[cfg(not(target_os = "macos"))]
    let _ = window;
}

#[cfg(target_os = "macos")]
// objc 0.2's msg_send! expands to a stale `cfg(feature = "cargo-clippy")`.
#[allow(unexpected_cfgs)]
mod macos {
    use gpui::Window;
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const BLUR_RADIUS: i32 = 40;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGSMainConnectionID() -> u32;
        fn CGSSetWindowBackgroundBlurRadius(connection: u32, window: isize, radius: i32) -> i32;
    }

    pub fn apply(window: &Window) {
        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
            return;
        };
        let view = appkit.ns_view.as_ptr() as *mut Object;
        // SAFETY: `view` is the live NSView backing `window`, and these are
        // plain Objective-C getters plus a WindowServer call.
        unsafe {
            let ns_window: *mut Object = msg_send![view, window];
            if ns_window.is_null() {
                return;
            }
            let number: isize = msg_send![ns_window, windowNumber];
            CGSSetWindowBackgroundBlurRadius(CGSMainConnectionID(), number, BLUR_RADIUS);
        }
    }
}
