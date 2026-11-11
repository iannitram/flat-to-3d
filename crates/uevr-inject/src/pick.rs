#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pick {
    pub pid: u32,
    pub preferred: bool,
}

pub const FALLBACK_AFTER_S: u64 = 10;

pub fn stem_of(path: &str) -> String {
    let basename = path.rsplit(['\\', '/']).next().unwrap_or(path);
    let suffix_start = basename.len().saturating_sub(4);
    if basename
        .get(suffix_start..)
        .is_some_and(|suffix| suffix.eq_ignore_ascii_case(".exe"))
    {
        basename.get(..suffix_start).unwrap_or(basename).to_owned()
    } else {
        basename.to_owned()
    }
}

fn is_preferred(path: &str) -> bool {
    path.replace('/', "\\")
        .to_ascii_lowercase()
        .contains("\\binaries\\win64\\")
}

pub fn choose(procs: &[(u32, String)], stem: &str) -> Option<Pick> {
    let mut last = None;
    let mut last_preferred = None;
    for (pid, path) in procs {
        if !stem_of(path).eq_ignore_ascii_case(stem) {
            continue;
        }
        let pick = Pick {
            pid: *pid,
            preferred: is_preferred(path),
        };
        last = Some(pick);
        if pick.preferred {
            last_preferred = Some(pick);
        }
    }
    last_preferred.or(last)
}

pub struct Waiter {
    stem: String,
    nonpreferred: Option<(u32, u64)>,
}

impl Waiter {
    pub fn new(stem: &str) -> Waiter {
        Waiter {
            stem: stem.to_owned(),
            nonpreferred: None,
        }
    }

    pub fn step(&mut self, procs: &[(u32, String)], now_ms: u64) -> Option<u32> {
        let Some(pick) = choose(procs, &self.stem) else {
            self.nonpreferred = None;
            return None;
        };
        if pick.preferred {
            self.nonpreferred = None;
            return Some(pick.pid);
        }
        match self.nonpreferred {
            Some((pid, first_seen_ms)) if pid == pick.pid => {
                if now_ms.saturating_sub(first_seen_ms) >= FALLBACK_AFTER_S * 1000 {
                    Some(pid)
                } else {
                    None
                }
            }
            _ => {
                self.nonpreferred = Some((pick.pid, now_ms));
                None
            }
        }
    }
}

pub fn has_module(modules: &[String], dll_path: &str) -> bool {
    let wanted = dll_path.rsplit(['\\', '/']).next().unwrap_or(dll_path);
    modules.iter().any(|module| {
        module
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or(module)
            .eq_ignore_ascii_case(wanted)
    })
}
