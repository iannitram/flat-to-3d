//! Installed Steam games: parse `libraryfolders.vdf` and each library's `appmanifest_*.acf`.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Steam's root on SteamOS, relative to the home folder.
pub const DEFAULT_STEAM_ROOT: &str = ".local/share/Steam";

/// `StateFlags` bit 4: the game is fully installed (not downloading or updating).
const FULLY_INSTALLED: u32 = 4;

/// One fully installed game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub appid: u32,
    pub name: String,
    /// The game's folder under `steamapps/common/`.
    pub install_dir: PathBuf,
}

/// Every fully installed game in the Steam root and in every library `libraryfolders.vdf` lists,
/// sorted by name. A game that two of those folders hold is listed once.
///
/// A missing `libraryfolders.vdf` gives an empty list. A library folder that is gone, or a
/// manifest that does not parse, is skipped. A `libraryfolders.vdf` that does not parse is an error.
pub fn installed_games(steam_root: &Path) -> io::Result<Vec<Game>> {
    let vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
    let text = match fs::read_to_string(&vdf) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };
    let top = parse(&text).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "malformed libraryfolders.vdf")
    })?;
    // The root is always read: older files may list only the extra libraries.
    let libraries = std::iter::once(steam_root.to_path_buf()).chain(library_paths(&top));
    let mut games: Vec<Game> = libraries
        .flat_map(|library| installed_in(&library))
        .collect();
    // A game two folders hold is listed once: its first copy whose install folder exists, else its
    // first copy. The sort is stable, so the earlier folder wins among copies that are alike.
    games.sort_by_key(|game| !game.install_dir.is_dir());
    let mut seen = HashSet::new();
    games.retain(|game| seen.insert(game.appid));
    games.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.appid.cmp(&b.appid))
    });
    Ok(games)
}

/// The path of every library listed under `"libraryfolders"`, in file order. A relative or empty
/// path is dropped: read from the working directory, it would find games that are in no library.
fn library_paths(top: &[Pair]) -> Vec<PathBuf> {
    let Some(folders) = block(top, "libraryfolders") else {
        return Vec::new();
    };
    folders
        .iter()
        .filter_map(|(key, value)| match value {
            Value::Block(library) => field(library, "path").map(PathBuf::from),
            // Steam before 2021 wrote each library as `"<number>" "<path>"`.
            Value::Text(path) if key.parse::<u32>().is_ok() => Some(PathBuf::from(path)),
            Value::Text(_) => None,
        })
        .filter(|path| path.is_absolute())
        .collect()
}

/// The fully installed games whose manifests sit in one library's `steamapps` folder.
fn installed_in(library: &Path) -> Vec<Game> {
    let Ok(entries) = fs::read_dir(library.join("steamapps")) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| is_manifest(&entry.file_name()))
        .filter_map(|entry| game_from_manifest(&fs::read_to_string(entry.path()).ok()?, library))
        .collect()
}

fn is_manifest(file_name: &OsStr) -> bool {
    file_name
        .to_str()
        .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"))
}

/// One manifest as a game, or `None` when it does not parse, is not fully installed, lacks a field,
/// or names an install folder that is not a single name under `steamapps/common`.
fn game_from_manifest(text: &str, library: &Path) -> Option<Game> {
    let top = parse(text)?;
    let app = block(&top, "AppState")?;
    let flags: u32 = field(app, "StateFlags")?.parse().ok()?;
    if flags & FULLY_INSTALLED == 0 {
        return None;
    }
    let installdir = field(app, "installdir").filter(|dir| is_folder_name(dir))?;
    Some(Game {
        appid: field(app, "appid")?.parse().ok()?,
        name: field(app, "name")?.to_string(),
        install_dir: library.join("steamapps").join("common").join(installdir),
    })
}

/// True when `name` is one plain folder name. An absolute path, a `..` or an empty value would point
/// outside `steamapps/common`, and `Path::join` with an absolute path replaces the whole base.
/// Windows drops trailing dots and spaces, so `...` names `common` itself and `Game 50.` names the
/// sibling `Game 50`: on Windows a name ending in a dot or a space is refused. Linux keeps such names.
fn is_folder_name(name: &str) -> bool {
    if cfg!(windows) && (name.ends_with('.') || name.ends_with(' ')) {
        return false;
    }
    let mut parts = Path::new(name).components();
    matches!(
        (parts.next(), parts.next()),
        (Some(Component::Normal(_)), None)
    )
}

