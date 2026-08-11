// Data-driven replacement for the old per-locale .rs match tables: every
// language lives entirely in a JSON file under `languages/` (repo root, next
// to `src/`), read fresh at every run. Adding a language needs a new file
// there and nothing else — no `mod` declaration, no registry entry, no
// rebuild.

use qmk_via_api::keycodes::Keycode;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
struct LanguageFile {
    name: String,
    #[serde(default)]
    base: HashMap<String, String>,
    #[serde(default)]
    shifted: HashMap<String, String>,
    #[serde(default)]
    altgr: HashMap<String, String>,
}

pub struct Language {
    pub code: String,
    pub name: String,
    base: HashMap<String, String>,
    shifted: HashMap<String, String>,
    altgr: HashMap<String, String>,
}

impl Language {
    pub fn base_char(&self, keycode_bytes: u16) -> Option<&str> {
        lookup(&self.base, keycode_bytes)
    }

    pub fn shifted_char(&self, keycode_bytes: u16) -> Option<&str> {
        lookup(&self.shifted, keycode_bytes)
    }

    pub fn altgr_char(&self, keycode_bytes: u16) -> Option<&str> {
        lookup(&self.altgr, keycode_bytes)
    }
}

fn lookup(table: &HashMap<String, String>, keycode_bytes: u16) -> Option<&str> {
    let keycode = Keycode::try_from(keycode_bytes).ok()?;
    table.get(keycode.as_ref()).map(String::as_str)
}

/// `<repo>/languages` — resolved at compile time (via `CARGO_MANIFEST_DIR`)
/// but *read* fresh on every run, so new files there don't need a rebuild.
fn languages_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("languages")
}

/// Locale codes for every `<code>.json` file found in `languages/`.
pub fn list_available() -> Vec<String> {
    let mut codes: Vec<String> = std::fs::read_dir(languages_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()? == "json")
                .then(|| path.file_stem()?.to_str().map(str::to_string))
                .flatten()
        })
        .collect();
    codes.sort();
    codes
}

pub fn load(code: &str) -> Result<Language, Box<dyn std::error::Error>> {
    let path = languages_dir().join(format!("{code}.json"));
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("can't read language file {}: {e}", path.display()))?;
    let file: LanguageFile = serde_json::from_str(&text)
        .map_err(|e| format!("invalid language file {}: {e}", path.display()))?;
    Ok(Language {
        code: code.to_string(),
        name: file.name,
        base: file.base,
        shifted: file.shifted,
        altgr: file.altgr,
    })
}
