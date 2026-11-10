//! The 3D Ready set. docs/code-map.md, Parts.

use std::collections::BTreeSet;

use crate::api::Game;

/// The name of the Steam collection that holds the 3D Ready games.
pub const COLLECTION_NAME: &str = "3D Ready";

/// What to add to and remove from the collection to take it from `current` to `wanted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionDiff {
    /// Wanted but not current, ascending, no repeats.
    pub add: Vec<u32>,
    /// Current but not wanted, ascending, no repeats.
    pub remove: Vec<u32>,
}

/// The appids that belong in the 3D Ready collection: games whose status is `works-now`
/// and that have a recipe. Ascending, no repeats.
pub fn three_d_ready_appids(games: &[Game]) -> Vec<u32> {
    let set: BTreeSet<u32> = games
        .iter()
        .filter(|g| g.status == "works-now" && g.has_recipe)
        .map(|g| g.appid)
        .collect();
    set.into_iter().collect()
}

/// The diff from the `current` collection to the `wanted` one, both ways.
pub fn collection_diff(current: &[u32], wanted: &[u32]) -> CollectionDiff {
    let have: BTreeSet<u32> = current.iter().copied().collect();
    let want: BTreeSet<u32> = wanted.iter().copied().collect();
    CollectionDiff {
        add: want.difference(&have).copied().collect(),
        remove: have.difference(&want).copied().collect(),
    }
}
