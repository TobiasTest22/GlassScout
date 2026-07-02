use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;

use crate::connector::{capture_mapping_lab_player, MappingLabCaptureData};

const MODE_VARIABLE: &str = "GLASSSCOUT_MAPPING_MODE";
const MAX_WINDOW_BYTES: usize = 4096;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingLabStatus {
    enabled: bool,
    read_only: bool,
    maximum_window_bytes: usize,
    evidence_directory: Option<String>,
    message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceWindow {
    object: String,
    base_address: String,
    byte_length: usize,
    bytes_base64: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceSnapshot {
    schema_version: u32,
    captured_at: String,
    label: String,
    player_id: String,
    player_name: String,
    process_id: u32,
    save_pointer: String,
    entity_map_profile_id: String,
    executable_sha256: String,
    read_only: bool,
    windows: Vec<EvidenceWindow>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResult {
    success: bool,
    snapshot_id: String,
    evidence_file: String,
    player_id: String,
    player_name: String,
    windows_captured: usize,
    bytes_captured: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DifferenceRange {
    object: String,
    start_offset: usize,
    end_offset: usize,
    before_hex: String,
    after_hex: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonResult {
    success: bool,
    first_snapshot_id: String,
    second_snapshot_id: String,
    changed_bytes: usize,
    unchanged_bytes: usize,
    differences: Vec<DifferenceRange>,
    candidate_offsets: Vec<CandidateOffset>,
    evidence_file: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateOffset {
    object: String,
    offset: usize,
    width: usize,
    score: f64,
    rationale: String,
}

fn enabled() -> bool {
    std::env::var(MODE_VARIABLE).as_deref() == Ok("1")
}

fn lab_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("The local app-data directory is unavailable: {error}"))?
        .join("mapping-lab");
    fs::create_dir_all(&root).map_err(|error| {
        format!("The local mapping evidence directory could not be created: {error}")
    })?;
    Ok(root)
}

fn require_mode(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if !enabled() {
        return Err(format!(
            "Mapping Lab is disabled. Set {MODE_VARIABLE}=1 before launching the development build."
        ));
    }
    lab_directory(app)
}

fn safe_label(value: &str) -> String {
    let clean: String = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(120)
        .collect();
    if clean.is_empty() {
        "snapshot".to_string()
    } else {
        clean
    }
}

fn snapshot_path(directory: &Path, id: &str) -> Result<PathBuf, String> {
    let safe = safe_label(id);
    if safe != id {
        return Err(
            "Snapshot identifiers may contain only letters, digits, hyphens and underscores."
                .to_string(),
        );
    }
    Ok(directory.join(format!("{safe}.json")))
}

fn timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn to_snapshot(data: MappingLabCaptureData, label: String) -> EvidenceSnapshot {
    EvidenceSnapshot {
        schema_version: 1,
        captured_at: timestamp(),
        label,
        player_id: data.player_id,
        player_name: data.player_name,
        process_id: data.process_id,
        save_pointer: data.save_pointer,
        entity_map_profile_id: data.entity_map_profile_id,
        executable_sha256: data.executable_sha256,
        read_only: true,
        windows: data
            .windows
            .into_iter()
            .map(|window| EvidenceWindow {
                object: window.object.to_string(),
                base_address: window.base_address,
                byte_length: window.bytes.len(),
                bytes_base64: STANDARD.encode(window.bytes),
            })
            .collect(),
    }
}

#[tauri::command]
pub fn mapping_lab_status(app: tauri::AppHandle) -> MappingLabStatus {
    let mode_enabled = enabled();
    let directory = mode_enabled
        .then(|| lab_directory(&app).ok())
        .flatten()
        .map(|path| path.display().to_string());
    MappingLabStatus {
        enabled: mode_enabled,
        read_only: true,
        maximum_window_bytes: MAX_WINDOW_BYTES,
        evidence_directory: directory,
        message: if mode_enabled {
            "Developer mapping evidence capture is enabled. Reads are bounded and local."
                .to_string()
        } else {
            "Developer mapping evidence capture is disabled in normal builds.".to_string()
        },
    }
}

#[tauri::command]
pub fn mapping_lab_capture(
    app: tauri::AppHandle,
    player_id: String,
    label: String,
    window_size: Option<usize>,
) -> Result<CaptureResult, String> {
    let directory = require_mode(&app)?;
    let size = window_size.unwrap_or(1024).clamp(64, MAX_WINDOW_BYTES);
    let snapshot = to_snapshot(
        capture_mapping_lab_player(player_id.trim(), size)?,
        label.trim().to_string(),
    );
    let snapshot_id = format!(
        "{}-{}-{}",
        snapshot.captured_at,
        safe_label(&snapshot.player_id),
        safe_label(&snapshot.label)
    );
    let path = snapshot_path(&directory, &snapshot_id)?;
    let bytes_captured = snapshot
        .windows
        .iter()
        .map(|window| window.byte_length)
        .sum();
    let windows_captured = snapshot.windows.len();
    let player_name = snapshot.player_name.clone();
    let player_id = snapshot.player_id.clone();
    let json = serde_json::to_vec_pretty(&snapshot)
        .map_err(|error| format!("Mapping evidence could not be encoded: {error}"))?;
    fs::write(&path, json)
        .map_err(|error| format!("Mapping evidence could not be stored locally: {error}"))?;
    Ok(CaptureResult {
        success: true,
        snapshot_id,
        evidence_file: path.display().to_string(),
        player_id,
        player_name,
        windows_captured,
        bytes_captured,
    })
}

fn load_snapshot(directory: &Path, id: &str) -> Result<EvidenceSnapshot, String> {
    let path = snapshot_path(directory, id)?;
    let bytes =
        fs::read(&path).map_err(|error| format!("Snapshot {id} could not be read: {error}"))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("Snapshot {id} is not valid mapping evidence: {error}"))
}

fn difference_ranges(
    before: &EvidenceSnapshot,
    after: &EvidenceSnapshot,
) -> Result<(usize, usize, Vec<DifferenceRange>), String> {
    if before.entity_map_profile_id != after.entity_map_profile_id
        || before.executable_sha256 != after.executable_sha256
    {
        return Err(
            "Snapshots from different FM26 build fingerprints cannot be compared.".to_string(),
        );
    }
    let mut changed = 0;
    let mut unchanged = 0;
    let mut ranges = Vec::new();
    for first in &before.windows {
        let Some(second) = after
            .windows
            .iter()
            .find(|window| window.object == first.object)
        else {
            continue;
        };
        let left = STANDARD
            .decode(&first.bytes_base64)
            .map_err(|_| "A snapshot contains invalid binary evidence.".to_string())?;
        let right = STANDARD
            .decode(&second.bytes_base64)
            .map_err(|_| "A snapshot contains invalid binary evidence.".to_string())?;
        let length = left.len().min(right.len());
        let mut offset = 0;
        while offset < length {
            if left[offset] == right[offset] {
                unchanged += 1;
                offset += 1;
                continue;
            }
            let start = offset;
            while offset < length && left[offset] != right[offset] {
                changed += 1;
                offset += 1;
            }
            let end = offset;
            ranges.push(DifferenceRange {
                object: first.object.clone(),
                start_offset: start,
                end_offset: end,
                before_hex: left[start..end]
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect(),
                after_hex: right[start..end]
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect(),
            });
        }
    }
    Ok((changed, unchanged, ranges))
}

fn score_candidates(differences: &[DifferenceRange]) -> Vec<CandidateOffset> {
    differences
        .iter()
        .filter(|range| (1..=8).contains(&(range.end_offset - range.start_offset)))
        .map(|range| {
            let width = range.end_offset - range.start_offset;
            let aligned = range.start_offset % width.max(1) == 0;
            CandidateOffset {
                object: range.object.clone(),
                offset: range.start_offset,
                width,
                score: if aligned { 0.45 } else { 0.3 },
                rationale: "Changed in a controlled two-snapshot diff; candidate only until repeated cross-player and cross-save validation.".to_string(),
            }
        })
        .collect()
}

#[tauri::command]
pub fn mapping_lab_compare(
    app: tauri::AppHandle,
    first_snapshot_id: String,
    second_snapshot_id: String,
) -> Result<ComparisonResult, String> {
    let directory = require_mode(&app)?;
    let first = load_snapshot(&directory, &first_snapshot_id)?;
    let second = load_snapshot(&directory, &second_snapshot_id)?;
    let (changed_bytes, unchanged_bytes, differences) = difference_ranges(&first, &second)?;
    let candidate_offsets = score_candidates(&differences);
    let comparison_id = format!(
        "comparison-{}-{}-{}",
        timestamp(),
        safe_label(&first_snapshot_id),
        safe_label(&second_snapshot_id)
    );
    let path = snapshot_path(&directory, &comparison_id)?;
    let evidence = serde_json::json!({
        "schemaVersion": 1,
        "generatedAt": timestamp(),
        "firstSnapshotId": first_snapshot_id,
        "secondSnapshotId": second_snapshot_id,
        "changedBytes": changed_bytes,
        "unchangedBytes": unchanged_bytes,
        "differences": &differences,
        "candidateOffsets": &candidate_offsets
    });
    fs::write(
        &path,
        serde_json::to_vec_pretty(&evidence)
            .map_err(|error| format!("Comparison evidence could not be encoded: {error}"))?,
    )
    .map_err(|error| format!("Comparison evidence could not be stored locally: {error}"))?;
    Ok(ComparisonResult {
        success: true,
        first_snapshot_id,
        second_snapshot_id,
        changed_bytes,
        unchanged_bytes,
        differences,
        candidate_offsets,
        evidence_file: path.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(bytes: &[u8]) -> EvidenceSnapshot {
        EvidenceSnapshot {
            schema_version: 1,
            captured_at: "1".to_string(),
            label: "test".to_string(),
            player_id: "1".to_string(),
            player_name: "Player".to_string(),
            process_id: 1,
            save_pointer: "0x1".to_string(),
            entity_map_profile_id: "profile".to_string(),
            executable_sha256: "hash".to_string(),
            read_only: true,
            windows: vec![EvidenceWindow {
                object: "player".to_string(),
                base_address: "0x10".to_string(),
                byte_length: bytes.len(),
                bytes_base64: STANDARD.encode(bytes),
            }],
        }
    }

    #[test]
    fn diff_groups_adjacent_changed_offsets() {
        let (changed, unchanged, ranges) =
            difference_ranges(&snapshot(&[1, 2, 3, 4]), &snapshot(&[1, 8, 9, 4])).unwrap();
        assert_eq!(changed, 2);
        assert_eq!(unchanged, 2);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_offset, 1);
        assert_eq!(ranges[0].end_offset, 3);
        let candidates = score_candidates(&ranges);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].score < 0.5);
    }

    #[test]
    fn labels_cannot_escape_the_evidence_directory() {
        assert_eq!(safe_label("../../outside"), "outside");
        assert!(snapshot_path(Path::new("lab"), "../../outside").is_err());
    }
}
