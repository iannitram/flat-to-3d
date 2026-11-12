//! Flat to 3D engine: installed games, recipes, stereo routes, the link service and launch wrapper.

pub mod api;
pub mod catalog;
pub mod cli;
pub mod exec;
pub mod fsx;
pub mod paths;
pub mod sys;
#[doc(hidden)]
pub mod testing;
pub use cli::link as cli_link;
pub use face::{collection, headset_view, link_view, rows, toasts, view};
pub mod face;
pub mod health;
pub mod injectors;
pub mod install;
pub mod launch_options;
pub mod link;

pub mod recipes;
pub mod runs;
pub mod setup;
pub mod state;
pub mod status;
pub mod steam;
pub mod switches;

pub mod undo;

pub mod watch;
pub mod wrapper;
pub use link::wivrn;

use std::fmt;

/// The one error a stub returns, so tests and callers can tell "not built yet" apart.
/// The string names the function and the issue that fills it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotBuilt(pub &'static str);

impl fmt::Display for NotBuilt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not built yet: {}", self.0)
    }
}

impl std::error::Error for NotBuilt {}
