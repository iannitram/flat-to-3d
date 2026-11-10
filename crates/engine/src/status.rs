//! Stereo status wording, colours and order. docs/code-map.md, Parts.
use crate::api::Game;
use std::cmp::Ordering;

pub const STATUSES: [&str; 6] = [
    "works-now",
    "likely",
    "build",
    "not-safe",
    "wont-work",
    "untested",
];

pub fn status_label(s: &str) -> &'static str {
    match s {
        "works-now" => "Works now",
        "likely" => "Likely",
        "build" => "Needs a build",
        "not-safe" => "Not safe",
        "wont-work" => "Won't work",
        "untested" => "Untested",
        _ => "Untested",
    }
}

pub fn status_colour(s: &str) -> &'static str {
    match s {
        "works-now" => "#2e9e5b",
        "likely" => "#8bc34a",
        "build" => "#e0a030",
        "not-safe" => "#d6453d",
        "wont-work" => "#6b6b6b",
        "untested" => "#b0b0b0",
        _ => "#b0b0b0",
    }
}

pub fn compare_games(a: &Game, b: &Game) -> Ordering {
    // First compare by status order
    let a_status_index = STATUSES
        .iter()
        .position(|&x| x == a.status)
        .unwrap_or(STATUSES.len());
    let b_status_index = STATUSES
        .iter()
        .position(|&x| x == b.status)
        .unwrap_or(STATUSES.len());

    match a_status_index.cmp(&b_status_index) {
        Ordering::Equal => {
            // If statuses are equal, compare by name (case insensitive)
            let a_name_lower = a.name.to_lowercase();
            let b_name_lower = b.name.to_lowercase();

            match a_name_lower.cmp(&b_name_lower) {
                Ordering::Equal => {
                    // If names are equal, compare by appid
                    a.appid.cmp(&b.appid)
                }
                other => other,
            }
        }
        other => other,
    }
}

pub fn sort_games(games: &[Game]) -> Vec<Game> {
    let mut sorted = games.to_vec();
    sorted.sort_by(compare_games);
    sorted
}

pub fn filter_by_status(games: &[Game], status: &str) -> Vec<Game> {
    if status == "all" {
        games.to_vec()
    } else {
        games
            .iter()
            .filter(|game| game.status == status)
            .cloned()
            .collect()
    }
}
