//! The command line layer: a thin clap router. No domain logic lives here;
//! it dispatches to `ui::app` for the gui and `feature::info` for docs.

pub mod info;
pub mod router;
