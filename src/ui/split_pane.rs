//! The root view: a navigation sidebar next to a content pane, with a
//! draggable divider between them.

pub mod resize;

use crate::feature::section::Section;
use crate::feature::section::source::SectionSource;
use crate::ui::colors;
use crate::ui::section_pane::SectionPane;
use crate::ui::split_pane::resize::ResizeState;
use gpui::{
    App, Context, Entity, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render,
    Subscription, Window, canvas, div, prelude::*, px, rgb, rgba,
};
use std::sync::Arc;

const DIVIDER_WIDTH: f32 = 8.0;

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
    pub fn new(
        sections: Vec<Section>,
        source: Arc<dyn SectionSource>,
        no_sections_message: String,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = cx.new(|cx| SectionPane::new(&sections, source, no_sections_message, cx));
        let observation = cx.observe(&content, |_, _, cx| cx.notify());
        Self {
            resize: ResizeState::new(),
            sections,
            selected: 0,
            content,
            _content_observation: observation,
        }
    }

    /// Re-runs the selected section's script, forwarding to the content pane.
    pub fn refresh_selected(&mut self, cx: &mut Context<Self>) {
        self.content.update(cx, |pane, cx| pane.refresh(cx));
    }

    /// Selects the tab `offset` places from the current one, wrapping at the
    /// ends. A no-op with no sections.
    pub fn select_adjacent(&mut self, offset: isize, cx: &mut Context<Self>) {
        if let Some(ix) = adjacent_index(self.selected, self.sections.len(), offset) {
            self.select(ix, cx);
        }
    }

    fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = ix;
        let section = self.sections[ix].clone();
        self.content.update(cx, |pane, cx| pane.select(section, cx));
        cx.notify();
    }

    fn sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(self.resize.width())
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .px_2()
            .pt_2()
            .gap_0p5()
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
            .rounded_lg()
            .cursor_pointer()
            .text_color(rgb(if selected {
                colors::PRIMARY_TEXT
            } else {
                colors::SECONDARY_TEXT
            }))
            .when(selected, |row| {
                row.bg(rgba(colors::SIDEBAR_ROW_SELECTED_BACKGROUND))
            })
            .when(!selected, |row| {
                row.hover(|row| row.bg(rgba(colors::SIDEBAR_ROW_HOVER_BACKGROUND)))
            })
            .on_click(cx.listener(move |this, _event, _window, cx| this.select(ix, cx)))
            .child(self.content.read(cx).sidebar_title(section))
    }

    fn divider(&self, cx: &Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div()
            .w(px(DIVIDER_WIDTH))
            .flex_shrink_0()
            .h_full()
            .cursor_col_resize()
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

/// The index `offset` places from `current` among `len` tabs, wrapping at the
/// ends; `None` when there are no tabs.
fn adjacent_index(current: usize, len: usize, offset: isize) -> Option<usize> {
    let len = isize::try_from(len).ok().filter(|len| *len > 0)?;
    let current = isize::try_from(current).ok()?;
    usize::try_from((current + offset).rem_euclid(len)).ok()
}

impl Render for SplitPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgba(colors::SIDEBAR_BLUR_BACKGROUND))
            .child(self.sidebar(cx))
            .child(self.divider(cx))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .pt_2()
                    .pr_2()
                    .pb_2()
                    .child(self.content.clone()),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_index_moves_forward_and_back() {
        assert_eq!(adjacent_index(1, 4, 1), Some(2));
        assert_eq!(adjacent_index(1, 4, -1), Some(0));
    }

    #[test]
    fn adjacent_index_wraps_at_both_ends() {
        assert_eq!(adjacent_index(3, 4, 1), Some(0));
        assert_eq!(adjacent_index(0, 4, -1), Some(3));
    }

    #[test]
    fn adjacent_index_is_none_without_tabs() {
        assert_eq!(adjacent_index(0, 0, 1), None);
    }
}
