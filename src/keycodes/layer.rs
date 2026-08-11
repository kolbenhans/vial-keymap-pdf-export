use crate::layout_key::{behavior_names, BorderStyle, Label, LayoutKey};
use crate::keycodes::constants::*;

pub fn get_layer_layout_key(keycode_bytes: u16) -> Option<LayoutKey> {
    // Layer-switch keys are shown by their border alone (Solid = persists, Dashed =
    // sticky/one-shot, None = momentary) and carry no legend strip. The non-layer
    // behaviors here (tap dance / macro / custom) keep a name strip and default border.
    let (behavior, center, layer_ref, border) = match keycode_bytes {
        b if QK_TO.contains(&b) => {
            let l = (b - QK_TO.start) as u8;
            (None, format!("L{}", l), Some(l), BorderStyle::Solid)
        }
        b if QK_MOMENTARY.contains(&b) => {
            let l = (b - QK_MOMENTARY.start) as u8;
            (None, format!("L{}", l), Some(l), BorderStyle::None)
        }
        b if QK_TOGGLE_LAYER.contains(&b) => {
            let l = (b - QK_TOGGLE_LAYER.start) as u8;
            (None, format!("L{}", l), Some(l), BorderStyle::Solid)
        }
        b if QK_ONE_SHOT_LAYER.contains(&b) => {
            let l = (b - QK_ONE_SHOT_LAYER.start) as u8;
            (None, format!("L{}", l), Some(l), BorderStyle::Dashed)
        }
        b if QK_LAYER_TAP_TOGGLE.contains(&b) => {
            let l = (b - QK_LAYER_TAP_TOGGLE.start) as u8;
            (None, format!("L{}", l), Some(l), BorderStyle::None)
        }
        b if QK_DEF_LAYER.contains(&b) => {
            let l = (b - QK_DEF_LAYER.start) as u8;
            (None, format!("L{}", l), None, BorderStyle::Solid)
        }
        b if QK_TAP_DANCE.contains(&b) => {
            let n = b - QK_TAP_DANCE.start;
            (
                Some(behavior_names::TAP_DANCE.label()),
                n.to_string(),
                None,
                BorderStyle::None,
            )
        }
        b if QK_KB.contains(&b) => {
            let n = b - QK_KB.start;
            (
                Some(behavior_names::CUSTOM.label()),
                n.to_string(),
                None,
                BorderStyle::None,
            )
        }
        b if QK_MACRO.contains(&b) => {
            let n = b - QK_MACRO.start;
            (
                Some(behavior_names::MACRO.label()),
                n.to_string(),
                None,
                BorderStyle::None,
            )
        }
        _ => return None,
    };

    Some(LayoutKey {
        tap: Label::new(center),
        behavior,
        layer_ref,
        border,
        ..Default::default()
    })
}
