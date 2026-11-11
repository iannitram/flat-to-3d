//! Locked tests for E9a: the 3D Ready set and the collection diff. Pure.

use flat3d_inject::api::Game;
use flat3d_inject::collection::{
    collection_diff, three_d_ready_appids, CollectionDiff, COLLECTION_NAME,
};

fn g(appid: u32, status: &str, has_recipe: bool) -> Game {
    Game {
        appid,
        name: format!("G{appid}"),
        install_dir: format!("/g/{appid}"),
        status: status.into(),
        has_recipe,
    }
}
fn d(add: &[u32], remove: &[u32]) -> CollectionDiff {
    CollectionDiff {
        add: add.to_vec(),
        remove: remove.to_vec(),
    }
}

#[test]
fn only_works_now_games_with_a_recipe_count_sorted() {
    let games = vec![
        g(30, "works-now", true),
        g(10, "works-now", true),
        g(20, "likely", true),
        g(40, "untested", false),
        g(50, "not-safe", true),
        g(60, "build", true),
        g(70, "works-now", false),
    ];
    assert_eq!(three_d_ready_appids(&games), vec![10, 30]);
}

#[test]
fn no_repeats_and_an_empty_list() {
    assert_eq!(
        three_d_ready_appids(&[g(5, "works-now", true), g(5, "works-now", true)]),
        vec![5]
    );
    assert!(three_d_ready_appids(&[]).is_empty());
}

#[test]
fn the_diff_adds_what_is_missing_and_removes_what_no_longer_belongs() {
    assert_eq!(collection_diff(&[1, 2, 3], &[3, 4, 2, 5]), d(&[4, 5], &[1]));
}

#[test]
fn the_diff_is_empty_when_they_match_in_any_order() {
    assert_eq!(collection_diff(&[2, 1], &[1, 2]), d(&[], &[]));
}

#[test]
fn the_diff_from_or_to_an_empty_collection() {
    assert_eq!(collection_diff(&[], &[9, 3, 9]), d(&[3, 9], &[]));
    assert_eq!(collection_diff(&[7, 7, 1], &[]), d(&[], &[1, 7]));
}

#[test]
fn the_name() {
    assert_eq!(COLLECTION_NAME, "3D Ready");
}
