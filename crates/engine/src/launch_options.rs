//! The launch-option wrapper rule. docs/code-map.md, Interlinks, the wrapper rule.

pub const WRAPPER: &str = "flat3d-run";

const COMMAND: &str = "%command%";

/// The words of `options`, split on whitespace outside double quotes. A quoted value such as
/// `NAME="a b"` is one word, so the wrapper is never put inside it and its spaces survive.
/// When `%command%` is only inside quotes (`sh -c "x %command% y"`), it is not a word of its own,
/// so the plain whitespace split is used instead and the command stays visible.
fn words(options: &str) -> Vec<&str> {
    let quoted_split = quote_split(options);
    if !quoted_split.contains(&COMMAND) && options.contains(COMMAND) {
        return options.split_whitespace().collect();
    }
    quoted_split
}

fn quote_split(options: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = None;
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in options.char_indices() {
        // Inside double quotes a backslash escapes the next character, so `\"` does not close the quote.
        if escaped {
            escaped = false;
        } else if c == '\\' && quoted {
            escaped = true;
        } else if c == '"' {
            quoted = !quoted;
        }
        if c.is_whitespace() && !quoted {
            if let Some(from) = start.take() {
                out.push(&options[from..i]);
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if quoted {
        // An unclosed quote is broken input: split it as before rather than let it hide the command.
        return options.split_whitespace().collect();
    }
    if let Some(from) = start {
        out.push(&options[from..]);
    }
    out
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

pub fn add_wrapper_at(options: &str, wrapper_abs: &str) -> String {
    if wrapper_abs.chars().any(char::is_whitespace)
        || wrapper_abs.contains('"')
        || wrapper_abs.rsplit('/').next() != Some(WRAPPER)
    {
        return add_wrapper(options);
    }
    let mut words = words(options);
    if words
        .iter()
        .any(|word| word.contains(COMMAND) && *word != COMMAND)
    {
        words = words
            .into_iter()
            .map(|word| word.trim_matches('"'))
            .collect();
    }
    if has_wrapper(options) {
        let end = command_index(&words).unwrap_or(words.len());
        if let Some(first) = words[..end].iter().position(|word| is_wrapper(word)) {
            words[first] = wrapper_abs;
            return words
                .into_iter()
                .enumerate()
                .filter_map(|(index, word)| {
                    if index >= end || index == first {
                        Some(word)
                    } else if is_wrapper(word) {
                        None
                    } else {
                        Some(word)
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
    }
    let Some(command) = command_index(&words) else {
        let mut out = vec![wrapper_abs, COMMAND];
        out.append(&mut words);
        return out.join(" ");
    };
    let at = words[..command]
        .iter()
        .position(|w| !is_env_word(w))
        .unwrap_or(command);
    words.insert(at, wrapper_abs);
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
