//! The content pane: a grid of items produced by the selected section's
//! script.

pub mod item;

use crate::feature::script;
use crate::feature::script::shell::SystemShell;
use crate::feature::section::Section;
use crate::feature::section::store::{DisplayState, FetchDecision, SectionStore};
use crate::ui::colors;
use gpui::{AnyElement, Context, IntoElement, Render, SharedString, Window, div, prelude::*, rgb};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const LOADING_MESSAGE: &str = "Loading…";
const EMPTY_MESSAGE: &str = "Nothing here right now.";
const REFRESHING_MESSAGE: &str = "Refreshing…";
const NO_SECTIONS_MESSAGE: &str = "No config scripts found in ~/.sprawl/default.";

/// The content pane: a grid of item tiles for the selected section.
pub struct SectionPane {
    store: SectionStore,
    selected: Option<Section>,
}

impl SectionPane {
    /// Creates the pane and starts running every section's script, so the
    /// sidebar gets its script-provided titles and the cache is warm before
    /// a section is opened.
    pub fn new(sections: &[Section], cx: &mut Context<Self>) -> Self {
        let mut pane = Self {
            store: SectionStore::new(),
            selected: sections.first().cloned(),
        };
        for section in sections {
            pane.load(&section.path, cx);
        }
        pane
    }

    /// Shows `section`, running its script when nothing fresh is cached.
    pub fn select(&mut self, section: Section, cx: &mut Context<Self>) {
        self.load(&section.path, cx);
        self.selected = Some(section);
        cx.notify();
    }

    /// The sidebar title for `section`: the script's own title once it has
    /// run, its file-name-derived title until then.
    pub fn sidebar_title(&self, section: &Section) -> String {
        self.store
            .title(&section.path)
            .map_or_else(|| section.fallback_title.clone(), str::to_string)
    }

    fn load(&mut self, path: &Path, cx: &mut Context<Self>) {
        match self.store.visit(path) {
            FetchDecision::Idle => {}
            FetchDecision::Fetch | FetchDecision::Refresh => self.fetch(path.to_path_buf(), cx),
        }
    }

    fn fetch(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        cx.spawn(async move |pane, cx| {
            let script_path = path.clone();
            let fetch = cx.background_spawn(async move {
                script::run_file(&script_path, Arc::new(SystemShell))
            });
            let result = fetch.await;
            pane.update(cx, |pane, cx| {
                pane.store.finish_fetch(&path, result);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn header(&self) -> impl IntoElement {
        let title = self
            .selected
            .as_ref()
            .map(|section| self.sidebar_title(section))
            .unwrap_or_default();
        let refreshing = self
            .selected
            .as_ref()
            .is_some_and(|section| self.store.is_refreshing(&section.path));
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .pt_3()
            .child(
                div()
                    .text_lg()
                    .text_color(rgb(colors::PRIMARY_TEXT))
                    .child(title),
            )
            .when(refreshing, |header| {
                header.child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors::SECONDARY_TEXT))
                        .child(REFRESHING_MESSAGE),
                )
            })
    }

    fn body(&self) -> AnyElement {
        let Some(section) = &self.selected else {
            return Self::message(NO_SECTIONS_MESSAGE).into_any_element();
        };
        match self.store.display_state(&section.path) {
            DisplayState::Loading => Self::message(LOADING_MESSAGE).into_any_element(),
            DisplayState::Failed => {
                Self::message(self.failure_message(&section.path)).into_any_element()
            }
            DisplayState::Empty => Self::message(EMPTY_MESSAGE).into_any_element(),
            DisplayState::Populated => self.grid(&section.path).into_any_element(),
        }
    }

    fn message(text: impl Into<SharedString>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .justify_center()
            .items_center()
            .p_3()
            .text_color(rgb(colors::SECONDARY_TEXT))
            .child(text.into())
    }

    fn failure_message(&self, path: &Path) -> String {
        self.store
            .error(path)
            .map_or_else(|| EMPTY_MESSAGE.to_string(), ToString::to_string)
    }

    fn grid(&self, path: &Path) -> impl IntoElement {
        let items = self
            .store
            .config(path)
            .map(|config| config.items.as_slice())
            .unwrap_or_default();
        div().flex().flex_row().flex_wrap().gap_3().p_3().children(
            items
                .iter()
                .enumerate()
                .map(|(index, entry)| item::render_item(index, entry)),
        )
    }
}

impl Render for SectionPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("section-pane")
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .bg(rgb(colors::CONTENT_BACKGROUND))
            .child(self.header())
            .child(self.body())
    }
}
