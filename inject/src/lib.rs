//! Flat to 3D's Inject engine (SPEC.md M9-M12): find installed games, read their recipes,
//! inject the recipe's stereo route, and bring up the headset link. Rust, by the owner's
//! 2026-10-26 ruling.
//!
//! Skeleton (2026-10-26): every function is a stub with its final signature that returns
//! `Err(NotBuilt)`; each one is filled in by its own issue. GPL-3.0.

pub mod api;
pub mod cli;
pub mod cli_link;
pub mod injectors;
pub mod launch_options;
pub mod link;
pub mod recipes;
pub mod runs;
pub mod setup;
pub mod state;
pub mod steam;
pub mod switches;
pub mod undo;
pub mod watch;

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
