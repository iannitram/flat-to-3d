//! Locked tests for E10a: how both faces show a game's stereo status, and the game list order.

use flat3d_inject::api::Game;
use flat3d_inject::status::{compare_games, filter_by_status, sort_games, status_colour, status_label, STATUSES};
use std::cmp::Ordering;

fn g(appid: u32, name: &str, status: &str) -> Game {
    Game { appid, name: name.into(), install_dir: format!("/g/{appid}"), status: status.into(), has_recipe: status != "untested" }
}
fn ids(games: &[Game]) -> Vec<u32> {
    games.iter().map(|x| x.appid).collect()
}

#[test]
fn the_six_labels() {
    let got: Vec<&str> = STATUSES.iter().map(|s| status_label(s)).collect();
    assert_eq!(got, vec!["Works now", "Likely", "Needs a build", "Not safe", "Won't work", "Untested"]);
    assert_eq!(status_label("???"), "Untested");
}

#[test]
fn the_six_colours_are_distinct() {
    let got: Vec<&str> = STATUSES.iter().map(|s| status_colour(s)).collect();
    assert_eq!(got, vec!["#2e9e5b", "#8bc34a", "#e0a030", "#d6453d", "#6b6b6b", "#b0b0b0"]);
    assert_eq!(status_colour("???"), "#b0b0b0");
}

#[test]
fn better_statuses_come_first() {
    assert_eq!(compare_games(&g(1, "Z", "works-now"), &g(2, "A", "likely")), Ordering::Less);
    assert_eq!(compare_games(&g(1, "A", "untested"), &g(2, "Z", "build")), Ordering::Greater);
    assert_eq!(compare_games(&g(1, "A", "???"), &g(2, "A", "untested")), Ordering::Greater);
}

#[test]
fn then_name_ignoring_case_then_appid() {
    assert_eq!(compare_games(&g(1, "apple", "likely"), &g(2, "Banana", "likely")), Ordering::Less);
    assert_eq!(compare_games(&g(1, "Same", "likely"), &g(2, "same", "likely")), Ordering::Less);
    assert_eq!(compare_games(&g(2, "Same", "likely"), &g(1, "same", "likely")), Ordering::Greater);
    assert_eq!(compare_games(&g(1, "Same", "likely"), &g(1, "Same", "likely")), Ordering::Equal);
}

#[test]
fn sort_returns_a_new_ordered_list() {
    let games = vec![
        g(5, "zeta", "untested"), g(4, "Beta", "works-now"), g(3, "alpha", "works-now"),
        g(2, "Control", "build"), g(1, "Ace", "likely"),
    ];
    assert_eq!(ids(&sort_games(&games)), vec![3, 4, 1, 2, 5]);
    assert_eq!(ids(&games), vec![5, 4, 3, 2, 1]);
}

#[test]
fn filter_keeps_one_status_or_all() {
    let games = vec![g(1, "A", "likely"), g(2, "B", "works-now"), g(3, "C", "likely")];
    assert_eq!(ids(&filter_by_status(&games, "likely")), vec![1, 3]);
    assert_eq!(ids(&filter_by_status(&games, "all")), vec![1, 2, 3]);
    assert!(filter_by_status(&games, "build").is_empty());
}
