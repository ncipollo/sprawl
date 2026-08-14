//! The root view: a navigation sidebar next to a content pane, with a
//! draggable divider between them.

pub mod resize;

use crate::feature::section::Section;
use crate::ui::colors;
use crate::ui::section_pane::SectionPane;
use crate::ui::split_pane::resize::ResizeState;
use gpui::{
    App, Context, Entity, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render,
    Subscription, Window, canvas, div, prelude::*, px, rgb,
};

const DIVIDER_WIDTH: f32 = 6.0;

/// The root view: a navigation sidebar on the left, a content pane on the
/// right, and a draggable divider between them.
pub struct SplitPane {
    resize: ResizeState,
    sections: Vec<Section>,
    selected: usize,
    content: Entity<SectionPane>,
    /// Re-renders the sidebar when the content pane learns a section's
    /// script-provided title.
    _content_observation: Subscription,
}

impl SplitPane {
    pub fn new(sections: Vec<Section>, cx: &mut Context<Self>) -> Self {
        let content = cx.new(|cx| SectionPane::new(&sections, cx));
        let observation = cx.observe(&content, |_, _, cx| cx.notify());
        Self {
            resize: ResizeState::new(),
            sections,
            selected: 0,
            content,
            _content_observation: observation,
        }
    }

    fn sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(self.resize.width())
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(colors::SIDEBAR_BACKGROUND))
            .border_r_1()
            .border_color(rgb(colors::BORDER))
            .children(
                self.sections
                    .iter()
                    .enumerate()
                    .map(|(ix, section)| self.sidebar_row(section, ix, cx)),
            )
    }

    fn sidebar_row(&self, section: &Section, ix: usize, cx: &Context<Self>) -> impl IntoElement {
        let selected = ix == self.selected;
        div()
            .id(("section", ix))
            .px_2()
            .py_1()
            .cursor_pointer()
            .text_color(rgb(if selected {
                colors::PRIMARY_TEXT
            } else {
                colors::SECONDARY_TEXT
            }))
            .when(selected, |row| row.bg(rgb(colors::ROW_SELECTED_BACKGROUND)))
            .hover(|row| row.bg(rgb(colors::ROW_HOVER_BACKGROUND)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.selected = ix;
                let section = this.sections[ix].clone();
                let content = this.content.clone();
                content.update(cx, |pane, cx| pane.select(section, cx));
                cx.notify();
            }))
            .child(self.content.read(cx).sidebar_title(section))
    }

    fn divider(&self, cx: &Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div()
            .w(px(DIVIDER_WIDTH))
            .flex_shrink_0()
            .h_full()
            .cursor_col_resize()
            .bg(rgb(colors::BORDER))
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
}

impl Render for SplitPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(self.sidebar(cx))
            .child(self.divider(cx))
            .child(self.content.clone())
    }
}