/// A KeyValues value: quoted text, or a `{ }` block of more pairs.
#[derive(Debug, PartialEq)]
enum Value {
    Text(String),
    Block(Vec<Pair>),
}

type Pair = (String, Value);

enum Token {
    Text(String),
    Open,
    Close,
}

/// The first text value under `key` (keys match without regard to case).
fn field<'a>(pairs: &'a [Pair], key: &str) -> Option<&'a str> {
    pairs.iter().find_map(|(k, value)| match value {
        Value::Text(text) if k.eq_ignore_ascii_case(key) => Some(text.as_str()),
        _ => None,
    })
}

/// The first block under `key` (keys match without regard to case).
fn block<'a>(pairs: &'a [Pair], key: &str) -> Option<&'a [Pair]> {
    pairs.iter().find_map(|(k, value)| match value {
        Value::Block(pairs) if k.eq_ignore_ascii_case(key) => Some(pairs.as_slice()),
        _ => None,
    })
}

/// How deeply blocks may nest. Steam's files nest three or four levels; deeper is malformed, and
/// the limit keeps a hostile file from overflowing the stack.
const MAX_DEPTH: usize = 64;

/// Parses a whole KeyValues file into its top-level pairs, or `None` when it is malformed.
fn parse(text: &str) -> Option<Vec<Pair>> {
    let tokens = tokenize(text)?;
    let mut pos = 0;
    parse_pairs(&tokens, &mut pos, true, 0)
}

/// Reads pairs until the end of input (top level) or a closing brace (inside a block).
/// `depth` is how many blocks enclose these pairs.
fn parse_pairs(
    tokens: &[Token],
    pos: &mut usize,
    top_level: bool,
    depth: usize,
) -> Option<Vec<Pair>> {
    let mut pairs = Vec::new();
    loop {
        let key = match tokens.get(*pos) {
            None => return top_level.then_some(pairs),
            Some(Token::Close) => {
                *pos += 1;
                return (!top_level).then_some(pairs);
            }
            Some(Token::Open) => return None,
            Some(Token::Text(key)) => key.clone(),
        };
        *pos += 1;
        let value = match tokens.get(*pos)? {
            Token::Text(text) => {
                *pos += 1;
                Value::Text(text.clone())
            }
            Token::Open => {
                *pos += 1;
                if depth >= MAX_DEPTH {
                    return None;
                }
                Value::Block(parse_pairs(tokens, pos, false, depth + 1)?)
            }
            Token::Close => return None,
        };
        pairs.push((key, value));
    }
}

