//! The content pane: a grid of pull request tiles for the selected section.

pub mod badge;

use crate::feature::github::client::{GhCli, fetch_pull_requests};
use crate::feature::github::pull_request::PullRequest;
use crate::feature::github::query::PullRequestQuery;
use crate::feature::github::store::{DisplayState, FetchDecision, PullRequestStore};
use crate::feature::section::Section;
use crate::ui::colors;
use crate::ui::components::tile::Tile;
use gpui::{AnyElement, Context, IntoElement, Render, SharedString, Window, div, prelude::*, rgb};

const LOADING_MESSAGE: &str = "Loading pull requests…";
const EMPTY_MESSAGE: &str = "No pull requests right now.";
const REFRESHING_MESSAGE: &str = "Refreshing…";
const PLACEHOLDER_MESSAGE: &str = "Coming soon.";

/// The content pane: a grid of pull request tiles for the selected section.
pub struct PullRequestPane {
    store: PullRequestStore,
    selected: Section,
}

impl PullRequestPane {
    pub fn new(selected: Section, cx: &mut Context<Self>) -> Self {
        let mut pane = Self {
            store: PullRequestStore::new(),
            selected,
        };
        pane.load_selection(cx);
        pane
    }

    /// Shows `section`, fetching its pull requests when nothing fresh is
    /// cached.
    pub fn select(&mut self, section: Section, cx: &mut Context<Self>) {
        self.selected = section;
        self.load_selection(cx);
        cx.notify();
    }

    fn load_selection(&mut self, cx: &mut Context<Self>) {
        let Some(query) = self.selected.pull_request_query() else {
            return;
        };
        match self.store.visit(query) {
            FetchDecision::Idle => {}
            FetchDecision::Fetch(query) | FetchDecision::Refresh(query) => self.fetch(query, cx),
        }
    }

    fn fetch(&mut self, query: PullRequestQuery, cx: &mut Context<Self>) {
        cx.spawn(async move |pane, cx| {
            let fetch = cx.background_spawn(async move { fetch_pull_requests(&GhCli, query) });
            let result = fetch.await;
            pane.update(cx, |pane, cx| {
                pane.store.finish_fetch(query, result);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn header(&self) -> impl IntoElement {
        let refreshing = self
            .selected
            .pull_request_query()
            .is_some_and(|query| self.store.is_refreshing(query));
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
                    .child(self.selected.title()),
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
        let Some(query) = self.selected.pull_request_query() else {
            return Self::message(PLACEHOLDER_MESSAGE).into_any_element();
        };
        match self.store.display_state(query) {
            DisplayState::Loading => Self::message(LOADING_MESSAGE).into_any_element(),
            DisplayState::Failed => Self::message(self.failure_message(query)).into_any_element(),
            DisplayState::Empty => Self::message(EMPTY_MESSAGE).into_any_element(),
            DisplayState::Populated => self.grid(query).into_any_element(),
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

    fn failure_message(&self, query: PullRequestQuery) -> String {
        self.store
            .error(query)
            .map_or_else(|| EMPTY_MESSAGE.to_string(), ToString::to_string)
    }

    fn grid(&self, query: PullRequestQuery) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_3()
            .p_3()
            .children(self.store.pull_requests(query).iter().map(Self::tile))
    }

    fn tile(pull_request: &PullRequest) -> Tile {
        let url = pull_request.url.clone();
        Tile::new(
            SharedString::from(pull_request.url.clone()),
            pull_request.title.clone(),
        )
        .subtitle(format!(
            "{} #{}",
            pull_request.repository, pull_request.number
        ))
        .badge(badge::comment_badge(pull_request.comment_count))
        .badge(badge::review_badge(pull_request.review))
        .badge(badge::checks_badge(pull_request.checks))
        .when(pull_request.is_draft, |tile| {
            tile.badge(badge::draft_badge())
        })
        .on_click(move |_event, _window, cx| cx.open_url(&url))
    }
}

impl Render for PullRequestPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("pull-request-pane")
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
