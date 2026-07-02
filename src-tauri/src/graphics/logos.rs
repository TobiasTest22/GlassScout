use std::{fs, path::Path};

use super::{
    config_xml::{attribute_value, safe_single_component_stem, MAX_GRAPHICS_CONFIG_BYTES},
    faces::{graphics_roots, image_mime, MAX_IMAGE_BYTES},
};

pub(crate) fn resolve_logo_path(club_id: &str) -> Option<std::path::PathBuf> {
    for root in graphics_roots() {
        for directory in [root.join("logos"), root.join("clubs"), root.join("badges")] {
            for stem in [
                club_id.to_string(),
                format!("club_{club_id}"),
                format!("logo_{club_id}"),
            ] {
                for extension in ["png", "jpg", "jpeg", "webp"] {
                    let path = directory.join(format!("{stem}.{extension}"));
                    if valid_image_file(&path) {
                        return Some(path);
                    }
                }
            }
            let config = directory.join("config.xml");
            if let Some(path) = resolve_logo_from_config(&config, &directory, club_id) {
                return Some(path);
            }
        }
    }
    None
}

fn valid_image_file(path: &Path) -> bool {
    image_mime(path).is_some()
        && path.metadata().is_ok_and(|metadata| {
            metadata.is_file() && metadata.len() > 0 && metadata.len() <= MAX_IMAGE_BYTES
        })
}

fn resolve_logo_from_config(
    config: &Path,
    directory: &Path,
    club_id: &str,
) -> Option<std::path::PathBuf> {
    let metadata = config.metadata().ok()?;
    if metadata.len() > MAX_GRAPHICS_CONFIG_BYTES {
        return None;
    }
    let content = fs::read_to_string(config).ok()?;
    for marker in [
        format!("team/{club_id}/logo"),
        format!("club/{club_id}/logo"),
    ] {
        for line in content.lines().filter(|line| line.contains(&marker)) {
            let from = safe_single_component_stem(attribute_value(line, "from")?)?;
            for extension in ["png", "jpg", "jpeg", "webp"] {
                let path = directory.join(format!("{from}.{extension}"));
                if valid_image_file(&path) {
                    return Some(path);
                }
            }
        }
    }
    None
}
