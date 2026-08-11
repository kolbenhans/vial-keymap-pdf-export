use crate::layout_geometry::flattened_top_left_after_center_rotation;
use crate::types::{EncoderTile, Key, KeyboardDefinition, KeyboardLayout};
use serde_json::Value;
use std::error::Error;

pub fn parse_vial_definition(
    json: &Value,
    vid: u16,
    pid: u16,
) -> Result<KeyboardDefinition, Box<dyn Error>> {
    let matrix = json
        .get("matrix")
        .ok_or_else(|| Box::<dyn Error>::from("No 'matrix' found in VIAL definition"))?;

    let rows = matrix
        .get("rows")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| Box::<dyn Error>::from("No 'rows' in matrix"))? as usize;

    let cols = matrix
        .get("cols")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| Box::<dyn Error>::from("No 'cols' in matrix"))? as usize;

    let layouts_obj = json
        .get("layouts")
        .ok_or_else(|| Box::<dyn Error>::from("No 'layouts' found in VIAL definition"))?;

    let keymap = layouts_obj
        .get("keymap")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Box::<dyn Error>::from("No 'keymap' array in layouts"))?;

    let (keys, encoders) = parse_kle_keymap(keymap)?;

    let layout = KeyboardLayout {
        name: "default".to_string(),
        keys,
        encoders,
    };

    Ok(KeyboardDefinition {
        vid,
        pid,
        rows,
        cols,
        layouts: vec![layout],
    })
}

struct RawEncoderTile {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    r: f32,
}

fn parse_kle_keymap(keymap: &[Value]) -> Result<(Vec<Key>, Vec<EncoderTile>), Box<dyn Error>> {
    let mut keys = Vec::new();
    let mut raw_encoders = Vec::new();
    let mut current_y: f32 = 0.0;

    // Rotation state (persists across rows until changed)
    let mut rotation_angle: f32 = 0.0;
    let mut rotation_x: f32 = 0.0;
    let mut rotation_y: f32 = 0.0;

    for row in keymap {
        let row_array = match row.as_array() {
            Some(arr) => arr,
            None => continue, // Skip non-array rows
        };

        let mut current_x: f32 = 0.0;
        let mut current_w: f32 = 1.0;
        let mut current_h: f32 = 1.0;

        for item in row_array {
            if let Some(obj) = item.as_object() {
                // Handle rotation properties (these persist until changed)
                // When rx or ry changes, reset the current position to the rotation origin
                let mut origin_changed = false;
                if let Some(rx) = obj.get("rx").and_then(|v| v.as_f64()) {
                    rotation_x = rx as f32;
                    origin_changed = true;
                }
                if let Some(ry) = obj.get("ry").and_then(|v| v.as_f64()) {
                    rotation_y = ry as f32;
                    origin_changed = true;
                }

                if origin_changed {
                    current_x = 0.0;
                    current_y = 0.0;
                }

                if let Some(r) = obj.get("r").and_then(|v| v.as_f64()) {
                    rotation_angle = r as f32;
                }

                if let Some(w) = obj.get("w").and_then(|v| v.as_f64()) {
                    current_w = w as f32;
                }
                if let Some(h) = obj.get("h").and_then(|v| v.as_f64()) {
                    current_h = h as f32;
                }
                if let Some(x) = obj.get("x").and_then(|v| v.as_f64()) {
                    current_x += x as f32;
                }
                if let Some(y) = obj.get("y").and_then(|v| v.as_f64()) {
                    current_y += y as f32;
                }
            } else if let Some(label) = item.as_str() {
                // Normalize KLE-relative coordinates to absolute space, then flatten rotation.
                let absolute_x = rotation_x + current_x;
                let absolute_y = rotation_y + current_y;
                let (final_x, final_y) = flattened_top_left_after_center_rotation(
                    absolute_x,
                    absolute_y,
                    current_w,
                    current_h,
                    rotation_angle,
                    rotation_x,
                    rotation_y,
                );

                if is_encoder_label(label) {
                    raw_encoders.push(RawEncoderTile {
                        x: final_x,
                        y: final_y,
                        w: current_w,
                        h: current_h,
                        r: rotation_angle,
                    });
                } else if let Some((row, col)) = parse_matrix_label(label) {
                    keys.push(Key {
                        row,
                        col,
                        x: final_x,
                        y: final_y,
                        w: current_w,
                        h: current_h,
                        r: rotation_angle,
                    });
                }

                current_x += current_w;

                current_w = 1.0;
                current_h = 1.0;
            }
        }

        current_y += 1.0;
    }

    let encoders = group_encoder_tiles(raw_encoders);

    Ok((keys, encoders))
}

fn group_encoder_tiles(raw: Vec<RawEncoderTile>) -> Vec<EncoderTile> {
    let mut column_xs: Vec<f32> = raw.iter().map(|t| t.x).collect();
    column_xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    column_xs.dedup_by(|a, b| (*a - *b).abs() < 0.01);

    raw.iter()
        .map(|t| {
            let id = column_xs
                .iter()
                .position(|&x| (x - t.x).abs() < 0.01)
                .unwrap_or(0) as u8;
            let is_topmost = raw.iter().all(|o| (o.x - t.x).abs() >= 0.01 || o.y >= t.y);
            EncoderTile {
                id,
                clockwise: is_topmost,
                x: t.x,
                y: t.y,
                w: t.w,
                h: t.h,
                r: t.r,
            }
        })
        .collect()
}

fn is_encoder_label(label: &str) -> bool {
    label.lines().last() == Some("e")
}

fn parse_matrix_label(label: &str) -> Option<(usize, usize)> {
    let first_line = label.lines().next().unwrap_or(label);

    let parts: Vec<&str> = first_line.split(',').collect();
    if parts.len() >= 2 {
        let row = parts[0].trim().parse::<usize>().ok()?;
        let col = parts[1].trim().parse::<usize>().ok()?;
        return Some((row, col));
    }

    None
}
