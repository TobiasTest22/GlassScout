use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::fs;

use crate::graphics::{
    faces::{image_mime, resolve_face_path, MAX_IMAGE_BYTES},
    logos::resolve_logo_path,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerFaceResult {
    found: bool,
    player_id: String,
    data_url: Option<String>,
    source: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClubLogoResult {
    found: bool,
    club_id: String,
    data_url: Option<String>,
}

#[tauri::command]
pub fn player_face_data(player_id: String, icon: bool) -> PlayerFaceResult {
    let player_id = player_id.trim().to_string();
    if player_id.is_empty() || !player_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return missing(player_id);
    }

    let Some(path) = resolve_face_path(&player_id, icon) else {
        return missing(player_id);
    };
    let Ok(metadata) = path.metadata() else {
        return missing(player_id);
    };
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_IMAGE_BYTES {
        return missing(player_id);
    }
    let Some(mime) = image_mime(&path) else {
        return missing(player_id);
    };
    let Ok(bytes) = fs::read(path) else {
        return missing(player_id);
    };

    PlayerFaceResult {
        found: true,
        player_id,
        data_url: Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes))),
        source: "fm-unique-id",
    }
}

#[tauri::command]
pub fn club_logo_data(club_id: String) -> ClubLogoResult {
    let club_id = club_id.trim().to_string();
    if club_id.is_empty() || !club_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return ClubLogoResult {
            found: false,
            club_id,
            data_url: None,
        };
    }
    if let Some(path) = resolve_logo_path(&club_id) {
        if let (Some(mime), Ok(bytes)) = (image_mime(&path), fs::read(path)) {
            return ClubLogoResult {
                found: true,
                club_id,
                data_url: Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes))),
            };
        }
    }
    ClubLogoResult {
        found: false,
        club_id,
        data_url: None,
    }
}

fn missing(player_id: String) -> PlayerFaceResult {
    PlayerFaceResult {
        found: false,
        player_id,
        data_url: None,
        source: "fallback",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn player_face_command_rejects_non_numeric_ids() {
        let result = super::player_face_data("../bad".to_string(), false);
        assert!(!result.found);
        assert_eq!(result.source, "fallback");
    }
}
