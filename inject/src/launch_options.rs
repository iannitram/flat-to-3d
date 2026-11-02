//! The launch-option wrapper rule. docs/code-map.md, Interlinks, the wrapper rule.

pub const WRAPPER: &str = "flat3d-run";

const COMMAND: &str = "%command%";

fn words(options: &str) -> Vec<&str> {
    options.split_whitespace().collect()
}

fn is_wrapper(word: &str) -> bool {
    word.rsplit('/').next() == Some(WRAPPER)
}

fn is_env_word(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn command_index(words: &[&str]) -> Option<usize> {
    words.iter().position(|w| *w == COMMAND)
}

pub fn has_wrapper(options: &str) -> bool {
    let words = words(options);
    let end = command_index(&words).unwrap_or(words.len());
    words[..end].iter().any(|w| is_wrapper(w))
}

pub fn add_wrapper(options: &str) -> String {
    if has_wrapper(options) {
        return options.to_string();
    }
    let mut words = words(options);
    let Some(command) = command_index(&words) else {
        let mut out = vec![WRAPPER, COMMAND];
        out.append(&mut words);
        return out.join(" ");
    };
    let at = words[..command]
        .iter()
        .position(|w| !is_env_word(w))
        .unwrap_or(command);
    words.insert(at, WRAPPER);
    words.join(" ")
}

pub fn remove_wrapper(options: &str) -> String {
    if !has_wrapper(options) {
        return options.to_string();
    }
    let words = words(options);
    let end = command_index(&words).unwrap_or(words.len());
    let kept: Vec<&str> = words
        .iter()
        .enumerate()
        .filter(|(i, w)| !(*i < end && is_wrapper(w)))
        .map(|(_, w)| *w)
        .collect();
    if kept.len() == 1 && kept[0] == COMMAND {
        return String::new();
    }
    if kept.len() > 1 && kept[0] == COMMAND {
        return kept[1..].join(" ");
    }
    kept.join(" ")
}
