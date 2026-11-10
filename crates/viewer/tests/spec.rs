use flat3d_viewer::spec::{Layout, Source, ViewerSpec};

#[test]
fn every_layout_and_source_round_trips_through_args() {
    for layout in [Layout::Sbs, Layout::Tab, Layout::SbsRev, Layout::TabRev] {
        for source in [
            Source::Auto,
            Source::Gamescope,
            Source::Portal,
            Source::TestPattern,
        ] {
            let spec = ViewerSpec {
                layout,
                source,
                swap_eyes: true,
                crop_aspect: Some(1.5),
                ..ViewerSpec::default()
            };
            assert_eq!(ViewerSpec::from_args(&spec.to_args()).unwrap(), spec);
        }
    }
    assert_eq!(
        ViewerSpec::from_args(&["--layout".into(), "tab_reversed".into()])
            .unwrap()
            .layout,
        Layout::TabRev
    );
    assert_eq!(
        ViewerSpec::from_args(&["--layout".into(), "sbs_reversed".into()])
            .unwrap()
            .layout,
        Layout::SbsRev
    );
    assert_eq!(
        ViewerSpec::from_args(&["--source".into(), "test_pattern".into()])
            .unwrap()
            .source,
        Source::TestPattern
    );
}

#[test]
fn defaults_omit_screen_dimensions_and_unknown_flag_is_an_error() {
    let spec = ViewerSpec::default();
    assert_eq!(spec.screen_width_m, 0.0);
    assert_eq!(spec.distance_m, 0.0);
    assert!(!spec
        .to_args()
        .iter()
        .any(|arg| arg == "--screen-width-m" || arg == "--distance-m"));
    assert!(ViewerSpec::from_args(&["--unknown".into()]).is_err());
}
