use flat3d_inject::recipes::{check_route_settings, route_settings, RouteSettings};

fn one(key: &str, value: &str) -> RouteSettings {
    [(key.to_string(), value.to_string())].into_iter().collect()
}

#[test]
fn uevr_value_rules_accept_valid_values_and_refuse_bad_values() {
    for (key, good, bad) in [
        ("uevr-exe", "Game.exe", "   "),
        ("uevr-rendering", "native", "fast"),
        ("uevr-2d-screen", "yes", "true"),
        ("uevr-native-stereo-fix", "no", "maybe"),
        ("uevr-delay", "120", "121"),
        ("uevr-profile", "ac7-profile", "Bad_Key"),
    ] {
        let mut valid = one("uevr-exe", "Game.exe");
        valid.insert(key.into(), good.into());
        assert!(check_route_settings("uevr", &valid).is_ok(), "{key}");
        let mut invalid = one("uevr-exe", "Game.exe");
        invalid.insert(key.into(), bad.into());
        assert!(check_route_settings("uevr", &invalid).is_err(), "{key}");
    }
    assert!(check_route_settings("uevr", &RouteSettings::new()).is_err());
}

#[test]
fn geo11_value_rules_accept_valid_values_and_refuse_bad_values() {
    for (key, good, bad) in [
        ("geo11-fix", "geo11", "Geo11"),
        ("geo11-exe", "Game.exe", ""),
        ("geo11-layout", "sbs_reversed", "reverse"),
        ("geo11-separation", "100", "101"),
        ("geo11-convergence", "0.25", "0"),
        ("geo11-d3dx", "d3d11.dll=geo11.dll", "broken"),
        (
            "geo11-remove-first",
            "dxgi.dll;bin/d3d11.dll",
            "../outside.dll",
        ),
    ] {
        let mut valid = one("geo11-fix", "geo11");
        valid.insert("geo11-exe".into(), "Game.exe".into());
        valid.insert(key.into(), good.into());
        assert!(check_route_settings("geo-11", &valid).is_ok(), "{key}");
        let mut invalid = one("geo11-fix", "geo11");
        invalid.insert("geo11-exe".into(), "Game.exe".into());
        invalid.insert(key.into(), bad.into());
        assert!(check_route_settings("geo-11", &invalid).is_err(), "{key}");
    }
    assert!(check_route_settings("geo-11", &one("geo11-fix", "geo11")).is_err());
}

#[test]
fn unknown_keys_and_port_settings_are_refused() {
    let mut unknown = one("uevr-exe", "Game.exe");
    unknown.insert("uevr-x".into(), "x".into());
    assert!(check_route_settings("uevr", &unknown).is_err());
    assert!(check_route_settings("build: port stereo screen", &one("port-x", "x")).is_err());
    assert!(check_route_settings("build: port stereo screen", &RouteSettings::new()).is_ok());
}

#[test]
fn route_settings_drops_recipe_keys_and_comments_and_strips_quotes() {
    let settings = route_settings(
        "# comment\nname: Game\nstereo-route: uevr\nuevr-exe: \"Game.exe\"\nuevr-profile: \"profile-one\"\n",
    );
    assert_eq!(settings.len(), 2);
    assert_eq!(settings["uevr-exe"], "Game.exe");
    assert_eq!(settings["uevr-profile"], "profile-one");
    assert!(!settings.contains_key("name"));
}
