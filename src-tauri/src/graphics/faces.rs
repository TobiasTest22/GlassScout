use std::{
    env, fs,
    path::{Path, PathBuf},
};

use super::config_xml::{attribute_value, safe_single_component_stem, MAX_GRAPHICS_CONFIG_BYTES};

pub(crate) const MAX_IMAGE_BYTES: u64 = 8 * 1024 * 1024;

pub(crate) fn resolve_face_path(player_id: &str, icon: bool) -> Option<PathBuf> {
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
        if let Some(path) = resolve_face_from_config(&config, &directory, player_id) {
            return Some(path);
        }
    }
    None
}

pub(crate) fn graphics_roots() -> Vec<PathBuf> {
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

pub(crate) fn image_mime(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn resolve_face_from_config(config: &Path, directory: &Path, player_id: &str) -> Option<PathBuf> {
    let metadata = config.metadata().ok()?;
    if metadata.len() > MAX_GRAPHICS_CONFIG_BYTES {
        return None;
    }
    let content = fs::read_to_string(config).ok()?;
    let marker = format!("person/{player_id}/");
    for line in content.lines().filter(|line| line.contains(&marker)) {
        let from = safe_single_component_stem(attribute_value(line, "from")?)?;
        for extension in ["png", "jpg", "jpeg", "webp"] {
            let path = directory.join(format!("{from}.{extension}"));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_supported_image_types_receive_a_mime() {
        assert_eq!(image_mime(Path::new("face_1.png")), Some("image/png"));
        assert_eq!(image_mime(Path::new("face_1.svg")), None);
    }
}
