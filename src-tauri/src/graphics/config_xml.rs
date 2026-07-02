use std::path::{Component, Path, PathBuf};

pub(crate) const MAX_GRAPHICS_CONFIG_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let start_marker = format!("{name}=\"");
    let start = line.find(&start_marker)? + start_marker.len();
    let end = line[start..].find('"')? + start;
    line.get(start..end)
}

pub(crate) fn safe_single_component_stem(value: &str) -> Option<&str> {
    let relative = Path::new(value);
    (relative.components().count() == 1).then_some(value)
}

pub(crate) fn safe_relative_asset_path(value: &str) -> Option<PathBuf> {
    let mut output = PathBuf::new();
    let mut has_component = false;
    for component in Path::new(value).components() {
        match component {
            Component::Normal(part) => {
                output.push(part);
                has_component = true;
            }
            _ => return None,
        }
    }
    has_component.then_some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_attribute_is_read_without_accepting_paths() {
        let line =
            r#"<record from="face_2000370823" to="graphics/pictures/person/2000370823/portrait"/>"#;
        assert_eq!(attribute_value(line, "from"), Some("face_2000370823"));
        assert_eq!(safe_single_component_stem("../bad"), None);
        assert_eq!(
            safe_relative_asset_path("clubs/normal/2000370823").as_deref(),
            Some(Path::new("clubs/normal/2000370823"))
        );
        assert_eq!(safe_relative_asset_path("../bad"), None);
    }
}
