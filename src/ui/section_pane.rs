//! The content pane: a grid of items produced by the selected section's
//! source.

pub mod chart;
pub mod item;

use crate::feature::section::source::SectionSource;
use crate::feature::section::store::{DisplayState, FetchDecision, SectionStore};
use crate::feature::section::{Section, SectionKey};
use crate::ui::colors;
use gpui::{
    AnyElement, Context, IntoElement, Render, SharedString, Window, div, prelude::*, rgb, rgba,
};
use std::sync::Arc;

const LOADING_MESSAGE: &str = "Loading…";
const EMPTY_MESSAGE: &str = "Nothing here right now.";
const REFRESHING_MESSAGE: &str = "Refreshing…";

/// How a fetch should treat the source's own caching.
#[derive(Clone, Copy)]
enum FetchKind {
    Load,
    Refresh,
}

/// The content pane: a grid of item tiles for the selected section.
pub struct SectionPane {
    store: SectionStore,
    selected: Option<Section>,
    source: Arc<dyn SectionSource>,
    no_sections_message: String,
}

impl SectionPane {
    /// Creates the pane and starts loading every section, so the sidebar
    /// gets its real titles and the cache is warm before a section is
    /// opened. `no_sections_message` is shown when `sections` is empty.
    pub fn new(
        sections: &[Section],
        source: Arc<dyn SectionSource>,
        no_sections_message: String,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut pane = Self {
            store: SectionStore::new(),
            selected: sections.first().cloned(),
            source,
            no_sections_message,
        };
        for section in sections {
            pane.load(&section.key, cx);
        }
        pane
    }

    /// Shows `section`. A local section runs its script when nothing fresh
    /// is cached; a remote one is refreshed on the server first.
    pub fn select(&mut self, section: Section, cx: &mut Context<Self>) {
        if self.source.refreshes_on_select() {
            self.refresh_key(&section.key, cx);
        } else {
            self.load(&section.key, cx);
        }
        self.selected = Some(section);
        cx.notify();
    }

    /// The sidebar title for `section`: the section's own title once it has
    /// loaded, its fallback title until then.
    pub fn sidebar_title(&self, section: &Section) -> String {
        self.store
            .title(&section.key)
            .map_or_else(|| section.fallback_title.clone(), str::to_string)
    }

    /// Recomputes the selected section even when its cache entry is fresh.
    /// A no-op with no selection or a fetch already in flight.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(section) = self.selected.clone() else {
            return;
        };
        self.refresh_key(&section.key, cx);
        cx.notify();
    }

    fn load(&mut self, key: &SectionKey, cx: &mut Context<Self>) {
        let decision = self.store.visit(key);
        self.act_on(decision, key.clone(), FetchKind::Load, cx);
    }

    fn refresh_key(&mut self, key: &SectionKey, cx: &mut Context<Self>) {
        let decision = self.store.force_refresh(key);
        self.act_on(decision, key.clone(), FetchKind::Refresh, cx);
    }

    fn act_on(
        &mut self,
        decision: FetchDecision,
        key: SectionKey,
        kind: FetchKind,
        cx: &mut Context<Self>,
    ) {
        match decision {
            FetchDecision::Idle => {}
            FetchDecision::Fetch | FetchDecision::Refresh => self.fetch(key, kind, cx),
        }
    }

    fn fetch(&mut self, key: SectionKey, kind: FetchKind, cx: &mut Context<Self>) {
        let source = self.source.clone();
        cx.spawn(async move |pane, cx| {
            let fetch_key = key.clone();
            let fetch = cx.background_spawn(async move {
                match kind {
                    FetchKind::Load => source.load(&fetch_key),
                    FetchKind::Refresh => source.refresh(&fetch_key),
                }
            });
            let result = fetch.await;
            pane.update(cx, |pane, cx| {
                pane.store.finish_fetch(&key, result);
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
            .is_some_and(|section| self.store.is_refreshing(&section.key));
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
            return Self::message(self.no_sections_message.clone()).into_any_element();
        };
        match self.store.display_state(&section.key) {
            DisplayState::Loading => Self::message(LOADING_MESSAGE).into_any_element(),
            DisplayState::Failed => {
                Self::message(self.failure_message(&section.key)).into_any_element()
            }
            DisplayState::Empty => Self::message(EMPTY_MESSAGE).into_any_element(),
            DisplayState::Populated => self.grid(&section.key).into_any_element(),
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

    fn failure_message(&self, key: &SectionKey) -> String {
        self.store
            .error(key)
            .map_or_else(|| EMPTY_MESSAGE.to_string(), ToString::to_string)
    }

    fn grid(&self, key: &SectionKey) -> impl IntoElement {
        let items = self
            .store
            .config(key)
            .map(|config| config.items.as_slice())
            .unwrap_or_default();
        div()
            .w_full()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_3()
            .p_3()
            .children(
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
            .bg(rgba(colors::CONTENT_BLUR_BACKGROUND))
            .child(self.header())
            .child(self.body())
    }
}
