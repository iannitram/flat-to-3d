use flat3d_inject::api::ERROR_CODES;
use flat3d_inject::injectors::geo11::{self, FileStep, Geo11Settings};
use flat3d_inject::recipes::RouteSettings;
use flat3d_inject::undo::{record_path, undo_game};
use flat3d_viewer::spec::{Layout, ViewerSpec};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn root(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("geo11")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn put(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut entries = entries.map(Result::unwrap).collect::<Vec<_>>();
            entries.sort_by_key(|e| e.path());
            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    walk(root, &path, out);
                } else if path.is_file() {
                    out.push((
                        path.strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/"),
                        fs::read(path).unwrap(),
                    ));
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn settings(items: &[(&str, &str)]) -> RouteSettings {
    items
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect::<BTreeMap<_, _>>()
}

fn geo11_overlay(cache: &Path) {
    let version = cache.join("geo11/geo-11_v0.7.12");
    for arch in ["x64", "x32"] {
        put(version.join(arch).join("d3d11.dll"), arch);
        put(version.join(arch).join("nvapi64.dll"), arch);
        put(
            version.join(arch).join("d3dxdm.ini"),
            "direct_mode=sbs\ndm_separation=30\ndm_convergence=1.0\n",
        );
    }
}

fn control(cache: &Path, game: &Path) -> RouteSettings {
    put(
        cache.join("control-fix/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n[Device]\n;get_resolution_from = depth_buffer\n",
    );
    put(cache.join("control-fix/d3d11.dll"), "fix");
    put(cache.join("control-fix/ShaderFixes/a-ps.txt"), "shader");
    put(cache.join("control-fix/uninstall.bat"), "ignore");
    geo11_overlay(cache);
    put(game.join("Control_DX11.exe"), "exe");
    put(game.join("dxgi.dll"), "old dxgi");
    settings(&[
        ("geo11-fix", "control-fix"),
        ("geo11-exe", "Control_DX11.exe"),
        ("geo11-d3dx", "get_resolution_from=swap_chain"),
        ("geo11-remove-first", "dxgi.dll"),
    ])
}

#[test]
fn d3dxdm_and_layout_formats_are_exact() {
    let d = Geo11Settings::default();
    assert_eq!(
        geo11::d3dxdm_text(&d),
        "direct_mode=sbs\ndm_separation=40\ndm_convergence=2.0\n"
    );
    for (layout, name) in [
        (Layout::Sbs, "sbs"),
        (Layout::Tab, "tab"),
        (Layout::SbsRev, "sbs_reversed"),
        (Layout::TabRev, "tab_reversed"),
    ] {
        assert_eq!(geo11::direct_mode(layout), name);
    }
    let mut custom = d;
    custom.convergence = 2.25;
    assert_eq!(
        geo11::d3dxdm_text(&custom),
        "direct_mode=sbs\ndm_separation=40\ndm_convergence=2.25\n"
    );
}

#[test]
fn route_defaults_and_invalid_values() {
    assert_eq!(
        Geo11Settings::from_route(&RouteSettings::new()).unwrap(),
        Geo11Settings::default()
    );
    for (key, value) in [
        ("geo11-layout", "interlaced"),
        ("geo11-separation", "101"),
        ("geo11-convergence", "0"),
        ("geo11-convergence", "inf"),
        ("geo11-d3dx", "nokey"),
        ("geo11-remove-first", "../x"),
    ] {
        assert!(Geo11Settings::from_route(&settings(&[(key, value)]))
            .unwrap_err()
            .starts_with("bad-recipe: "));
    }
}

#[test]
fn d3dx_patch_replaces_inserts_appends_crlf_and_is_idempotent() {
    let d = Geo11Settings::default();
    assert_eq!(
        geo11::d3dx_patch("[Stereo]\nforce_stereo = 0\n", &d).unwrap(),
        "[Stereo]\nforce_stereo = 2\n"
    );
    assert_eq!(
        geo11::d3dx_patch(";force_stereo = 0\n", &d).unwrap(),
        ";force_stereo = 0\nforce_stereo=2\n"
    );
    assert_eq!(
        geo11::d3dx_patch("[Other]\nvalue=1\n", &d).unwrap(),
        "[Other]\nvalue=1\n\n[Stereo]\nforce_stereo=2\n"
    );
    let crlf = geo11::d3dx_patch("[Stereo]\r\nforce_stereo = 0\r\n", &d).unwrap();
    assert!(crlf.contains("\r\n") && !crlf.replace("\r\n", "").contains('\n'));
    assert_eq!(
        geo11::d3dx_patch(
            &geo11::d3dx_patch("[Stereo]\nforce_stereo=0\n", &d).unwrap(),
            &d
        )
        .unwrap(),
        "[Stereo]\nforce_stereo=2\n"
    );
    assert!(geo11::d3dx_patch(
        "[Other]\nx=1\n",
        &Geo11Settings {
            d3dx: vec![("unknown".into(), "1".into())],
            ..d
        }
    )
    .unwrap_err()
    .starts_with("bad-recipe: "));
}

#[test]
fn control_plan_selects_x64_and_orders_remove_copy_write() {
    let r = root("control-plan");
    let cache = r.join("cache");
    let game = r.join("game");
    let m = control(&cache, &game);
    let plan = geo11::plan_files(&cache, &game, &m).unwrap();
    assert!(
        matches!(plan.first(), Some(FileStep::Remove(p)) if p.file_name().unwrap() == "dxgi.dll")
    );
    assert!(!plan.iter().any(
        |s| matches!(s, FileStep::Copy { to, .. } if to.file_name().unwrap() == "uninstall.bat")
    ));
    assert!(plan.iter().any(|s| matches!(s, FileStep::Copy { from, to } if to.file_name().unwrap() == "d3d11.dll" && from.to_string_lossy().contains("x64"))));
    assert!(plan.iter().any(|s| matches!(s, FileStep::Write { to, text } if to.file_name().unwrap() == "d3dx.ini" && text.contains("force_stereo = 2") && text.contains("get_resolution_from=swap_chain"))));
}

#[test]
fn fix_with_d3dxdm_does_not_require_overlay() {
    let r = root("lana");
    let cache = r.join("cache");
    let game = r.join("game");
    put(
        cache.join("lana-fix/Planet of Lana/d3dx.ini"),
        "[Stereo]\nforce_stereo=0\n",
    );
    put(cache.join("lana-fix/Planet of Lana/d3d11.dll"), "fix dll");
    put(
        cache.join("lana-fix/Planet of Lana/d3dxdm.ini"),
        "direct_mode=tab\ndm_separation=8\ndm_convergence=1.0\n",
    );
    put(game.join("Lana.exe"), "exe");
    let m = settings(&[
        ("geo11-fix", "lana-fix"),
        ("geo11-exe", "Lana.exe"),
        ("geo11-layout", "sbs"),
    ]);
    let plan = geo11::plan_files(&cache, &game, &m).unwrap();
    assert!(!plan.iter().any(
        |s| matches!(s, FileStep::Copy { from, .. } if from.starts_with(cache.join("geo11")))
    ));
    assert!(plan.iter().any(|s| matches!(s, FileStep::Write { to, text } if to.file_name().unwrap() == "d3dxdm.ini" && text.contains("direct_mode=sbs"))));
}

#[test]
fn error_cases_have_closed_codes_and_specific_fetch_hints() {
    let r = root("errors");
    let cache = r.join("cache");
    let game = r.join("game");
    let exe = game.join("Game.exe");
    put(&exe, "exe");
    geo11_overlay(&cache);
    let cases: Vec<(RouteSettings, &str, Option<&str>)> = vec![
        (
            settings(&[("geo11-exe", "Game.exe"), ("geo11-layout", "interlaced")]),
            "bad-recipe",
            None,
        ),
        (
            settings(&[("geo11-exe", "Game.exe"), ("geo11-d3dx", "nokey")]),
            "bad-recipe",
            None,
        ),
        (settings(&[("geo11-exe", "No.exe")]), "not-found", None),
        (
            settings(&[("geo11-exe", "Game.exe"), ("geo11-fix", "control-fix")]),
            "not-found",
            Some("setup fetch control-fix"),
        ),
    ];
    for (m, code, text) in cases {
        let err = geo11::plan_files(&cache, &game, &m).unwrap_err();
        assert!(err.starts_with(&format!("{code}: ")), "{err}");
        assert!(ERROR_CODES.contains(&code));
        if let Some(text) = text {
            assert!(err.contains(text), "{err}");
        }
    }
    let unknown = geo11::d3dx_patch(
        "[Stereo]\nforce_stereo=0\n",
        &Geo11Settings {
            d3dx: vec![("unknown".into(), "1".into())],
            ..Geo11Settings::default()
        },
    )
    .unwrap_err();
    assert!(unknown.starts_with("bad-recipe: "));
    assert!(ERROR_CODES.contains(&unknown.split_once(':').unwrap().0));
    let empty_cache = r.join("empty-cache");
    let err = geo11::plan_files(&empty_cache, &game, &settings(&[("geo11-exe", "Game.exe")]))
        .unwrap_err();
    assert!(err.starts_with("not-found: ") && err.contains("setup fetch geo11"));
}

#[test]
fn viewer_check_requires_file_and_unix_execute_bit() {
    let r = root("viewer-check");
    let missing = r.join("nope");
    assert_eq!(geo11::viewer_check(&missing).unwrap_err().code, "not-found");
    assert!(geo11::viewer_check(&missing)
        .unwrap_err()
        .message
        .contains("flat3d-viewer not found"));
    let file = r.join("viewer");
    put(&file, "");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(geo11::viewer_check(&file).unwrap_err().code, "not-found");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(geo11::viewer_check(&file).is_ok());
    }
    #[cfg(not(unix))]
    assert!(geo11::viewer_check(&file).is_ok());
}

#[test]
fn apply_undo_restores_snapshot_and_sidecar() {
    let r = root("apply-undo");
    let cache = r.join("cache");
    let game = r.join("game");
    let m = control(&cache, &game);
    let viewer = r.join("flat3d-viewer");
    put(&viewer, "");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&viewer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let before = snapshot(&r);
    let (applied, env) = geo11::apply(&cache, &game, 123, &m, &viewer).unwrap();
    assert_eq!(applied.record.launch_options_before, "");
    let data = r.join("data");
    let record_path = record_path(&data, 123);
    fs::create_dir_all(record_path.parent().unwrap()).unwrap();
    fs::write(&record_path, applied.record.to_text()).unwrap();
    undo_game(&data, 123).unwrap();
    assert_eq!(snapshot(&r), before);
    assert_eq!(fs::read(game.join("dxgi.dll")).unwrap(), b"old dxgi");
    assert!(game.join("ShaderFixes").is_dir());
    assert_eq!(fs::read_dir(game.join("ShaderFixes")).unwrap().count(), 0);
    assert_eq!(
        env.env,
        [("WINEDLLOVERRIDES".into(), "d3d11,nvapi64=n,b".into())]
    );
    assert!(env.unset.is_empty());
    let mut expected = vec![viewer.to_string_lossy().into_owned()];
    expected.extend(
        ViewerSpec {
            layout: Layout::Sbs,
            ..ViewerSpec::default()
        }
        .to_args(),
    );
    assert_eq!(env.sidecar, Some(expected));
}

#[test]
fn second_apply_refuses_and_dxgi_overlay_sets_override() {
    let r = root("second");
    let cache = r.join("cache");
    let game = r.join("game");
    let m = control(&cache, &game);
    let viewer = r.join("viewer");
    put(&viewer, "");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&viewer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let (first, _) = geo11::apply(&cache, &game, 5, &m, &viewer).unwrap();
    let after = snapshot(&r);
    let err = geo11::apply(&cache, &game, 5, &m, &viewer).unwrap_err();
    assert!(err.starts_with("refused: ") && err.contains("flat3d-inject undo 5"));
    assert_eq!(snapshot(&r), after);
    let data = r.join("data");
    let record = record_path(&data, 5);
    fs::create_dir_all(record.parent().unwrap()).unwrap();
    fs::write(record, first.record.to_text()).unwrap();
    undo_game(&data, 5).unwrap();
}

#[test]
fn rollback_after_backup_failure_restores_snapshot() {
    let r = root("rollback");
    let cache = r.join("cache");
    let game = r.join("game");
    let m = control(&cache, &game);
    fs::remove_file(game.join("d3d11.dll")).ok();
    fs::create_dir_all(game.join("d3d11.dll")).unwrap();
    let viewer = r.join("viewer");
    put(&viewer, "");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&viewer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let before = snapshot(&r);
    let err = geo11::apply(&cache, &game, 6, &m, &viewer).unwrap_err();
    assert!(err.starts_with("io: "));
    assert_eq!(snapshot(&r), before);
}

fn viewer_exe(r: &Path) -> PathBuf {
    let viewer = r.join("viewer");
    put(&viewer, "");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&viewer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    viewer
}

#[test]
fn nested_remove_and_copy_do_not_touch_top_level_namesakes() {
    let r = root("nested");
    let cache = r.join("cache");
    let game = r.join("game");
    geo11_overlay(&cache);
    put(cache.join("nest-fix/d3d11.dll"), "fix");
    put(cache.join("nest-fix/d3dx.ini"), "[Stereo]\nx = 0\n");
    put(cache.join("nest-fix/bin/dxgi.dll"), "new nested");
    put(cache.join("nest-fix/bin/d3dx.ini"), "[Stereo]\nx = 1\n");
    put(game.join("Game.exe"), "exe");
    put(game.join("dxgi.dll"), "top dxgi");
    put(game.join("bin/dxgi.dll"), "nested dxgi");
    let m = settings(&[
        ("geo11-fix", "nest-fix"),
        ("geo11-exe", "Game.exe"),
        ("geo11-remove-first", "bin/dxgi.dll"),
    ]);
    let steps = geo11::plan_files(&cache, &game, &m).unwrap();
    assert!(steps.contains(&FileStep::Remove(game.join("bin/dxgi.dll"))));
    assert!(!steps.contains(&FileStep::Remove(game.join("dxgi.dll"))));
    assert!(steps.contains(&FileStep::Copy {
        from: cache.join("nest-fix/bin/dxgi.dll"),
        to: game.join("bin/dxgi.dll"),
    }));
    // A nested d3dx.ini is copied verbatim, not patched.
    assert!(steps.contains(&FileStep::Copy {
        from: cache.join("nest-fix/bin/d3dx.ini"),
        to: game.join("bin/d3dx.ini"),
    }));
    // Copies come before Writes.
    let first_write = steps
        .iter()
        .position(|s| matches!(s, FileStep::Write { .. }))
        .unwrap();
    assert!(steps[first_write..]
        .iter()
        .all(|s| matches!(s, FileStep::Write { .. })));
    let (_, env) = geo11::apply(&cache, &game, 7, &m, &viewer_exe(&r)).unwrap();
    // Only a dxgi.dll next to the exe adds dxgi to the override.
    assert_eq!(env.env[0].1, "d3d11,nvapi64=n,b");
    assert_eq!(fs::read(game.join("dxgi.dll")).unwrap(), b"top dxgi");
    assert_eq!(fs::read(game.join("bin/dxgi.dll")).unwrap(), b"new nested");
}

#[test]
fn rollback_restores_original_of_removed_and_recreated_path() {
    let r = root("rollback-recreate");
    let cache = r.join("cache");
    let game = r.join("game");
    let m = control(&cache, &game);
    put(cache.join("control-fix/dxgi.dll"), "new dxgi");
    put(cache.join("control-fix/zzz.dll"), "late");
    fs::create_dir_all(game.join("zzz.dll")).unwrap();
    let viewer = viewer_exe(&r);
    let before = snapshot(&r);
    let err = geo11::apply(&cache, &game, 8, &m, &viewer).unwrap_err();
    assert!(err.starts_with("io: "));
    assert_eq!(fs::read(game.join("dxgi.dll")).unwrap(), b"old dxgi");
    assert_eq!(snapshot(&r), before);
}

#[test]
fn route_paths_and_pin_keys_cannot_leave_their_folders() {
    let r = root("traversal");
    let cache = r.join("cache");
    let game = r.join("game");
    let mut m = control(&cache, &game);
    put(r.join("outside.exe"), "exe");
    put(cache.join("evil/d3dx.ini"), "[Stereo]\n");
    for (key, value) in [
        ("geo11-exe", "../outside.exe"),
        ("geo11-exe", "/etc/hostname"),
        ("geo11-fix", "../cache/evil"),
        ("geo11-fix", "Control-Fix"),
        ("geo11-remove-first", "./dxgi.dll"),
        ("geo11-remove-first", ""),
        ("geo11-d3dx", "force_stereo=a=b"),
        ("geo11-d3dx", "force_stereo="),
        ("geo11-d3dx", "force_stereo=2\n[Evil]"),
    ] {
        let saved = m
            .insert(key.to_owned(), value.to_owned())
            .unwrap_or_default();
        let err = geo11::plan_files(&cache, &game, &m).unwrap_err();
        assert!(err.starts_with("bad-recipe: "), "{key}={value}: {err}");
        if saved.is_empty() {
            m.remove(key);
        } else {
            m.insert(key.to_owned(), saved);
        }
    }
    // Spaces around a d3dx key are trimmed, so patching stays idempotent.
    let s = Geo11Settings::from_route(&settings(&[("geo11-d3dx", " force_stereo = 3")])).unwrap();
    let once = geo11::d3dx_patch("[Stereo]\nforce_stereo=0\n", &s).unwrap();
    assert_eq!(once, "[Stereo]\nforce_stereo=3\n");
    assert_eq!(geo11::d3dx_patch(&once, &s).unwrap(), once);
}

#[test]
fn existing_names_keep_their_case_in_nested_folders() {
    let r = root("case");
    let cache = r.join("cache");
    let game = r.join("game");
    let m = control(&cache, &game);
    put(game.join("shaderfixes/old.txt"), "old");
    put(game.join("D3D11.DLL"), "game dll");
    let steps = geo11::plan_files(&cache, &game, &m).unwrap();
    assert!(steps.iter().any(
        |s| matches!(s, FileStep::Copy { to, .. } if *to == game.join("shaderfixes/a-ps.txt"))
    ));
    assert!(steps
        .iter()
        .any(|s| matches!(s, FileStep::Copy { to, .. } if *to == game.join("D3D11.DLL"))));
    assert!(!steps.iter().any(
        |s| matches!(s, FileStep::Copy { to, .. } if to.starts_with(game.join("ShaderFixes")))
    ));
}

#[test]
fn stale_nested_backup_refuses_and_symlink_loops_end() {
    let r = root("stale");
    let cache = r.join("cache");
    let game = r.join("game");
    let m = control(&cache, &game);
    #[cfg(unix)]
    std::os::unix::fs::symlink("..", cache.join("control-fix/ShaderFixes/loop")).unwrap();
    put(game.join("ShaderFixes/a-ps.txt"), "old shader");
    put(
        game.join("ShaderFixes/a-ps.txt.flat3d-backup"),
        "older shader",
    );
    let before = snapshot(&game);
    let err = geo11::apply(&cache, &game, 9, &m, &viewer_exe(&r)).unwrap_err();
    assert!(err.starts_with("refused: "), "{err}");
    assert_eq!(snapshot(&game), before);
}

#[test]
fn stale_version_folder_is_not_picked() {
    let pins = flat3d_inject::injectors::fetch::parse_pins(flat3d_inject::injectors::fetch::PINS);
    let v = pins.get("geo11").expect("geo11 pin").version.clone();
    assert!(!v.is_empty() && !v.starts_with("TBD"));

    let r = root("stale_version");
    let cache = r.join("cache");
    let game = r.join("game");
    put(game.join("Control_DX11.exe"), "exe");
    put(
        cache.join("control-fix/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n",
    );

    let oldver = "0.0.1";
    assert!(oldver < v.as_str());

    let old_dir = cache.join("geo11").join(oldver);
    put(old_dir.join(".flat3d-fetch"), "old\n");
    put(old_dir.join("x64/d3d11.dll"), "dll-old");
    put(old_dir.join("x64/nvapi64.dll"), "nvapi-old");
    put(
        old_dir.join("x64/d3dxdm.ini"),
        "direct_mode=sbs\ndm_separation=11\ndm_convergence=1.0\nver=11\n",
    );

    let v_dir = cache.join("geo11").join(&v);
    put(v_dir.join(".flat3d-fetch"), format!("{v}\n"));
    put(v_dir.join("x64/d3d11.dll"), "dll-new");
    put(v_dir.join("x64/nvapi64.dll"), "nvapi-new");
    put(
        v_dir.join("x64/d3dxdm.ini"),
        "direct_mode=sbs\ndm_separation=22\ndm_convergence=1.0\nver=22\n",
    );

    let m = settings(&[
        ("geo11-fix", "control-fix"),
        ("geo11-exe", "Control_DX11.exe"),
        ("geo11-separation", "22"),
    ]);

    let plan = geo11::plan_files(&cache, &game, &m).unwrap();
    let d3dxdm_write = plan
        .iter()
        .find_map(|s| match s {
            FileStep::Write { to, text } if to.file_name().unwrap() == "d3dxdm.ini" => {
                Some(text.as_str())
            }
            _ => None,
        })
        .expect("Write step for d3dxdm.ini");
    assert!(d3dxdm_write.contains("22"), "{d3dxdm_write}");
    assert!(!d3dxdm_write.contains("11"), "{d3dxdm_write}");
    assert!(plan
        .iter()
        .any(|s| matches!(s, FileStep::Copy { from, .. } if from == &v_dir.join("x64/d3d11.dll"))));

    // Variant: only the old version folder (with marker) exists and no V folder:
    // plan_files returns an Err starting with "not-found: geo11 is not fetched"
    let r_old_only = root("stale_old_only");
    let cache_old_only = r_old_only.join("cache");
    let game_old_only = r_old_only.join("game");
    put(game_old_only.join("Control_DX11.exe"), "exe");
    put(
        cache_old_only.join("control-fix/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n",
    );
    let old_only_dir = cache_old_only.join("geo11").join(oldver);
    put(old_only_dir.join(".flat3d-fetch"), "old\n");
    put(old_only_dir.join("x64/d3d11.dll"), "dll-old");
    put(old_only_dir.join("x64/nvapi64.dll"), "nvapi-old");
    put(
        old_only_dir.join("x64/d3dxdm.ini"),
        "direct_mode=sbs\ndm_separation=11\ndm_convergence=1.0\n",
    );
    let err = geo11::plan_files(&cache_old_only, &game_old_only, &m).unwrap_err();
    assert!(err.starts_with("not-found: geo11 is not fetched"), "{err}");
}

#[test]
fn leftover_tmp_folder_is_not_picked() {
    let pins = flat3d_inject::injectors::fetch::parse_pins(flat3d_inject::injectors::fetch::PINS);
    let v = pins.get("geo11").expect("geo11 pin").version.clone();
    assert!(!v.is_empty() && !v.starts_with("TBD"));

    let r = root("tmp_leftover");
    let cache = r.join("cache");
    let game = r.join("game");
    put(game.join("Control_DX11.exe"), "exe");
    put(
        cache.join("control-fix/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n",
    );

    let tmp_dir = cache.join("geo11").join(format!(".{v}.tmp"));
    put(tmp_dir.join("x64/d3d11.dll"), "dll-wrong");
    put(tmp_dir.join("x64/nvapi64.dll"), "nvapi-wrong");
    put(
        tmp_dir.join("x64/d3dxdm.ini"),
        "direct_mode=sbs\ndm_separation=99\ndm_convergence=1.0\nwrong\n",
    );

    let v_dir = cache.join("geo11").join(&v);
    put(v_dir.join(".flat3d-fetch"), format!("{v}\n"));
    put(v_dir.join("x64/d3d11.dll"), "dll-right");
    put(v_dir.join("x64/nvapi64.dll"), "nvapi-right");
    put(
        v_dir.join("x64/d3dxdm.ini"),
        "direct_mode=sbs\ndm_separation=22\ndm_convergence=1.0\nright\n",
    );

    let m = settings(&[
        ("geo11-fix", "control-fix"),
        ("geo11-exe", "Control_DX11.exe"),
        ("geo11-separation", "22"),
    ]);

    let plan = geo11::plan_files(&cache, &game, &m).unwrap();
    let d3dxdm_write = plan
        .iter()
        .find_map(|s| match s {
            FileStep::Write { to, text } if to.file_name().unwrap() == "d3dxdm.ini" => {
                Some(text.as_str())
            }
            _ => None,
        })
        .expect("Write step for d3dxdm.ini");
    assert!(d3dxdm_write.contains("right"), "{d3dxdm_write}");
    assert!(!d3dxdm_write.contains("wrong"), "{d3dxdm_write}");
    assert!(plan
        .iter()
        .any(|s| matches!(s, FileStep::Copy { from, .. } if from == &v_dir.join("x64/d3d11.dll"))));

    // Variant: with only the .tmp folder present, the result is Err "not-found: geo11 is not fetched"
    let r_only_tmp = root("tmp_only");
    let cache_only_tmp = r_only_tmp.join("cache");
    let game_only_tmp = r_only_tmp.join("game");
    put(game_only_tmp.join("Control_DX11.exe"), "exe");
    put(
        cache_only_tmp.join("control-fix/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n",
    );
    let tmp_only_dir = cache_only_tmp.join("geo11").join(format!(".{v}.tmp"));
    put(tmp_only_dir.join("x64/d3d11.dll"), "dll-wrong");
    put(tmp_only_dir.join("x64/nvapi64.dll"), "nvapi-wrong");
    put(
        tmp_only_dir.join("x64/d3dxdm.ini"),
        "direct_mode=sbs\ndm_separation=99\ndm_convergence=1.0\nwrong\n",
    );
    let err = geo11::plan_files(&cache_only_tmp, &game_only_tmp, &m).unwrap_err();
    assert!(err.starts_with("not-found: geo11 is not fetched"), "{err}");

    // Fix key: <cache>/control-fix/.1.2.tmp/d3dx.ini must not be picked over <cache>/control-fix/d3dx.ini
    // In sorted order, .1.2.tmp sorts before fix and would win without the fix.
    let r_fix = root("tmp_fix");
    let cache_fix = r_fix.join("cache");
    let game_fix = r_fix.join("game");
    put(game_fix.join("Control_DX11.exe"), "exe");
    geo11_overlay(&cache_fix);

    put(
        cache_fix.join("control-fix/.1.2.tmp/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n;wrong_ini = 1\n",
    );
    put(
        cache_fix.join("control-fix/fix/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n;right_ini = 1\n",
    );

    let m_fix = settings(&[
        ("geo11-fix", "control-fix"),
        ("geo11-exe", "Control_DX11.exe"),
    ]);

    let plan_fix = geo11::plan_files(&cache_fix, &game_fix, &m_fix).unwrap();
    let d3dx_write = plan_fix
        .iter()
        .find_map(|s| match s {
            FileStep::Write { to, text } if to.file_name().unwrap() == "d3dx.ini" => {
                Some(text.as_str())
            }
            _ => None,
        })
        .expect("Write step for d3dx.ini");
    assert!(d3dx_write.contains("right_ini"), "{d3dx_write}");
    assert!(!d3dx_write.contains("wrong_ini"), "{d3dx_write}");

    // Also verify direct d3dx.ini in <cache>/control-fix/ wins over leftover .1.2.tmp
    let r_direct = root("tmp_direct_fix");
    let cache_direct = r_direct.join("cache");
    let game_direct = r_direct.join("game");
    put(game_direct.join("Control_DX11.exe"), "exe");
    geo11_overlay(&cache_direct);
    put(
        cache_direct.join("control-fix/.1.2.tmp/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n;wrong_ini = 1\n",
    );
    put(
        cache_direct.join("control-fix/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n;direct_ini = 1\n",
    );
    let plan_direct = geo11::plan_files(&cache_direct, &game_direct, &m_fix).unwrap();
    let direct_write = plan_direct
        .iter()
        .find_map(|s| match s {
            FileStep::Write { to, text } if to.file_name().unwrap() == "d3dx.ini" => {
                Some(text.as_str())
            }
            _ => None,
        })
        .expect("Write step for d3dx.ini");
    assert!(direct_write.contains("direct_ini"), "{direct_write}");
    assert!(!direct_write.contains("wrong_ini"), "{direct_write}");

    // The copy walk from the key folder must skip the same folders: a sibling old version
    // (with the fetch marker) and the .1.2.tmp staging folder give no Copy or Write step.
    let old_dir = cache_direct.join("control-fix/1.0");
    put(
        old_dir.join(".flat3d-fetch"),
        "1.0
",
    );
    put(old_dir.join("ShaderFixes/old-ps.txt"), "old");
    put(
        cache_direct.join("control-fix/ShaderFixes/new-ps.txt"),
        "new",
    );
    let plan_direct = geo11::plan_files(&cache_direct, &game_direct, &m_fix).unwrap();
    let skipped = [cache_direct.join("control-fix/.1.2.tmp"), old_dir.clone()];
    for s in &plan_direct {
        if let FileStep::Copy { from, to } = s {
            assert!(
                !skipped.iter().any(|d| from.starts_with(d)),
                "copied from a skipped folder: {from:?}"
            );
            assert!(!to.starts_with(game_direct.join(".1.2.tmp")), "{to:?}");
            assert!(!to.starts_with(game_direct.join("1.0")), "{to:?}");
        }
        if let FileStep::Write { to, .. } = s {
            assert!(!to.starts_with(game_direct.join(".1.2.tmp")), "{to:?}");
            assert!(!to.starts_with(game_direct.join("1.0")), "{to:?}");
        }
    }
    assert!(plan_direct.iter().any(|s| matches!(s, FileStep::Copy { from, .. } if from == &cache_direct.join("control-fix/ShaderFixes/new-ps.txt"))));

    // Variant: only the .tmp folder present for the fix key returns not-found
    let r_only_fix_tmp = root("only_fix_tmp");
    let cache_only_fix = r_only_fix_tmp.join("cache");
    let game_only_fix = r_only_fix_tmp.join("game");
    put(game_only_fix.join("Control_DX11.exe"), "exe");
    geo11_overlay(&cache_only_fix);
    put(
        cache_only_fix.join("control-fix/.1.2.tmp/d3dx.ini"),
        "[Stereo]\nforce_stereo = 0\n",
    );
    let err_fix = geo11::plan_files(&cache_only_fix, &game_only_fix, &m_fix).unwrap_err();
    assert!(
        err_fix.starts_with("not-found: control-fix is not fetched"),
        "{err_fix}"
    );
}
