use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{
    config_xml::{attribute_value, safe_relative_asset_path, MAX_GRAPHICS_CONFIG_BYTES},
    faces::{graphics_roots, image_mime, MAX_IMAGE_BYTES},
};

const MAX_LOGO_CONFIGS_SCANNED: usize = 256;
const MAX_LOGO_CONFIG_DEPTH: usize = 10;

pub(crate) fn resolve_logo_path(club_id: &str) -> Option<std::path::PathBuf> {
    for root in graphics_roots() {
        for directory in [
            root.join("logos"),
            root.join("clubs"),
            root.join("badges"),
            root.join("pictures").join("club"),
        ] {
            if let Some(path) = resolve_logo_direct(&directory, club_id) {
                return Some(path);
            }
            if let Some(path) = resolve_logo_from_configs(&directory, club_id) {
                return Some(path);
            }
        }
    }
    None
}

fn resolve_logo_direct(directory: &Path, club_id: &str) -> Option<PathBuf> {
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
    None
}

fn resolve_logo_from_configs(directory: &Path, club_id: &str) -> Option<PathBuf> {
    if !directory.is_dir() {
        return None;
    }

    let mut stack = vec![(directory.to_path_buf(), 0usize)];
    let mut configs_scanned = 0usize;
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_LOGO_CONFIG_DEPTH || configs_scanned >= MAX_LOGO_CONFIGS_SCANNED {
            continue;
        }

        let config = current.join("config.xml");
        if config.is_file() {
            configs_scanned += 1;
            if let Some(path) = resolve_logo_from_config(&config, &current, club_id) {
                return Some(path);
            }
            continue;
        }

        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push((path, depth + 1));
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
        format!("graphics/pictures/club/{club_id}/logo"),
        format!("graphics/pictures/team/{club_id}/logo"),
        format!("team/{club_id}/logo"),
        format!("club/{club_id}/logo"),
        format!("graphics/pictures/club/{club_id}/icon"),
        format!("graphics/pictures/team/{club_id}/icon"),
        format!("team/{club_id}/icon"),
        format!("club/{club_id}/icon"),
    ] {
        for line in content.lines().filter(|line| line.contains(&marker)) {
            let from = safe_relative_asset_path(attribute_value(line, "from")?)?;
            let direct = directory.join(&from);
            if valid_image_file(&direct) {
                return Some(direct);
            }
            if from.extension().is_some() {
                continue;
            }
            for extension in ["png", "jpg", "jpeg", "webp"] {
                let path = directory.join(from.with_extension(extension));
                if valid_image_file(&path) {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_logo_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("glassscout-logo-test-{stamp}"))
    }

    #[test]
    fn resolves_fm_logo_pack_graphics_picture_club_mapping() {
        let root = temp_logo_dir();
        fs::create_dir_all(&root).expect("temp logo folder");
        fs::write(root.join("13172353.png"), b"not-empty").expect("test image");
        fs::write(
            root.join("config.xml"),
            r#"<record from="13172353" to="graphics/pictures/club/13172353/logo"/>"#,
        )
        .expect("test config");

        let path = resolve_logo_from_config(&root.join("config.xml"), &root, "13172353")
            .expect("club logo mapping");
        assert_eq!(path, root.join("13172353.png"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_unsafe_logo_config_paths() {
        let root = temp_logo_dir();
        fs::create_dir_all(&root).expect("temp logo folder");
        fs::write(
            root.join("config.xml"),
            r#"<record from="../outside" to="graphics/pictures/club/1/logo"/>"#,
        )
        .expect("test config");

        assert!(resolve_logo_from_config(&root.join("config.xml"), &root, "1").is_none());

        let _ = fs::remove_dir_all(root);
    }
}
