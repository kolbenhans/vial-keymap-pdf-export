// Data-driven keyboard-layout translation. Every language shipped in this
// repo's languages/ dir is embedded into the binary at compile time — that
// makes it work regardless of how the binary reaches the user (tarball,
// zip, AppImage; `CARGO_MANIFEST_DIR` only exists on the machine that
// *built* the binary, which for release builds is a throwaway CI runner,
// not the end user's machine). On top of that, the directory next to the
// running executable is scanned at runtime for additional/override
// *.json files, so a new language still doesn't need a rebuild — drop a
// file in a `languages/` folder next to the binary, using an existing one
// under `languages/` in this repo as a template.

use qmk_via_api::keycodes::Keycode;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

macro_rules! builtin {
    ($code:literal) => {
        ($code, include_str!(concat!("../languages/", $code, ".json")))
    };
}

const BUILTIN: &[(&str, &str)] = &[
    builtin!("en_US"),
    builtin!("en_GB"),
    builtin!("de_DE"),
    builtin!("de_CH"),
    builtin!("fr_FR"),
    builtin!("fr_BE"),
    builtin!("fr_CH"),
    builtin!("it_IT"),
    builtin!("es_ES"),
    builtin!("nl_NL"),
    builtin!("nl_BE"),
    builtin!("pl_PL"),
    builtin!("ru_RU"),
    builtin!("uk_UA"),
];

#[derive(Deserialize)]
struct LanguageFile {
    name: String,
    #[serde(default)]
    base: HashMap<String, String>,
    #[serde(default)]
    shifted: HashMap<String, String>,
    #[serde(default)]
    altgr: HashMap<String, String>,
    #[serde(default)]
    shift_altgr: HashMap<String, String>,
}

pub struct Language {
    pub code: String,
    pub name: String,
    base: HashMap<String, String>,
    shifted: HashMap<String, String>,
    altgr: HashMap<String, String>,
    shift_altgr: HashMap<String, String>,
}

impl Language {
    fn from_file(code: &str, file: LanguageFile) -> Self {
        Self {
            code: code.to_string(),
            name: file.name,
            base: file.base,
            shifted: file.shifted,
            altgr: file.altgr,
            shift_altgr: file.shift_altgr,
        }
    }

    pub fn base_char(&self, keycode_bytes: u16) -> Option<&str> {
        lookup(&self.base, keycode_bytes)
    }

    pub fn shifted_char(&self, keycode_bytes: u16) -> Option<&str> {
        lookup(&self.shifted, keycode_bytes)
    }

    pub fn altgr_char(&self, keycode_bytes: u16) -> Option<&str> {
        lookup(&self.altgr, keycode_bytes)
    }

    /// Shift+AltGr as its own combo — e.g. Polish Programmers Ę on
    /// Shift+AltGr+E, distinct from plain Shift or plain AltGr on E.
    pub fn shift_altgr_char(&self, keycode_bytes: u16) -> Option<&str> {
        lookup(&self.shift_altgr, keycode_bytes)
    }
}

fn lookup(table: &HashMap<String, String>, keycode_bytes: u16) -> Option<&str> {
    let keycode = Keycode::try_from(keycode_bytes).ok()?;
    table.get(keycode.as_ref()).map(String::as_str)
}

/// `languages/` next to the running executable — not `CARGO_MANIFEST_DIR`,
/// which only makes sense on the machine that built the binary.
fn runtime_languages_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join("languages"))
}

/// `(locale code, display name)` for every built-in language plus any
/// `*.json` next to the executable (a file there with the same code as a
/// built-in overrides it — [`load`] re-reads that file itself in that case).
pub fn list_available() -> Vec<(String, String)> {
    let mut seen: HashMap<String, String> = BUILTIN
        .iter()
        .filter_map(|(code, json)| {
            let name = serde_json::from_str::<LanguageFile>(json).ok()?.name;
            Some((code.to_string(), name))
        })
        .collect();

    if let Some(dir) = runtime_languages_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let Some(code) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(file) = serde_json::from_str::<LanguageFile>(&text) {
                        seen.insert(code.to_string(), file.name);
                    }
                }
            }
        }
    }

    let mut list: Vec<(String, String)> = seen.into_iter().collect();
    list.sort_by(|a, b| a.0.cmp(&b.0));
    list
}

/// Loads a language by locale code: the executable-relative directory
/// first (so it can override a built-in), then the embedded built-ins.
pub fn load(code: &str) -> Result<Language, Box<dyn std::error::Error>> {
    if let Some(dir) = runtime_languages_dir() {
        let path = dir.join(format!("{code}.json"));
        if let Ok(text) = std::fs::read_to_string(&path) {
            let file: LanguageFile = serde_json::from_str(&text)
                .map_err(|e| format!("invalid language file {}: {e}", path.display()))?;
            return Ok(Language::from_file(code, file));
        }
    }

    if let Some((_, json)) = BUILTIN.iter().find(|(c, _)| *c == code) {
        let file: LanguageFile = serde_json::from_str(json)?;
        return Ok(Language::from_file(code, file));
    }

    Err(format!(
        "unknown language '{code}' (not built in, and no {code}.json next to the executable)"
    )
    .into())
}
