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
    let combo_keycode = key.shift_altgr_base.or(key.shift_base).or(key.altgr_base);

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
    let shift_altgr_resolved = key
        .shift_altgr_base
        .and_then(|kc| language.shift_altgr_char(kc))
        .map(str::to_string);
    let altgr_resolved = key.altgr_base.and_then(|kc| language.altgr_char(kc)).map(str::to_string);
    let shift_resolved = key.shift_base.and_then(|kc| {
        language
            .shifted_char(kc)
            .map(str::to_string)
            .or_else(|| crate::keycodes::keycode_label::get_layout_key(kc).and_then(|k| k.shifted))
    });
    shift_altgr_resolved.or(altgr_resolved).or(shift_resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S(A(KC_E)) → Shift+AltGr+E should resolve to Polish "Ę", not fall
    /// through to plain "E" or get misfiled as lone-shift/lone-altgr.
    #[test]
    fn shift_altgr_combo_resolves_via_language_table() {
        // (mod_mask=LSFT|LALT|RIGHT_FLAG=0x16) << 8 | KC_E(0x08). QK_MODS's
        // bits 8-12 *are* the mod flags (LCTL=bit8) — no separate marker.
        let shift_altgr_e: u16 = (0x16 << 8) | 0x08;
        let key = crate::keycodes::keycode_label::get_layout_key(shift_altgr_e)
            .expect("mod-combo keycode should resolve to a LayoutKey");
        assert_eq!(key.shift_altgr_base, Some(0x08));

        let pl = crate::languages::load("pl_PL").expect("languages/pl_PL.json should load");
        let resolved = resolve_label(&key, &pl);
        assert_eq!(resolved.full, "Ę");
    }
}
