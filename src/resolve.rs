// Trimmed from keypeek's overlay_window/ui_overlay.rs (generate_tap_galleys /
// resolve_combo) — reproduces "Single" legend mode for a static print instead
// of a live overlay: no shift/altgr-held branches (nothing is being held),
// no Dual-mode stacking, no icon-font symbol handling (dropped upstream in
// layout_key.rs/keycodes/basic.rs already).

use crate::languages::Language;
use crate::layout_key::LayoutKey;

/// Main label text plus an optional shorter fallback for when `full` doesn't
/// fit the keycap box.
pub struct ResolvedLabel {
    pub full: String,
    pub short: Option<String>,
}

/// Final single-line label for a key under `language`, mirroring keypeek's
/// "Single" legend mode: a key programmed as e.g. `S(KC_8)` resolves straight
/// to "(" on German, a plain `KC_Y` resolves to "Z" (physical position swap),
/// anything without a language-specific mapping falls back to the raw US
/// label (English's own JSON tables are simply empty).
pub fn resolve_label(key: &LayoutKey, language: &Language) -> ResolvedLabel {
    let combo_keycode = key.shift_base.or(key.altgr_base);

    let raw_base_label = || {
        combo_keycode
            .and_then(|kc| crate::keycodes::keycode_label::get_layout_key(kc))
            .map(|k| k.tap.full)
            .unwrap_or_else(|| key.tap.full.clone())
    };

    let tap_full = key
        .base_keycode
        .and_then(|kc| language.base_char(kc))
        .map(str::to_string)
        .unwrap_or_else(|| {
            if combo_keycode.is_some() {
                raw_base_label()
            } else {
                key.tap.full.clone()
            }
        });

    match resolve_combo(key, language) {
        // A resolved combo (e.g. "(" for S(KC_8)) is always a single
        // character — no shorter fallback needed.
        Some(ch) => ResolvedLabel {
            full: ch,
            short: None,
        },
        None => ResolvedLabel {
            full: tap_full,
            short: key.tap.short.clone(),
        },
    }
}

fn resolve_combo(key: &LayoutKey, language: &Language) -> Option<String> {
    let altgr_resolved = key.altgr_base.and_then(|kc| language.altgr_char(kc)).map(str::to_string);
    let shift_resolved = key.shift_base.and_then(|kc| {
        language
            .shifted_char(kc)
            .map(str::to_string)
            .or_else(|| crate::keycodes::keycode_label::get_layout_key(kc).and_then(|k| k.shifted))
    });
    altgr_resolved.or(shift_resolved)
}
