//! Pure sidebar-resize math, kept separate from rendering so it can be unit
//! tested without booting a gpui `App`.

use gpui::{Pixels, px};

/// The sidebar width used before the user drags the divider.
pub const DEFAULT_SIDEBAR_WIDTH: Pixels = px(220.0);
/// The narrowest the sidebar can be dragged.
pub const MIN_SIDEBAR_WIDTH: Pixels = px(140.0);
/// The narrowest the content pane can be left after a drag.
pub const MIN_CONTENT_WIDTH: Pixels = px(240.0);

/// The mouse position and sidebar width recorded when a divider drag starts.
struct DragOrigin {
    mouse_x: Pixels,
    width: Pixels,
}

/// Tracks the sidebar width and any in-progress divider drag.
pub struct ResizeState {
    width: Pixels,
    drag: Option<DragOrigin>,
}

impl ResizeState {
    /// Starts with the default sidebar width and no drag in progress.
    pub fn new() -> Self {
        Self {
            width: DEFAULT_SIDEBAR_WIDTH,
            drag: None,
        }
    }

    /// The current sidebar width.
    pub fn width(&self) -> Pixels {
        self.width
    }

    /// Whether the divider is currently being dragged.
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Begins a divider drag at the given mouse x position.
    pub fn begin(&mut self, mouse_x: Pixels) {
        self.drag = Some(DragOrigin {
            mouse_x,
            width: self.width,
        });
    }

    /// Updates the sidebar width for a divider drag to the given mouse x
    /// position, clamped so both panes keep a minimum width. Does nothing if
    /// no drag is in progress.
    pub fn drag_to(&mut self, mouse_x: Pixels, viewport_width: Pixels) {
        let Some(origin) = &self.drag else {
            return;
        };
        let max_width = (viewport_width - MIN_CONTENT_WIDTH).max(MIN_SIDEBAR_WIDTH);
        let target = origin.width + (mouse_x - origin.mouse_x);
        self.width = target.clamp(MIN_SIDEBAR_WIDTH, max_width);
    }

    /// Ends the current divider drag, if any, keeping the current width.
    pub fn end(&mut self) {
        self.drag = None;
    }
}

impl Default for ResizeState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_the_default_width_with_no_drag() {
        let state = ResizeState::new();

        assert_eq!(state.width(), DEFAULT_SIDEBAR_WIDTH);
        assert!(!state.is_dragging());
    }

    #[test]
    fn drag_to_moves_the_edge_by_the_mouse_delta() {
        let mut state = ResizeState::new();
        let viewport_width = px(1024.0);

        state.begin(px(300.0));
        assert!(state.is_dragging());
        state.drag_to(px(350.0), viewport_width);

        assert_eq!(state.width(), DEFAULT_SIDEBAR_WIDTH + px(50.0));

        state.drag_to(px(280.0), viewport_width);

        assert_eq!(state.width(), DEFAULT_SIDEBAR_WIDTH - px(20.0));
    }

    #[test]
    fn drag_to_clamps_at_the_minimum_sidebar_width() {
        let mut state = ResizeState::new();

        state.begin(px(300.0));
        state.drag_to(px(0.0), px(1024.0));

        assert_eq!(state.width(), MIN_SIDEBAR_WIDTH);
    }

    #[test]
    fn drag_to_clamps_at_the_minimum_content_width() {
        let mut state = ResizeState::new();
        let viewport_width = px(1024.0);

        state.begin(px(300.0));
        state.drag_to(px(10_000.0), viewport_width);

        assert_eq!(state.width(), viewport_width - MIN_CONTENT_WIDTH);
    }

    #[test]
    fn drag_to_without_begin_is_a_no_op() {
        let mut state = ResizeState::new();

        state.drag_to(px(900.0), px(1024.0));

        assert_eq!(state.width(), DEFAULT_SIDEBAR_WIDTH);
    }

    #[test]
    fn end_clears_the_drag_and_keeps_the_width() {
        let mut state = ResizeState::new();

        state.begin(px(300.0));
        state.drag_to(px(350.0), px(1024.0));
        state.end();

        assert!(!state.is_dragging());
        assert_eq!(state.width(), DEFAULT_SIDEBAR_WIDTH + px(50.0));
    }
}
