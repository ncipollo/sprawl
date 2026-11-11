//! State shared by every request handler.

use crate::feature::server::service::SectionService;

/// What handlers need: the section service.
#[derive(Clone)]
pub struct AppState {
    pub service: SectionService,
}

impl AppState {
    pub fn new(service: SectionService) -> Self {
        Self { service }
    }
}
