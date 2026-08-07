//! The root view: a navigation sidebar next to a content pane, with a
//! draggable divider between them.

pub mod resize;

use crate::feature::section::Section;
use crate::ui::split_pane::resize::ResizeState;
use gpui::{
    App, Context, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render, Window,
    canvas, div, prelude::*, px, rgb,
};

const SIDEBAR_BACKGROUND: u32 = 0x252526;
const CONTENT_BACKGROUND: u32 = 0x1e1e1e;
const BORDER_COLOR: u32 = 0x3c3c3c;
const ROW_SELECTED_BACKGROUND: u32 = 0x37373d;
const ROW_HOVER_BACKGROUND: u32 = 0x2a2d2e;
const PRIMARY_TEXT: u32 = 0xffffff;
const SECONDARY_TEXT: u32 = 0xcccccc;
const DIVIDER_WIDTH: f32 = 6.0;

/// The root view: a navigation sidebar on the left, a content pane on the
/// right, and a draggable divider between them.
pub struct SplitPane {
    resize: ResizeState,
    selected: Section,
}

impl SplitPane {
    pub fn new() -> Self {
        Self {
            resize: ResizeState::new(),
            selected: Section::MyPrs,
        }
    }

    fn sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(self.resize.width())
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(SIDEBAR_BACKGROUND))
            .border_r_1()
            .border_color(rgb(BORDER_COLOR))
            .children(
                Section::all()
                    .into_iter()
                    .enumerate()
                    .map(|(ix, section)| self.sidebar_row(section, ix, cx)),
            )
    }

    fn sidebar_row(&self, section: Section, ix: usize, cx: &Context<Self>) -> impl IntoElement {
        let selected = section == self.selected;
        div()
            .id(("section", ix))
            .px_2()
            .py_1()
            .cursor_pointer()
            .text_color(rgb(if selected {
                PRIMARY_TEXT
            } else {
                SECONDARY_TEXT
            }))
            .when(selected, |row| row.bg(rgb(ROW_SELECTED_BACKGROUND)))
            .hover(|row| row.bg(rgb(ROW_HOVER_BACKGROUND)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.selected = section;
                cx.notify();
            }))
            .child(section.title())
    }

    fn divider(&self, cx: &Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div()
            .w(px(DIVIDER_WIDTH))
            .flex_shrink_0()
            .h_full()
            .cursor_col_resize()
            .bg(rgb(BORDER_COLOR))
            .child(
                canvas(
                    |_, _, _| (),
                    move |handle_bounds, _, window, _| {
                        Self::register_drag_handlers(window, handle_bounds, entity.clone());
                    },
                )
                .size_full(),
            )
    }

    fn register_drag_handlers(
        window: &mut Window,
        handle_bounds: gpui::Bounds<gpui::Pixels>,
        entity: gpui::Entity<Self>,
    ) {
        let down_entity = entity.clone();
        window.on_mouse_event(
            move |event: &MouseDownEvent, phase, _window, cx: &mut App| {
                if !phase.bubble() || !handle_bounds.contains(&event.position) {
                    return;
                }
                let mouse_x = event.position.x;
                down_entity.update(cx, |this, _| this.resize.begin(mouse_x));
            },
        );

        let move_entity = entity.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx: &mut App| {
            if !phase.bubble() || !event.dragging() {
                return;
            }
            let mouse_x = event.position.x;
            let viewport_width = window.viewport_size().width;
            move_entity.update(cx, |this, _| this.resize.drag_to(mouse_x, viewport_width));
            cx.notify(move_entity.entity_id());
        });

        window.on_mouse_event(move |_event: &MouseUpEvent, phase, _window, cx: &mut App| {
            if !phase.bubble() {
                return;
            }
            entity.update(cx, |this, _| this.resize.end());
        });
    }

    fn content(&self) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .flex()
            .justify_center()
            .items_center()
            .bg(rgb(CONTENT_BACKGROUND))
            .text_xl()
            .text_color(rgb(PRIMARY_TEXT))
            .child(self.selected.title())
    }
}

impl Default for SplitPane {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for SplitPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(self.sidebar(cx))
            .child(self.divider(cx))
            .child(self.content())
    }
}