/// Splits KeyValues text into quoted strings and braces. `//` runs to the end of the line.
/// Inside quotes only `\\` and `\"` are escapes; any other backslash is kept, so Windows paths survive.
fn tokenize(text: &str) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            // Editors can leave a byte-order mark at the start; it is not whitespace.
            c if c.is_whitespace() || c == '\u{feff}' => {
                chars.next();
            }
            '/' => {
                chars.next();
                if chars.next() != Some('/') {
                    return None;
                }
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '{' => {
                chars.next();
                tokens.push(Token::Open);
            }
            '}' => {
                chars.next();
                tokens.push(Token::Close);
            }
            '"' => {
                chars.next();
                let mut text = String::new();
                loop {
                    match chars.next()? {
                        '"' => break,
                        '\\' => match chars.peek() {
                            Some('\\') => text.push(chars.next()?),
                            Some('"') => text.push(chars.next()?),
                            _ => text.push('\\'),
                        },
                        c => text.push(c),
                    }
                }
                tokens.push(Token::Text(text));
            }
            _ => return None,
        }
    }
    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyvalues_keep_escaped_quotes_and_windows_paths() {
        let parsed = parse("\"a\" \"say \\\"hi\\\" C:\\Games\"\n\"b\" \"x\\\\y\"").unwrap();
        assert_eq!(
            parsed,
            vec![
                ("a".into(), Value::Text("say \"hi\" C:\\Games".into())),
                ("b".into(), Value::Text("x\\y".into())),
            ]
        );
    }

    #[test]
    fn an_unclosed_block_is_malformed() {
        assert!(parse("\"libraryfolders\" {\n\"0\" { \"path\" \"/x\"\n").is_none());
    }

    #[test]
    fn the_older_libraryfolders_layout_lists_paths_by_number() {
        let games = std::env::temp_dir().join("flat3d-steam-games");
        let text = format!(
            "\"LibraryFolders\"\n{{\n\t\"TimeNextStatsReport\"\t\t\"1\"\n\t\"1\"\t\t\"{}\"\n}}\n",
            games.display()
        );
        let top = parse(&text).unwrap();
        assert_eq!(library_paths(&top), vec![games]);
    }

    #[test]
    fn a_byte_order_mark_at_the_start_is_skipped() {
        let parsed = parse("\u{feff}\"a\" \"b\"").unwrap();
        assert_eq!(parsed, vec![("a".into(), Value::Text("b".into()))]);
    }

    #[test]
    fn an_empty_or_relative_library_path_is_dropped() {
        let top = parse(
            "\"libraryfolders\" { \"0\" { \"path\" \"\" } \"1\" { \"path\" \"games\" } \"2\" \"\" \"3\" \"games\" }",
        )
        .unwrap();
        assert_eq!(library_paths(&top), Vec::<PathBuf>::new());
    }

    #[test]
    fn blocks_nested_past_the_limit_are_malformed() {
        let nested = |depth: usize| format!("{}{}", "\"x\" {".repeat(depth), "}".repeat(depth));
        assert!(parse(&nested(MAX_DEPTH)).is_some());
        assert!(parse(&nested(MAX_DEPTH + 1)).is_none());
    }

    fn fresh_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("flat3d-steam-unit-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_vdf(root: &Path, text: &str) {
        fs::create_dir_all(root.join("steamapps")).unwrap();
        fs::write(root.join("steamapps").join("libraryfolders.vdf"), text).unwrap();
    }

    fn add_manifest(library: &Path, appid: u32) {
        let apps = library.join("steamapps");
        fs::create_dir_all(&apps).unwrap();
        let text = format!(
            "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"name\"\t\t\"Game {appid}\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game {appid}\"\n}}\n"
        );
        fs::write(apps.join(format!("appmanifest_{appid}.acf")), text).unwrap();
    }

    #[test]
    fn the_root_is_read_even_when_an_older_vdf_leaves_it_out() {
        let root = fresh_dir("root-left-out");
        let sd = fresh_dir("root-left-out-sd");
        write_vdf(
            &root,
            &format!(
                "\"LibraryFolders\"\n{{\n\t\"1\"\t\t\"{}\"\n}}\n",
                sd.display()
            ),
        );
        add_manifest(&root, 10);
        add_manifest(&sd, 20);
        let appids: Vec<u32> = installed_games(&root)
            .unwrap()
            .iter()
            .map(|game| game.appid)
            .collect();
        assert_eq!(appids, vec![10, 20]);
    }

    #[test]
    fn a_game_listed_by_two_folders_is_listed_once() {
        let root = fresh_dir("listed-twice");
        let sd = fresh_dir("listed-twice-sd");
        let (root_path, sd_path) = (root.display(), sd.display());
        write_vdf(
            &root,
            &format!(
                "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{root_path}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{sd_path}\"\n\t}}\n}}\n"
            ),
        );
        add_manifest(&root, 30);
        add_manifest(&sd, 30);
        add_manifest(&sd, 31);
        fs::create_dir_all(root.join("steamapps/common/Game 30")).unwrap();
        fs::create_dir_all(sd.join("steamapps/common/Game 30")).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![
                Game {
                    appid: 30,
                    name: "Game 30".into(),
                    install_dir: root.join("steamapps/common/Game 30")
                },
                Game {
                    appid: 31,
                    name: "Game 31".into(),
                    install_dir: sd.join("steamapps/common/Game 31")
                },
            ]
        );
    }

    #[test]
    fn a_stale_copy_does_not_hide_the_install_that_exists() {
        let root = fresh_dir("stale-root");
        let sd = fresh_dir("stale-sd");
        write_vdf(
            &root,
            &format!("\"libraryfolders\" {{ \"1\" \"{}\" }}\n", sd.display()),
        );
        add_manifest(&root, 40);
        add_manifest(&sd, 40);
        fs::create_dir_all(sd.join("steamapps/common/Game 40")).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 40,
                name: "Game 40".into(),
                install_dir: sd.join("steamapps/common/Game 40")
            }]
        );
    }

    #[test]
    fn an_installdir_that_leaves_steamapps_common_is_skipped() {
        let root = fresh_dir("escape");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        let apps = root.join("steamapps");
        let installdirs = [
            (41, "../../escape"),
            (42, "/etc"),
            (43, "Game 43"),
            (44, ".."),
            (45, "."),
            (46, "/"),
            (47, ""),
            (48, "Game/../../escape"),
            (49, "..."),
            (50, "Game 50."),
            (51, "Game 51 "),
        ];
        for (appid, installdir) in installdirs {
            let text = format!(
                "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"name\"\t\t\"Game {appid}\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"{installdir}\"\n}}\n"
            );
            fs::write(apps.join(format!("appmanifest_{appid}.acf")), text).unwrap();
        }
        let appids: Vec<u32> = installed_games(&root)
            .unwrap()
            .iter()
            .map(|game| game.appid)
            .collect();
        // Windows drops a trailing dot or space from a name, so 49, 50 and 51 are skipped there only.
        let expected = if cfg!(windows) {
            vec![43]
        } else {
            vec![43, 49, 50, 51]
        };
        assert_eq!(appids, expected);
    }

    #[test]
    fn a_non_numeric_key_with_an_absolute_path_is_not_a_library() {
        let root = fresh_dir("stats-key");
        let sd = fresh_dir("stats-key-sd");
        write_vdf(
            &root,
            &format!("\"libraryfolders\" {{ \"Foo\" \"{}\" }}\n", sd.display()),
        );
        add_manifest(&sd, 50);
        assert_eq!(installed_games(&root).unwrap(), vec![]);
    }

    #[test]
    fn among_extra_libraries_the_one_listed_first_wins() {
        let root = fresh_dir("extra-order");
        let first = fresh_dir("extra-order-first");
        let second = fresh_dir("extra-order-second");
        write_vdf(
            &root,
            &format!(
                "\"LibraryFolders\"\n{{\n\t\"1\"\t\t\"{}\"\n\t\"2\"\t\t\"{}\"\n}}\n",
                first.display(),
                second.display()
            ),
        );
        for library in [&first, &second] {
            add_manifest(library, 30);
            fs::create_dir_all(library.join("steamapps/common/Game 30")).unwrap();
        }
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 30,
                name: "Game 30".into(),
                install_dir: first.join("steamapps/common/Game 30")
            }]
        );
    }

    #[test]
    fn the_root_copy_wins_when_the_vdf_omits_the_root() {
        let root = fresh_dir("root-omitted");
        let sd = fresh_dir("root-omitted-sd");
        write_vdf(
            &root,
            &format!(
                "\"LibraryFolders\"\n{{\n\t\"1\"\t\t\"{}\"\n}}\n",
                sd.display()
            ),
        );
        add_manifest(&root, 30);
        add_manifest(&sd, 30);
        fs::create_dir_all(root.join("steamapps/common/Game 30")).unwrap();
        fs::create_dir_all(sd.join("steamapps/common/Game 30")).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 30,
                name: "Game 30".into(),
                install_dir: root.join("steamapps/common/Game 30")
            }]
        );
    }

    #[test]
    fn bit_4_decides_whether_a_game_is_listed() {
        let root = fresh_dir("bit-4");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        let apps = root.join("steamapps");
        for (appid, flags) in [(60, 0), (61, 5), (62, 6), (63, 1030), (64, 16)] {
            let text = format!(
                "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"name\"\t\t\"Game {appid}\"\n\t\"StateFlags\"\t\t\"{flags}\"\n\t\"installdir\"\t\t\"Game {appid}\"\n}}\n"
            );
            fs::write(apps.join(format!("appmanifest_{appid}.acf")), text).unwrap();
        }
        let appids: Vec<u32> = installed_games(&root)
            .unwrap()
            .iter()
            .map(|game| game.appid)
            .collect();
        assert_eq!(appids, vec![61, 62, 63]);
    }

    #[test]
    fn a_malformed_vdf_is_an_error_not_a_partial_list() {
        let root = fresh_dir("malformed-vdf");
        let sd = fresh_dir("malformed-vdf-sd");
        fs::create_dir_all(root.join("steamapps")).unwrap();
        // The outer block is never closed.
        let text = format!(
            "\"libraryfolders\"\n{{\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n",
            sd.display()
        );
        fs::write(root.join("steamapps").join("libraryfolders.vdf"), text).unwrap();
        add_manifest(&root, 10);
        add_manifest(&sd, 20);
        let err = installed_games(&root).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn a_stray_closing_brace_is_malformed() {
        assert!(parse("\"a\" \"b\" }").is_none());
    }

    fn add_named_manifest(library: &Path, appid: u32, name: &str) {
        let apps = library.join("steamapps");
        fs::create_dir_all(&apps).unwrap();
        let text = format!(
            "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"name\"\t\t\"{name}\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"{name} {appid}\"\n}}\n"
        );
        fs::write(apps.join(format!("appmanifest_{appid}.acf")), text).unwrap();
    }

    #[test]
    fn only_acf_files_are_manifests() {
        let root = fresh_dir("only-acf");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        add_manifest(&root, 71);
        // A real manifest's text under a name Steam does not write, so it is not a manifest.
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"70\"\n\t\"name\"\t\t\"Game 70\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game 70\"\n}\n";
        fs::write(root.join("steamapps").join("appmanifest_70.txt"), text).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 71,
                name: "Game 71".into(),
                install_dir: root.join("steamapps/common/Game 71")
            }]
        );
    }

    #[test]
    fn keys_match_without_regard_to_case() {
        let root = fresh_dir("key-case");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"80\"\n\t\"name\"\t\t\"Game 80\"\n\t\"stateflags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game 80\"\n}\n";
        fs::write(root.join("steamapps").join("appmanifest_80.acf"), text).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 80,
                name: "Game 80".into(),
                install_dir: root.join("steamapps/common/Game 80")
            }]
        );
    }

    #[test]
    fn games_with_the_same_name_sort_by_appid() {
        let root = fresh_dir("same-name-order");
        let sd = fresh_dir("same-name-order-sd");
        write_vdf(
            &root,
            &format!(
                "\"libraryfolders\" {{\n\t\"1\"\t\t\"{}\"\n}}\n",
                sd.display()
            ),
        );
        add_named_manifest(&root, 10, "Twin");
        add_named_manifest(&sd, 9, "Twin");
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![
                Game {
                    appid: 9,
                    name: "Twin".into(),
                    install_dir: sd.join("steamapps/common/Twin 9")
                },
                Game {
                    appid: 10,
                    name: "Twin".into(),
                    install_dir: root.join("steamapps/common/Twin 10")
                },
            ]
        );
    }

    #[test]
    fn two_games_with_the_same_name_are_both_listed() {
        let root = fresh_dir("same-name-twice");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        add_named_manifest(&root, 70, "Twin");
        add_named_manifest(&root, 71, "Twin");
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![
                Game {
                    appid: 70,
                    name: "Twin".into(),
                    install_dir: root.join("steamapps/common/Twin 70")
                },
                Game {
                    appid: 71,
                    name: "Twin".into(),
                    install_dir: root.join("steamapps/common/Twin 71")
                },
            ]
        );
    }

    #[test]
    fn a_manifest_without_an_installdir_is_left_out() {
        let root = fresh_dir("no-installdir");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"90\"\n\t\"name\"\t\t\"Game 90\"\n\t\"StateFlags\"\t\t\"4\"\n}\n";
        fs::write(root.join("steamapps").join("appmanifest_90.acf"), text).unwrap();
        add_manifest(&root, 91);
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 91,
                name: "Game 91".into(),
                install_dir: root.join("steamapps/common/Game 91")
            }]
        );
    }

    #[test]
    fn an_installdir_is_one_folder_name() {
        let root = fresh_dir("one-folder-name");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        fs::create_dir_all(
            root.join("steamapps")
                .join("common")
                .join("Game 93")
                .join("Sub"),
        )
        .unwrap();
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"93\"\n\t\"name\"\t\t\"Game 93\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game 93/Sub\"\n}\n";
        fs::write(root.join("steamapps").join("appmanifest_93.acf"), text).unwrap();
        add_manifest(&root, 94);
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 94,
                name: "Game 94".into(),
                install_dir: root.join("steamapps/common/Game 94")
            }]
        );
    }

    #[test]
    fn a_legacy_key_must_be_a_library_number() {
        let root = fresh_dir("legacy-key-number");
        let sd = fresh_dir("legacy-key-number-sd");
        let text = format!(
            "\"LibraryFolders\"\n{{\n\t\"\"\t\t\"{0}\"\n\t\"99999999999\"\t\t\"{0}\"\n}}\n",
            sd.display()
        );
        write_vdf(&root, &text);
        add_manifest(&sd, 50);
        assert_eq!(installed_games(&root).unwrap(), vec![]);
    }

    #[test]
    fn only_files_named_appmanifest_are_manifests() {
        let root = fresh_dir("appmanifest-name");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        add_manifest(&root, 72);
        // A complete manifest under another name is not one of Steam's manifests.
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"73\"\n\t\"name\"\t\t\"Game 73\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game 73\"\n}\n";
        fs::write(root.join("steamapps").join("backup.acf"), text).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 72,
                name: "Game 72".into(),
                install_dir: root.join("steamapps/common/Game 72")
            }]
        );
    }

    #[test]
    fn a_file_at_the_install_path_is_not_an_install() {
        let root = fresh_dir("file-not-install");
        let sd = fresh_dir("file-not-install-sd");
        write_vdf(
            &root,
            &format!(
                "\"libraryfolders\" {{\n\t\"1\"\t\t\"{}\"\n}}\n",
                sd.display()
            ),
        );
        add_manifest(&root, 40);
        add_manifest(&sd, 40);
        fs::create_dir_all(root.join("steamapps").join("common")).unwrap();
        fs::write(
            root.join("steamapps").join("common").join("Game 40"),
            "not a folder",
        )
        .unwrap();
        fs::create_dir_all(sd.join("steamapps").join("common").join("Game 40")).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 40,
                name: "Game 40".into(),
                install_dir: sd.join("steamapps/common/Game 40")
            }]
        );
    }

    #[test]
    fn names_sort_by_unicode_lowercase() {
        let root = fresh_dir("unicode-sort");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        add_named_manifest(&root, 1, "\u{c9}z");
        add_named_manifest(&root, 2, "\u{e9}a");
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![
                Game {
                    appid: 2,
                    name: "\u{e9}a".into(),
                    install_dir: root.join("steamapps/common/\u{e9}a 2")
                },
                Game {
                    appid: 1,
                    name: "\u{c9}z".into(),
                    install_dir: root.join("steamapps/common/\u{c9}z 1")
                },
            ]
        );
    }

    #[test]
    fn a_single_backslash_stays_a_backslash() {
        assert_eq!(
            parse("\"path\" \"C:\\new\\tools\""),
            Some(vec![(
                "path".to_string(),
                Value::Text("C:\\new\\tools".to_string())
            )])
        );
    }

    #[test]
    fn a_comment_ends_at_the_line_end() {
        let pair = || vec![("a".to_string(), Value::Text("b".to_string()))];
        assert_eq!(parse("// note\n\"a\" \"b\"\n"), Some(pair()));
        assert_eq!(parse("// note\r\n\"a\" \"b\"\r\n"), Some(pair()));
    }

    #[test]
    fn a_lone_slash_is_malformed() {
        assert!(parse("\"a\" \"b\" / \"c\" \"d\"").is_none());
    }

    #[test]
    fn an_unterminated_string_is_malformed() {
        assert!(parse("\"a\" \"b").is_none());
    }

    #[test]
    fn a_key_without_a_value_is_malformed() {
        assert!(parse("\"block\" { \"1\" }").is_none());
    }

    #[test]
    fn the_first_of_two_names_wins() {
        let root = fresh_dir("first-name");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"95\"\n\t\"name\"\t\t\"First\"\n\t\"name\"\t\t\"Second\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game 95\"\n}\n";
        fs::write(root.join("steamapps").join("appmanifest_95.acf"), text).unwrap();
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 95,
                name: "First".into(),
                install_dir: root.join("steamapps/common/Game 95")
            }]
        );
    }

    #[test]
    fn a_manifest_with_a_bad_appid_is_left_out() {
        let root = fresh_dir("bad-appid");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"abc\"\n\t\"name\"\t\t\"Game 96\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game 96\"\n}\n";
        fs::write(root.join("steamapps").join("appmanifest_96.acf"), text).unwrap();
        add_manifest(&root, 61);
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 61,
                name: "Game 61".into(),
                install_dir: root.join("steamapps/common/Game 61")
            }]
        );
    }

    #[test]
    fn a_manifest_without_a_name_is_left_out() {
        let root = fresh_dir("no-name");
        write_vdf(&root, "\"libraryfolders\" {}\n");
        let text = "\"AppState\"\n{\n\t\"appid\"\t\t\"98\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Game 98\"\n}\n";
        fs::write(root.join("steamapps").join("appmanifest_98.acf"), text).unwrap();
        add_manifest(&root, 97);
        assert_eq!(
            installed_games(&root).unwrap(),
            vec![Game {
                appid: 97,
                name: "Game 97".into(),
                install_dir: root.join("steamapps/common/Game 97")
            }]
        );
    }

    #[test]
    fn an_unreadable_vdf_is_an_error() {
        let root = fresh_dir("unreadable-vdf");
        fs::create_dir_all(root.join("steamapps")).unwrap();
        // Bytes that are not UTF-8: the file exists, so this is not "no libraries".
        fs::write(
            root.join("steamapps").join("libraryfolders.vdf"),
            [0xFFu8, 0xFE],
        )
        .unwrap();
        let err = installed_games(&root).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
