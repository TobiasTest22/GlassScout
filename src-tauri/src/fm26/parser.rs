use std::collections::HashMap;

use super::structs::{HIDDEN_ATTRIBUTE_INDEXES, PLAYER_ATTRIBUTE_NAMES};

pub(crate) fn display_attribute(raw: u8) -> u8 {
    ((raw.saturating_add(4)) / 5).clamp(1, 20)
}

pub(crate) fn visible_attribute_map(raw: &[u8]) -> HashMap<String, u8> {
    PLAYER_ATTRIBUTE_NAMES
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            !HIDDEN_ATTRIBUTE_INDEXES.contains(index) && !matches!(*index, 24 | 25)
        })
        .filter_map(|(index, name)| {
            raw.get(index)
                .copied()
                .map(|value| ((*name).to_string(), display_attribute(value)))
        })
        .collect()
}

pub(crate) fn preferred_foot_label(left: u8, right: u8) -> &'static str {
    let difference = i16::from(left) - i16::from(right);
    if difference >= 20 {
        "Left"
    } else if difference <= -20 {
        "Right"
    } else {
        "Both"
    }
}

pub(crate) fn default_role_for_position(position: &str) -> String {
    match position {
        "GK" => "Goalkeeper",
        "SW" | "DC" => "Central Defender",
        "DL" | "DR" | "WBL" | "WBR" => "Full-Back",
        "DM" => "Defensive Midfielder",
        "ML" | "MR" | "AML" | "AMR" => "Winger",
        "MC" => "Central Midfielder",
        "AMC" => "Attacking Midfielder",
        "ST" => "Advanced Forward",
        _ => "Natural Position",
    }
    .to_string()
}
