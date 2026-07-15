pub mod ast;
pub mod html;
pub mod lexer;
pub mod parser;

#[cfg(target_arch = "wasm32")]
mod wasm;

use std::collections::HashMap;

/// Logs a warning to stderr on native targets, or to the browser console on wasm32.
pub(crate) fn warn_log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::warn_1(&msg.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{}", msg);
    }
}

/// Parses `KEY=VALUE` (or bare `KEY`, treated as `KEY=true`) entries into the
/// define map consumed by the parser for conditional rendering. Shared between
/// the CLI's repeated `-d` flags and the wasm frontend's define list.
pub fn parse_defines<I, S>(entries: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut defines_map = HashMap::new();
    for entry in entries {
        let entry = entry.as_ref().trim();
        if entry.is_empty() {
            continue;
        }
        if let Some(pos) = entry.find('=') {
            let (key, value) = entry.split_at(pos);
            defines_map.insert(key.trim().to_string(), value[1..].trim().to_string());
        } else {
            defines_map.insert(entry.to_string(), "true".to_string());
        }
    }
    defines_map
}
