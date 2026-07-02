use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const MAX_FACE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerFaceResult {
    found: bool,
    player_id: String,
    data_url: Option<String>,
    source: &'static str,
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
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_FACE_BYTES {
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

fn missing(player_id: String) -> PlayerFaceResult {
    PlayerFaceResult {
        found: false,
        player_id,
        data_url: None,
        source: "fallback",
    }
}

fn image_mime(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn resolve_face_path(player_id: &str, icon: bool) -> Option<PathBuf> {
    let folder = if icon { "iconfaces" } else { "faces" };
    for root in graphics_roots() {
        let directory = root.join(folder);
        for stem in [
            format!("face_{player_id}"),
            player_id.to_string(),
            format!("iconface_{player_id}"),
        ] {
            for extension in ["png", "jpg", "jpeg", "webp"] {
                let path = directory.join(format!("{stem}.{extension}"));
                if path.is_file() {
                    return Some(path);
                }
            }
        }

        let config = directory.join("config.xml");
        if let Some(path) = resolve_from_config(&config, &directory, player_id) {
            return Some(path);
        }
    }
    None
}

fn graphics_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = env::var_os("USERPROFILE").map(PathBuf::from) {
        roots.push(
            home.join("Documents")
                .join("Sports Interactive")
                .join("Football Manager 26")
                .join("graphics"),
        );
        roots.push(
            home.join("OneDrive")
                .join("Documents")
                .join("Sports Interactive")
                .join("Football Manager 26")
                .join("graphics"),
        );
    }
    roots
}

fn resolve_from_config(config: &Path, directory: &Path, player_id: &str) -> Option<PathBuf> {
    let metadata = config.metadata().ok()?;
    if metadata.len() > 32 * 1024 * 1024 {
        return None;
    }
    let content = fs::read_to_string(config).ok()?;
    let marker = format!("person/{player_id}/");
    for line in content.lines().filter(|line| line.contains(&marker)) {
        let from = attribute_value(line, "from")?;
        let relative = Path::new(from);
        if relative.components().count() != 1 {
            continue;
        }
        for extension in ["png", "jpg", "jpeg", "webp"] {
            let path = directory.join(format!("{from}.{extension}"));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let start_marker = format!("{name}=\"");
    let start = line.find(&start_marker)? + start_marker.len();
    let end = line[start..].find('"')? + start;
    line.get(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_attribute_is_read_without_accepting_paths() {
        let line =
            r#"<record from="face_2000370823" to="graphics/pictures/person/2000370823/portrait"/>"#;
        assert_eq!(attribute_value(line, "from"), Some("face_2000370823"));
    }

    #[test]
    fn only_supported_image_types_receive_a_mime() {
        assert_eq!(image_mime(Path::new("face_1.png")), Some("image/png"));
        assert_eq!(image_mime(Path::new("face_1.svg")), None);
    }
}
