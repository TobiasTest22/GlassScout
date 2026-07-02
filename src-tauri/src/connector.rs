use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
    sync::{OnceLock, RwLock},
};
use tauri::Emitter;

use crate::{
    data::players::{IndexedPlayerRecord, PlayerDatabaseIndex},
    fm26::{
        memory::{ModuleInfo, ProcessReader},
        offsets::{find_entity_map, mapping_coverage, EntityMapProfile, MappingCoverage},
        parser::{default_role_for_position, preferred_foot_label, visible_attribute_map},
        permissions::{can_write_memory, READ_ONLY_PROCESS_ACCESS_LABEL},
        process::find_fm26_process,
        scanner::{parse_pattern, scan_module, scan_private_memory_for_pointers},
        structs::{FmDate, PLAYER_ATTRIBUTE_NAMES, POSITION_NAMES},
        validator,
    },
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorStatus {
    process_detected: bool,
    process_id: Option<u32>,
    process_path: Option<String>,
    save_detected: Option<bool>,
    memory_access: &'static str,
    parser_status: &'static str,
    state: &'static str,
    players_loaded: u32,
    managed_squad_players: u32,
    database_players_indexed: u32,
    background_players_indexed: u32,
    visible_players_loaded: u32,
    fully_scouted_players: u32,
    partial_scout_reports: u32,
    database_index_status: &'static str,
    database_scope: &'static str,
    clubs_loaded: u32,
    last_sync: Option<String>,
    bytes_read: usize,
    executable_header_valid: bool,
    can_write_memory: bool,
    game_build: Option<String>,
    product_version: Option<String>,
    executable_sha256: Option<String>,
    architecture: Option<String>,
    module_base: Option<String>,
    entity_map_status: &'static str,
    entity_map_profile_id: Option<String>,
    mapping_schema_version: u32,
    mapping_coverage: Vec<MappingCoverage>,
    pointer_validation: &'static str,
    handle_access_flags: &'static str,
    entity_root: Option<String>,
    save_pointer: Option<String>,
    managed_club_pointer: Option<String>,
    player_collection_pointer: Option<String>,
    live_memory_tactic_read: &'static str,
    tactic_manager_pointer: Option<String>,
    failure_stage: Option<String>,
    last_successful_read: Option<String>,
    windows_error_code: Option<u32>,
    message: String,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorSnapshot {
    status: ConnectorStatus,
    managed_club_id: Option<String>,
    manager_name: Option<String>,
    season: Option<String>,
    clubs: Vec<Value>,
    players: Vec<Value>,
    tactic: Option<Value>,
    tactic_source: &'static str,
    data_error: Option<String>,
    data_source: &'static str,
    data_warnings: Vec<String>,
}

#[derive(Default)]
struct ExecutableIdentity {
    file_version: Option<String>,
    product_version: Option<String>,
    sha256: Option<String>,
    architecture: Option<String>,
}

#[derive(Default)]
struct ExtractionDiagnostics {
    entity_root: Option<u64>,
    save_pointer: Option<u64>,
    managed_club_pointer: Option<u64>,
    player_collection_pointer: Option<u64>,
    last_successful_read: Option<String>,
}

struct LiveData {
    managed_club_id: String,
    manager_name: String,
    season: Option<String>,
    clubs: Vec<Value>,
    players: Vec<Value>,
    database_players_indexed: u32,
    background_players_indexed: u32,
    database_index_status: &'static str,
    database_scope: &'static str,
    warnings: Vec<String>,
    tactic_manager_pointer: Option<u64>,
}

static PLAYER_DATABASE_INDEX: OnceLock<RwLock<PlayerDatabaseIndex>> = OnceLock::new();

struct ExtractionFailure {
    stage: &'static str,
    message: String,
    windows_error_code: Option<u32>,
}

impl ExtractionFailure {
    fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
            windows_error_code: None,
        }
    }
}

#[tauri::command]
pub fn connector_status() -> ConnectorStatus {
    collect_snapshot(false, None).status
}

#[tauri::command]
pub fn connector_snapshot() -> ConnectorSnapshot {
    collect_snapshot(false, None)
}

#[tauri::command]
pub fn load_active_save(app: tauri::AppHandle) -> ConnectorSnapshot {
    let progress = |stage: &'static str| {
        let _ = app.emit("glassscout-load-progress", stage);
    };
    collect_snapshot(true, Some(&progress))
}

#[tauri::command]
pub fn search_indexed_players(query: String) -> Vec<Value> {
    let normalized = query.trim().to_lowercase();
    let Some(index) = PLAYER_DATABASE_INDEX.get() else {
        return Vec::new();
    };
    let Ok(index) = index.read() else {
        return Vec::new();
    };
    let _identity = (index.process_id, index.save_pointer);
    let mut results: Vec<Value> = index
        .records
        .values()
        .filter(|record| {
            normalized.is_empty()
                || record.name.to_lowercase().contains(&normalized)
                || record
                    .positions
                    .iter()
                    .any(|position| position.to_lowercase().contains(&normalized))
        })
        .map(indexed_player_json)
        .collect();
    results.sort_by(|left, right| {
        left["name"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["name"].as_str().unwrap_or_default())
    });
    results.truncate(500);
    results
}

#[tauri::command]
pub fn indexed_players_by_ids(player_ids: Vec<String>) -> Vec<Value> {
    let Some(index) = PLAYER_DATABASE_INDEX.get() else {
        return Vec::new();
    };
    let Ok(index) = index.read() else {
        return Vec::new();
    };
    player_ids
        .iter()
        .filter_map(|id| index.records.get(id))
        .map(indexed_player_json)
        .collect()
}

fn indexed_player_json(record: &IndexedPlayerRecord) -> Value {
    json!({
        "id": record.id,
        "name": record.name,
        "positions": record.positions,
        "managedSquad": record.managed_squad,
        "visibility": if record.visibility_safe { "known" } else { "unknown" },
        "scoutKnowledge": if record.visibility_safe { "fully_known" } else { "unknown" },
        "scoutConfidence": if record.visibility_safe { 100 } else { 0 }
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MappingLabWindow {
    pub(crate) object: &'static str,
    pub(crate) base_address: String,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MappingLabCaptureData {
    pub(crate) player_id: String,
    pub(crate) player_name: String,
    pub(crate) process_id: u32,
    pub(crate) save_pointer: String,
    pub(crate) entity_map_profile_id: String,
    pub(crate) executable_sha256: String,
    pub(crate) windows: Vec<MappingLabWindow>,
}

#[cfg(target_os = "windows")]
pub(crate) fn capture_mapping_lab_player(
    player_id: &str,
    window_size: usize,
) -> Result<MappingLabCaptureData, String> {
    let window_size = window_size.clamp(64, 4096);
    let index = PLAYER_DATABASE_INDEX
        .get()
        .ok_or_else(|| "Load the active save before capturing mapping evidence.".to_string())?
        .read()
        .map_err(|_| "The live player index is unavailable.".to_string())?;
    let record = index
        .records
        .get(player_id)
        .cloned()
        .or_else(|| {
            let mut matches = index
                .records
                .values()
                .filter(|record| record.name.eq_ignore_ascii_case(player_id));
            let first = matches.next()?.clone();
            matches.next().is_none().then_some(first)
        })
        .ok_or_else(|| {
            "No unique indexed player matched that FM ID or exact player name.".to_string()
        })?;
    let process_id = index.process_id;
    let save_pointer = index.save_pointer;
    drop(index);

    let mut reader = ProcessReader::open(process_id)
        .map_err(|code| format!("Read-only FM26 access failed with Windows error {code}."))?;
    let process_path = reader
        .process_path()
        .ok_or_else(|| "The FM26 executable path could not be read.".to_string())?;
    let identity = read_executable_identity(&process_path);
    let profile = find_entity_map(
        identity.file_version.as_deref(),
        identity.product_version.as_deref(),
        identity.sha256.as_deref(),
        identity.architecture.as_deref(),
    )
    .ok_or_else(|| "The running FM26 build does not match an exact entity map.".to_string())?;
    let mut targets = vec![
        ("player", record.raw_player_address),
        ("person", record.person_address),
    ];
    if let Some(contract) = record.contract_address {
        targets.push(("contract", contract));
    }
    let mut windows = Vec::with_capacity(targets.len());
    for (object, address) in targets {
        let bytes = reader
            .read_bytes(address, window_size)
            .ok_or_else(|| format!("The bounded {object} window was not readable."))?;
        windows.push(MappingLabWindow {
            object,
            base_address: hex_address(address),
            bytes,
        });
    }
    Ok(MappingLabCaptureData {
        player_id: record.id,
        player_name: record.name,
        process_id,
        save_pointer: hex_address(save_pointer),
        entity_map_profile_id: profile.id.clone(),
        executable_sha256: identity.sha256.unwrap_or_default(),
        windows,
    })
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn capture_mapping_lab_player(
    _player_id: &str,
    _window_size: usize,
) -> Result<MappingLabCaptureData, String> {
    Err("The FM26 mapping lab requires Windows.".to_string())
}

fn empty_status() -> ConnectorStatus {
    ConnectorStatus {
        process_detected: false,
        process_id: None,
        process_path: None,
        save_detected: None,
        memory_access: "not_checked",
        parser_status: "unverified",
        state: "process_not_found",
        players_loaded: 0,
        managed_squad_players: 0,
        database_players_indexed: 0,
        background_players_indexed: 0,
        visible_players_loaded: 0,
        fully_scouted_players: 0,
        partial_scout_reports: 0,
        database_index_status: "not_run",
        database_scope: "none",
        clubs_loaded: 0,
        last_sync: None,
        bytes_read: 0,
        executable_header_valid: false,
        can_write_memory: can_write_memory(),
        game_build: None,
        product_version: None,
        executable_sha256: None,
        architecture: None,
        module_base: None,
        entity_map_status: "not_checked",
        entity_map_profile_id: None,
        mapping_schema_version: 2,
        mapping_coverage: Vec::new(),
        pointer_validation: "not_run",
        handle_access_flags: READ_ONLY_PROCESS_ACCESS_LABEL,
        entity_root: None,
        save_pointer: None,
        managed_club_pointer: None,
        player_collection_pointer: None,
        live_memory_tactic_read: "not_run",
        tactic_manager_pointer: None,
        failure_stage: None,
        last_successful_read: None,
        windows_error_code: None,
        message: "FM26 is not running. Open FM26 and load your save to begin.".to_string(),
        warnings: Vec::new(),
    }
}

fn empty_snapshot(status: ConnectorStatus, error: String) -> ConnectorSnapshot {
    ConnectorSnapshot {
        status,
        managed_club_id: None,
        manager_name: None,
        season: None,
        clubs: Vec::new(),
        players: Vec::new(),
        tactic: None,
        tactic_source: "none",
        data_error: Some(error),
        data_source: "none",
        data_warnings: Vec::new(),
    }
}

#[cfg(target_os = "windows")]
fn collect_snapshot(
    include_database_index: bool,
    progress: Option<&dyn Fn(&'static str)>,
) -> ConnectorSnapshot {
    if let Some(progress) = progress {
        progress("detecting_fm26");
    }
    let Some((process_id, _)) = find_fm26_process() else {
        let status = empty_status();
        return empty_snapshot(status.clone(), status.message);
    };

    let mut status = empty_status();
    status.process_detected = true;
    status.process_id = Some(process_id);
    if let Some(progress) = progress {
        progress("validating_active_save");
    }

    let mut reader = match ProcessReader::open(process_id) {
        Ok(reader) => reader,
        Err(code) => {
            status.state = "access_denied";
            status.memory_access = "denied";
            status.windows_error_code = Some(code);
            status.failure_stage = Some("open_read_only_process".to_string());
            status.message =
                "FM26 is running, but GlassScout could not open its read-only connection."
                    .to_string();
            return empty_snapshot(status.clone(), status.message);
        }
    };
    status.memory_access = "read_only_handle_open";
    status.process_path = reader.process_path();

    let identity = status
        .process_path
        .as_deref()
        .map(read_executable_identity)
        .unwrap_or_default();
    status.game_build = identity.file_version.clone();
    status.product_version = identity.product_version.clone();
    status.executable_sha256 = identity.sha256.clone();
    status.architecture = identity.architecture.clone();

    let Some(profile) = find_entity_map(
        identity.file_version.as_deref(),
        identity.product_version.as_deref(),
        identity.sha256.as_deref(),
        identity.architecture.as_deref(),
    ) else {
        status.state = "parser_unverified";
        status.entity_map_status = "missing";
        status.failure_stage = Some("exact_build_match".to_string());
        status.last_successful_read = Some("read_executable_identity".to_string());
        status.bytes_read = reader.bytes_read;
        status.message = format!(
            "FM26 build {} is not supported safely yet. No game data was shown.",
            identity.file_version.as_deref().unwrap_or("unknown")
        );
        return empty_snapshot(status.clone(), status.message);
    };
    status.entity_map_status = "matched";
    status.entity_map_profile_id = Some(profile.id.clone());
    status.mapping_coverage = mapping_coverage(profile);

    let Some(module) = reader.module(&profile.module) else {
        status.state = "parser_unverified";
        status.pointer_validation = "failed";
        status.failure_stage = Some("locate_game_module".to_string());
        status.last_successful_read = Some("match_exact_build".to_string());
        status.windows_error_code = reader.last_error;
        status.bytes_read = reader.bytes_read;
        status.message =
            "The FM26 game module was not available. No game data was shown.".to_string();
        return empty_snapshot(status.clone(), status.message);
    };
    status.module_base = Some(hex_address(module.base));
    let header = reader.read_bytes(module.base, 2);
    status.executable_header_valid = header.as_deref() == Some(b"MZ");
    if !status.executable_header_valid {
        status.state = "parser_unverified";
        status.pointer_validation = "failed";
        status.failure_stage = Some("validate_game_module".to_string());
        status.last_successful_read = Some("locate_game_module".to_string());
        status.bytes_read = reader.bytes_read;
        status.message =
            "The FM26 game module could not be validated. No game data was shown.".to_string();
        return empty_snapshot(status.clone(), status.message);
    }

    if let Some(progress) = progress {
        progress("reading_managed_club");
    }
    let mut diagnostics = ExtractionDiagnostics::default();
    let extracted = extract_live_data(
        &mut reader,
        module,
        profile,
        &mut diagnostics,
        process_id,
        include_database_index,
        progress,
    );
    status.bytes_read = reader.bytes_read;
    status.entity_root = diagnostics.entity_root.map(hex_address);
    status.save_pointer = diagnostics.save_pointer.map(hex_address);
    status.managed_club_pointer = diagnostics.managed_club_pointer.map(hex_address);
    status.player_collection_pointer = diagnostics.player_collection_pointer.map(hex_address);
    status.last_successful_read = diagnostics.last_successful_read;

    match extracted {
        Ok(data) => {
            if let Some(progress) = progress {
                progress("ready");
            }
            status.save_detected = Some(true);
            status.parser_status = "ready";
            status.state = "connected";
            status.players_loaded = data.players.len() as u32;
            status.managed_squad_players = data.players.len() as u32;
            status.database_players_indexed = data.database_players_indexed;
            status.background_players_indexed = data.background_players_indexed;
            status.visible_players_loaded = data.players.len() as u32;
            status.fully_scouted_players = data.players.len() as u32;
            status.partial_scout_reports = 0;
            status.database_index_status = data.database_index_status;
            status.database_scope = data.database_scope;
            status.clubs_loaded = data.clubs.len() as u32;
            status.pointer_validation = "passed";
            status.live_memory_tactic_read = if data.tactic_manager_pointer.is_some() {
                "object_detected_unmapped"
            } else {
                "object_not_found"
            };
            status.tactic_manager_pointer = data.tactic_manager_pointer.map(hex_address);
            status.last_sync = Some(unix_milliseconds());
            status.message = format!(
                "Connected to {} with {} live squad players and {} indexed player records.",
                data.clubs
                    .first()
                    .and_then(|club| club.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or("the managed club"),
                data.players.len(),
                data.database_players_indexed
            );
            status.warnings = data.warnings.clone();
            ConnectorSnapshot {
                status,
                managed_club_id: Some(data.managed_club_id),
                manager_name: Some(data.manager_name),
                season: data.season,
                clubs: data.clubs,
                players: data.players,
                tactic: None,
                tactic_source: "none",
                data_error: None,
                data_source: "live-memory",
                data_warnings: data.warnings,
            }
        }
        Err(failure) => {
            status.save_detected = Some(false);
            status.parser_status = "error";
            status.state = "parser_unverified";
            status.pointer_validation = "failed";
            status.failure_stage = Some(failure.stage.to_string());
            status.windows_error_code = failure.windows_error_code.or(reader.last_error);
            status.message = failure.message;
            empty_snapshot(status.clone(), status.message)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn collect_snapshot(
    _include_database_index: bool,
    _progress: Option<&dyn Fn(&'static str)>,
) -> ConnectorSnapshot {
    let mut status = empty_status();
    status.message = "The live FM26 connector requires the installed Windows app.".to_string();
    empty_snapshot(status.clone(), status.message)
}

#[cfg(target_os = "windows")]
fn extract_live_data(
    reader: &mut ProcessReader,
    module: ModuleInfo,
    profile: &EntityMapProfile,
    diagnostics: &mut ExtractionDiagnostics,
    process_id: u32,
    include_database_index: bool,
    progress: Option<&dyn Fn(&'static str)>,
) -> Result<LiveData, ExtractionFailure> {
    let signature = profile
        .signatures
        .iter()
        .find(|item| item.name == "human_manager_registry")
        .ok_or_else(|| {
            ExtractionFailure::new("manager_signature", "The exact build map is incomplete.")
        })?;
    let pattern = parse_pattern(&signature.pattern).map_err(|_| {
        ExtractionFailure::new("entity_map", "The embedded signature pattern is invalid.")
    })?;
    let hits = scan_module(reader, module, &pattern)
        .map_err(|error| ExtractionFailure::new("manager_signature", error.to_string()))?;
    if hits.len() != 1 {
        return Err(ExtractionFailure::new(
            "manager_signature",
            format!(
                "The FM26 manager root did not validate uniquely ({} matches). No game data was shown.",
                hits.len()
            ),
        ));
    }
    let signature_address = hits[0];
    let displacement = reader.read_i32(signature_address + 3).ok_or_else(|| {
        ExtractionFailure::new(
            "manager_signature",
            "The FM26 manager signature could not be read.",
        )
    })?;
    let registry_slot = (signature_address + 7).wrapping_add_signed(displacement as i64);
    let registry = reader
        .read_pointer(registry_slot)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "manager_registry",
                "No active FM26 manager registry was available.",
            )
        })?;
    diagnostics.entity_root = Some(registry);
    diagnostics.last_successful_read = Some("resolve_manager_registry".to_string());

    let vector_start = reader
        .read_pointer(registry + profile.constants.manager_registry_vector_offset)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "manager_registry",
                "The active manager collection could not be read.",
            )
        })?;
    let vector_end = reader
        .read_pointer(registry + profile.constants.manager_registry_vector_offset + 8)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "manager_registry",
                "The active manager collection end could not be read.",
            )
        })?;
    if vector_end <= vector_start || vector_end - vector_start != 8 {
        return Err(ExtractionFailure::new(
            "manager_registry",
            "GlassScout could not identify exactly one active human manager.",
        ));
    }
    let human = reader
        .read_pointer(vector_start)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "human_manager",
                "The active human manager pointer was empty.",
            )
        })?;
    diagnostics.save_pointer = Some(human);
    diagnostics.last_successful_read = Some("resolve_human_manager".to_string());

    let person = human + profile.constants.human_person_offset;
    let first_name = read_name_field(reader, person + profile.constants.person_first_name_offset);
    let second_name = read_name_field(reader, person + profile.constants.person_second_name_offset);
    let common_name = read_name_field(reader, person + profile.constants.person_common_name_offset);
    let manager_name = display_name(first_name, second_name, common_name).ok_or_else(|| {
        ExtractionFailure::new(
            "manager_identity",
            "The active manager identity could not be validated.",
        )
    })?;

    let contract = reader
        .read_pointer(person + profile.constants.person_contract_offset)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "active_contract",
                "The manager's active FM26 contract was not available.",
            )
        })?;
    let team = reader
        .read_pointer(contract + profile.constants.contract_team_offset)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "managed_team",
                "The managed FM26 team could not be resolved.",
            )
        })?;
    validator::validate_vtable(
        reader,
        team,
        module.base + profile.constants.team_vtable_rva,
    )
    .map_err(|message| ExtractionFailure::new("managed_team", message))?;
    let club = reader
        .read_pointer(team + profile.constants.team_club_offset)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "managed_club",
                "The managed FM26 club could not be resolved.",
            )
        })?;
    validator::validate_vtable(
        reader,
        club,
        module.base + profile.constants.club_vtable_rva,
    )
    .map_err(|message| ExtractionFailure::new("managed_club", message))?;
    diagnostics.managed_club_pointer = Some(club);
    diagnostics.last_successful_read = Some("validate_managed_club".to_string());
    if let Some(progress) = progress {
        progress("loading_managed_squad");
    }

    let team_uid = reader
        .read_u32(team + profile.constants.entity_uid_offset)
        .filter(|uid| *uid > 0)
        .ok_or_else(|| {
            ExtractionFailure::new("managed_club", "The managed team identifier was invalid.")
        })?;
    let club_uid = reader
        .read_u32(club + profile.constants.entity_uid_offset)
        .filter(|uid| *uid == team_uid)
        .ok_or_else(|| {
            ExtractionFailure::new("managed_club", "The team-to-club identifier check failed.")
        })?;
    let club_name_pointer = reader
        .read_pointer(club + profile.constants.club_name_offset)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            ExtractionFailure::new("managed_club", "The managed club name pointer was empty.")
        })?;
    let club_name = reader
        .read_length_prefixed_string(club_name_pointer)
        .ok_or_else(|| {
            ExtractionFailure::new("managed_club", "The managed club name was not readable.")
        })?;
    let club_id = club_uid.to_string();

    let players_start = reader
        .read_pointer(team + profile.constants.team_players_start_offset)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "player_collection",
                "The managed squad collection was not readable.",
            )
        })?;
    let players_end = reader
        .read_pointer(team + profile.constants.team_players_end_offset)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "player_collection",
                "The managed squad collection end was not readable.",
            )
        })?;
    if players_end <= players_start
        || (players_end - players_start) % 8 != 0
        || !(1..=200).contains(&((players_end - players_start) / 8))
    {
        return Err(ExtractionFailure::new(
            "player_collection",
            "The managed squad collection failed its size and alignment checks.",
        ));
    }
    diagnostics.player_collection_pointer = Some(players_start);
    let player_count = ((players_end - players_start) / 8) as usize;
    let mut players = Vec::with_capacity(player_count);
    let mut managed_player_ids = HashSet::with_capacity(player_count);
    let mut managed_index_records = Vec::with_capacity(player_count);
    let mut player_vtable = None;
    for index in 0..player_count {
        let raw_player = reader
            .read_pointer(players_start + (index as u64 * 8))
            .filter(|value| *value != 0)
            .ok_or_else(|| {
                ExtractionFailure::new(
                    "player_collection",
                    "A managed squad player pointer was empty.",
                )
            })?;
        let vtable = reader
            .read_pointer(raw_player)
            .filter(|value| *value != 0)
            .ok_or_else(|| {
                ExtractionFailure::new(
                    "player_identity",
                    "A managed squad player type record was invalid.",
                )
            })?;
        player_vtable.get_or_insert(vtable);
        let person = raw_player + profile.constants.player_person_offset;
        let uid = reader
            .read_u32(person + profile.constants.entity_uid_offset)
            .filter(|uid| *uid > 0)
            .ok_or_else(|| {
                ExtractionFailure::new(
                    "player_identity",
                    "A managed squad player identifier was invalid.",
                )
            })?;
        let name = display_name(
            read_name_field(reader, person + profile.constants.person_first_name_offset),
            read_name_field(reader, person + profile.constants.person_second_name_offset),
            read_name_field(reader, person + profile.constants.person_common_name_offset),
        )
        .ok_or_else(|| {
            ExtractionFailure::new(
                "player_identity",
                "A managed squad player name was not readable.",
            )
        })?;
        let position_bytes = reader
            .read_bytes(
                raw_player + profile.constants.player_positions_offset,
                POSITION_NAMES.len(),
            )
            .ok_or_else(|| {
                ExtractionFailure::new(
                    "player_positions",
                    "A managed squad position record was not readable.",
                )
            })?;
        if position_bytes.iter().any(|rating| *rating > 20) {
            return Err(ExtractionFailure::new(
                "player_positions",
                "A managed squad position record failed its rating bounds check.",
            ));
        }
        let mut positions: Vec<String> = position_bytes
            .iter()
            .enumerate()
            .filter(|(_, rating)| **rating >= 15)
            .map(|(position, _)| POSITION_NAMES[position].to_string())
            .collect();
        if positions.is_empty() {
            if let Some((position, _)) = position_bytes
                .iter()
                .enumerate()
                .max_by_key(|(_, rating)| **rating)
            {
                positions.push(POSITION_NAMES[position].to_string());
            }
        }
        let calculated_position = position_bytes
            .iter()
            .enumerate()
            .max_by_key(|(_, rating)| **rating)
            .map(|(position, _)| POSITION_NAMES[position].to_string());
        let birth_date = read_fm_date(reader, person + profile.constants.person_birth_date_offset);
        let current_date = read_fm_date(
            reader,
            raw_player + profile.constants.player_current_date_offset,
        );
        let age = birth_date
            .zip(current_date)
            .and_then(|(birth, current)| calculate_age(birth, current));
        let date_of_birth = birth_date.and_then(format_fm_date);
        let season = current_date.map(|date| {
            let next_year = date.year.saturating_add(1);
            format!("{}/{}", date.year, next_year % 100)
        });
        let nationality = read_nationality(reader, person, profile);
        let attribute_bytes = reader
            .read_bytes(
                raw_player + profile.constants.player_attributes_offset,
                PLAYER_ATTRIBUTE_NAMES.len(),
            )
            .ok_or_else(|| {
                ExtractionFailure::new(
                    "player_attributes",
                    "A managed squad attribute record was not readable.",
                )
            })?;
        if attribute_bytes.iter().any(|value| *value > 100) {
            return Err(ExtractionFailure::new(
                "player_attributes",
                "A managed squad attribute record failed its 1–100 storage bounds check.",
            ));
        }
        let visible_attributes = visible_attribute_map(&attribute_bytes);
        let preferred_foot = preferred_foot_label(attribute_bytes[24], attribute_bytes[25]);
        let best_role = calculated_position
            .as_deref()
            .map(default_role_for_position);
        let ability_score = calculated_position
            .as_deref()
            .and_then(|position| visible_ability_score(position, &visible_attributes));
        let (strengths, weaknesses) = attribute_evidence(&visible_attributes);
        let validated_at = unix_milliseconds();
        let attribute_knowledge: serde_json::Map<String, Value> = visible_attributes
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    json!({
                        "value": value,
                        "visibility": "known",
                        "source": "own-squad",
                        "confidence": 100,
                        "lastValidated": validated_at
                    }),
                )
            })
            .collect();
        let player_id = uid.to_string();
        let contract_address = reader
            .read_pointer(person + profile.constants.person_contract_offset)
            .filter(|value| *value != 0);
        managed_player_ids.insert(player_id.clone());
        managed_index_records.push(IndexedPlayerRecord {
            id: player_id.clone(),
            name: name.clone(),
            positions: positions.clone(),
            managed_squad: true,
            visibility_safe: true,
            raw_player_address: raw_player,
            person_address: person,
            contract_address,
        });
        players.push(json!({
            "id": player_id,
            "name": name.clone(),
            "_season": season,
            "age": age,
            "dateOfBirth": date_of_birth,
            "nationality": nationality.clone(),
            "secondNationality": null,
            "positions": positions.clone(),
            "bestRole": best_role,
            "currentAbility": null,
            "potentialAbility": null,
            "abilityScore": ability_score,
            "form": null,
            "averageRating": null,
            "minutesPlayed": null,
            "goals": null,
            "assists": null,
            "contractStatus": null,
            "value": null,
            "wage": null,
            "squadImportance": null,
            "developmentTrend": null,
            "tacticalFit": null,
            "roleFit": ability_score,
            "preferredFoot": preferred_foot,
            "strengths": strengths,
            "weaknesses": weaknesses,
            "clubId": club_id,
            "transferInterest": null,
            "loanInterest": null,
            "transferAvailable": null,
            "loanAvailable": null,
            "attributes": visible_attributes,
            "per90": {},
            "scoutKnowledge": "fully_known",
            "scoutConfidence": 100,
            "lastScoutedDate": null,
            "reportReliability": null,
            "bestCalculatedPosition": calculated_position,
            "truePrice": null,
            "fairPriceRange": null,
            "valuationLabel": "unavailable",
            "valuationReasoning": ["FM26 valuation fields are not mapped for this exact build."],
            "retrainingSuggestion": null,
            "roleReasoning": ["Role evidence uses only the managed player's visible FM26 attributes and positional familiarity."],
            "riskLevel": "unknown",
            "marketValueAmount": null
            ,"personality": null
            ,"condition": null
            ,"recommendation": {
                "minimum": ability_score,
                "maximum": ability_score,
                "completeness": 100,
                "label": if ability_score.is_some() { "full visible-attribute evidence" } else { "not enough evidence" }
            }
            ,"knowledge": {
                "name": { "value": name, "visibility": "known", "source": "own-squad", "confidence": 100, "lastValidated": validated_at },
                "age": { "value": age, "visibility": "known", "source": "own-squad", "confidence": 100, "lastValidated": validated_at },
                "nationality": { "value": nationality, "visibility": "known", "source": "own-squad", "confidence": 100, "lastValidated": validated_at },
                "positions": { "value": positions, "visibility": "known", "source": "own-squad", "confidence": 100, "lastValidated": validated_at },
                "attributes": attribute_knowledge,
                "form": { "value": null, "visibility": "unknown", "source": "memory-raw", "confidence": 0, "lastValidated": null },
                "contract": { "value": null, "visibility": "unknown", "source": "memory-raw", "confidence": 0, "lastValidated": null },
                "wage": { "value": null, "visibility": "unknown", "source": "memory-raw", "confidence": 0, "lastValidated": null },
                "value": { "value": null, "visibility": "unknown", "source": "memory-raw", "confidence": 0, "lastValidated": null },
                "interest": { "value": null, "visibility": "unknown", "source": "memory-raw", "confidence": 0, "lastValidated": null }
            }
        }));
    }
    diagnostics.last_successful_read = Some("extract_managed_squad".to_string());

    let (
        database_players_indexed,
        background_players_indexed,
        database_index_status,
        database_scope,
        tactic_manager_pointer,
    ) = if include_database_index {
        if let Some(progress) = progress {
            progress("indexing_player_database");
        }
        let seed_vtable = player_vtable.ok_or_else(|| {
            ExtractionFailure::new(
                "player_database",
                "The live squad did not provide a player type signature.",
            )
        })?;
        match index_full_player_database(
            reader,
            profile,
            process_id,
            diagnostics.save_pointer.unwrap_or_default(),
            seed_vtable,
            module,
            &managed_player_ids,
            managed_index_records,
        ) {
            Ok(indexed) => {
                if let Some(progress) = progress {
                    progress("building_visibility_index");
                }
                diagnostics.last_successful_read = Some("index_full_player_database".to_string());
                (
                    indexed.total,
                    indexed.background,
                    "ready",
                    "full-save-index",
                    indexed.tactic_manager_pointer,
                )
            }
            Err(_) => (player_count as u32, 0, "partial", "managed-squad", None),
        }
    } else {
        store_player_index(
            process_id,
            diagnostics.save_pointer.unwrap_or_default(),
            managed_index_records,
        );
        (player_count as u32, 0, "not_run", "managed-squad", None)
    };

    let season = players
        .first()
        .and_then(|player| player.get("_season"))
        .and_then(Value::as_str)
        .map(str::to_string);
    for player in &mut players {
        if let Some(object) = player.as_object_mut() {
            object.remove("_season");
        }
    }
    let mut warnings = vec![
        "Managed-squad IDs, names, dates of birth, ages, nationality, positions, preferred foot and visible attributes are validated for this FM26 build. Hidden CA/PA is never read or scored.".to_string(),
        "Form, match ratings, contract, wage, valuation, fitness and squad-status relationships are not yet validated for this build and remain Unknown.".to_string(),
        "Own-squad players are fully known. Wider-save records remain hidden until FM26 scout-report visibility, ranges and confidence are validated.".to_string(),
        "The live FM26 tactic manager is detected with read-only access. Packed formation, phase-role and instruction layouts are not yet validated, so no tactic is guessed.".to_string(),
        "The FM26 shortlist collection is not mapped safely. GlassScout Favorites remains a local list resolved against live players.".to_string(),
    ];
    if include_database_index && database_index_status == "ready" {
        warnings.push(format!(
            "{background_players_indexed} wider-save player records were indexed in memory. They remain hidden from the UI until FM scout-knowledge visibility can be validated."
        ));
    } else if include_database_index {
        warnings.push(
            "The wider player database could not be indexed safely; only the managed squad is available."
                .to_string(),
        );
    }
    let clubs = vec![json!({
        "id": club_id,
        "name": club_name,
        "nation": null,
        "league": null
    })];
    Ok(LiveData {
        managed_club_id: club_uid.to_string(),
        manager_name,
        season,
        clubs,
        players,
        database_players_indexed,
        background_players_indexed,
        database_index_status,
        database_scope,
        warnings,
        tactic_manager_pointer,
    })
}

#[cfg(target_os = "windows")]
struct IndexSummary {
    total: u32,
    background: u32,
    tactic_manager_pointer: Option<u64>,
}

#[cfg(target_os = "windows")]
fn store_player_index(process_id: u32, save_pointer: u64, records: Vec<IndexedPlayerRecord>) {
    let records = records
        .into_iter()
        .map(|record| (record.id.clone(), record))
        .collect();
    let index = PLAYER_DATABASE_INDEX.get_or_init(|| RwLock::new(PlayerDatabaseIndex::default()));
    if let Ok(mut index) = index.write() {
        *index = PlayerDatabaseIndex {
            process_id,
            save_pointer,
            records,
        };
    }
}

#[cfg(target_os = "windows")]
fn index_full_player_database(
    reader: &mut ProcessReader,
    profile: &EntityMapProfile,
    process_id: u32,
    save_pointer: u64,
    player_vtable: u64,
    module: ModuleInfo,
    managed_player_ids: &HashSet<String>,
    managed_records: Vec<IndexedPlayerRecord>,
) -> Result<IndexSummary, ExtractionFailure> {
    let mut records: HashMap<String, IndexedPlayerRecord> = managed_records
        .into_iter()
        .map(|record| (record.id.clone(), record))
        .collect();
    let tactics_manager_vtable = module.base + profile.constants.tactics_manager_vtable_rva;
    let mut object_hits =
        scan_private_memory_for_pointers(reader, &[player_vtable, tactics_manager_vtable])
            .map_err(|error| ExtractionFailure::new("player_database", error.to_string()))?;
    let candidate_addresses = object_hits.remove(&player_vtable).unwrap_or_default();
    let tactic_manager_hits = object_hits
        .remove(&tactics_manager_vtable)
        .unwrap_or_default();
    let tactic_manager_pointer = (tactic_manager_hits.len() == 1).then_some(tactic_manager_hits[0]);

    for address in candidate_addresses {
        if records.len() >= 250_000 {
            break;
        }
        let Some(mut record) = read_indexed_player_candidate(reader, address, profile) else {
            continue;
        };
        if managed_player_ids.contains(&record.id) {
            continue;
        }
        record.managed_squad = false;
        record.visibility_safe = false;
        records.entry(record.id.clone()).or_insert(record);
    }

    if records.len() < managed_player_ids.len() {
        return Err(ExtractionFailure::new(
            "player_database",
            "The wider player index failed its managed-squad integrity check.",
        ));
    }

    let total = records.len() as u32;
    let background = records
        .values()
        .filter(|record| !record.managed_squad)
        .count() as u32;
    let index = PLAYER_DATABASE_INDEX.get_or_init(|| RwLock::new(PlayerDatabaseIndex::default()));
    if let Ok(mut index) = index.write() {
        *index = PlayerDatabaseIndex {
            process_id,
            save_pointer,
            records,
        };
    }
    Ok(IndexSummary {
        total,
        background,
        tactic_manager_pointer,
    })
}

#[cfg(target_os = "windows")]
fn read_indexed_player_candidate(
    reader: &mut ProcessReader,
    raw_player: u64,
    profile: &EntityMapProfile,
) -> Option<IndexedPlayerRecord> {
    let person = raw_player.checked_add(profile.constants.player_person_offset)?;
    let uid = reader
        .read_u32(person.checked_add(profile.constants.entity_uid_offset)?)
        .filter(|uid| *uid > 0)?;
    let name = display_name(
        read_name_field(
            reader,
            person.checked_add(profile.constants.person_first_name_offset)?,
        ),
        read_name_field(
            reader,
            person.checked_add(profile.constants.person_second_name_offset)?,
        ),
        read_name_field(
            reader,
            person.checked_add(profile.constants.person_common_name_offset)?,
        ),
    )?;
    let position_bytes = reader.read_bytes(
        raw_player.checked_add(profile.constants.player_positions_offset)?,
        POSITION_NAMES.len(),
    )?;
    if position_bytes.iter().any(|rating| *rating > 20)
        || position_bytes.iter().all(|rating| *rating == 0)
    {
        return None;
    }
    let mut positions: Vec<String> = position_bytes
        .iter()
        .enumerate()
        .filter(|(_, rating)| **rating >= 15)
        .map(|(position, _)| POSITION_NAMES[position].to_string())
        .collect();
    if positions.is_empty() {
        let (position, _) = position_bytes
            .iter()
            .enumerate()
            .max_by_key(|(_, rating)| **rating)?;
        positions.push(POSITION_NAMES[position].to_string());
    }
    Some(IndexedPlayerRecord {
        id: uid.to_string(),
        name,
        positions,
        managed_squad: false,
        visibility_safe: false,
        raw_player_address: raw_player,
        person_address: person,
        contract_address: reader
            .read_pointer(person.checked_add(profile.constants.person_contract_offset)?)
            .filter(|value| *value != 0),
    })
}

fn visible_ability_score(position: &str, attributes: &HashMap<String, u8>) -> Option<u8> {
    let keys: &[&str] = match position {
        "GK" => &[
            "Aerial Reach",
            "Command of Area",
            "Communication",
            "Handling",
            "One on Ones",
            "Reflexes",
            "Positioning",
            "Decisions",
        ],
        "DC" | "SW" => &[
            "Marking",
            "Tackling",
            "Heading",
            "Positioning",
            "Anticipation",
            "Decisions",
            "Jumping Reach",
            "Strength",
        ],
        "DL" | "DR" | "WBL" | "WBR" => &[
            "Marking",
            "Tackling",
            "Positioning",
            "Crossing",
            "Pace",
            "Acceleration",
            "Stamina",
            "Work Rate",
        ],
        "DM" => &[
            "Tackling",
            "Positioning",
            "Anticipation",
            "Decisions",
            "Passing",
            "Teamwork",
            "Work Rate",
            "Stamina",
        ],
        "ML" | "MR" | "AML" | "AMR" => &[
            "Crossing",
            "Dribbling",
            "First Touch",
            "Off the Ball",
            "Pace",
            "Acceleration",
            "Agility",
            "Technique",
        ],
        "MC" | "AMC" => &[
            "Passing",
            "Vision",
            "First Touch",
            "Technique",
            "Decisions",
            "Anticipation",
            "Teamwork",
            "Work Rate",
        ],
        "ST" => &[
            "Finishing",
            "First Touch",
            "Off the Ball",
            "Anticipation",
            "Composure",
            "Acceleration",
            "Pace",
            "Technique",
        ],
        _ => &["Decisions", "Teamwork", "Work Rate", "Natural Fitness"],
    };
    let values = keys
        .iter()
        .filter_map(|key| attributes.get(*key).copied())
        .map(u32::from)
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| {
        let average = values.iter().sum::<u32>() as f32 / values.len() as f32;
        ((average / 20.0) * 100.0).round().clamp(0.0, 100.0) as u8
    })
}

fn attribute_evidence(attributes: &HashMap<String, u8>) -> (Vec<String>, Vec<String>) {
    let mut values = attributes
        .iter()
        .map(|(name, value)| (name.clone(), *value))
        .collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let strengths = values
        .iter()
        .take(3)
        .map(|(name, value)| format!("{name} {value}"))
        .collect();
    let weaknesses = values
        .iter()
        .rev()
        .take(3)
        .map(|(name, value)| format!("{name} {value}"))
        .collect();
    (strengths, weaknesses)
}

fn is_leap_year(year: u16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_day(date: FmDate) -> Option<(u8, u8)> {
    let mut remaining = u32::from(date.day_of_year);
    if remaining == 0 {
        return None;
    }
    let month_lengths = [
        31_u32,
        if is_leap_year(date.year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for (index, length) in month_lengths.iter().enumerate() {
        if remaining <= *length {
            return Some(((index + 1) as u8, remaining as u8));
        }
        remaining -= length;
    }
    None
}

fn format_fm_date(date: FmDate) -> Option<String> {
    let (month, day) = month_day(date)?;
    Some(format!("{:04}-{month:02}-{day:02}", date.year))
}

fn calculate_age(birth: FmDate, current: FmDate) -> Option<u8> {
    if current.year < birth.year {
        return None;
    }
    let birthday_passed = current.day_of_year >= birth.day_of_year;
    let years = current.year - birth.year - u16::from(!birthday_passed);
    u8::try_from(years).ok().filter(|age| *age <= 100)
}

#[cfg(target_os = "windows")]
fn read_fm_date(reader: &mut ProcessReader, address: u64) -> Option<FmDate> {
    let bytes = reader.read_bytes(address, 4)?;
    let day_of_year = u16::from_le_bytes([bytes[0], bytes[1]]);
    let year = u16::from_le_bytes([bytes[2], bytes[3]]);
    let max_day = if is_leap_year(year) { 366 } else { 365 };
    (year >= 1900 && day_of_year > 0 && day_of_year <= max_day)
        .then_some(FmDate { year, day_of_year })
}

#[cfg(target_os = "windows")]
fn read_nationality(
    reader: &mut ProcessReader,
    person: u64,
    profile: &EntityMapProfile,
) -> Option<String> {
    let nation = reader.read_pointer(person + profile.constants.person_nationality_offset)?;
    let name = (nation != 0)
        .then(|| reader.read_pointer(nation + profile.constants.nation_name_offset))
        .flatten()?;
    (name != 0)
        .then(|| reader.read_length_prefixed_string(name))
        .flatten()
}

fn display_name(
    first_name: Option<String>,
    second_name: Option<String>,
    common_name: Option<String>,
) -> Option<String> {
    if let Some(common) = common_name.filter(|value| !value.trim().is_empty()) {
        return Some(common);
    }
    let full = [first_name, second_name]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!full.is_empty()).then_some(full)
}

#[cfg(target_os = "windows")]
fn read_name_field(reader: &mut ProcessReader, field: u64) -> Option<String> {
    let entry = reader.read_pointer(field)?;
    if entry == 0 {
        return None;
    }
    let text = reader.read_pointer(entry)?;
    if text == 0 {
        return None;
    }
    reader.read_length_prefixed_string(text)
}

#[cfg(test)]
fn checked_currency_transform(raw: i64, scale: u64, maximum: u64) -> Option<u64> {
    let value = u64::try_from(raw).ok()?.checked_mul(scale)?;
    (value <= maximum).then_some(value)
}

#[cfg(test)]
fn validated_enum<'a>(raw: usize, values: &'a [&'a str]) -> Option<&'a str> {
    values.get(raw).copied()
}

fn read_executable_identity(path: &str) -> ExecutableIdentity {
    let mut identity = ExecutableIdentity {
        sha256: hash_file(path),
        architecture: read_pe_architecture(path),
        ..ExecutableIdentity::default()
    };

    #[cfg(target_os = "windows")]
    {
        let escaped_path = path.replace('\'', "''");
        let script = format!(
            "$v=(Get-Item -LiteralPath '{escaped_path}').VersionInfo; [Console]::Write($v.FileVersion+'|'+$v.ProductVersion)"
        );
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
        {
            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout);
                let mut parts = value.splitn(2, '|');
                identity.file_version = parts
                    .next()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
                identity.product_version = parts
                    .next()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
            }
        }
    }
    identity
}

fn hash_file(path: &str) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Some(format!("{:X}", hasher.finalize()))
}

fn read_pe_architecture(path: &str) -> Option<String> {
    let mut file = File::open(Path::new(path)).ok()?;
    let mut dos_header = [0_u8; 64];
    file.read_exact(&mut dos_header).ok()?;
    if &dos_header[..2] != b"MZ" {
        return None;
    }
    let pe_offset = u32::from_le_bytes(dos_header[0x3c..0x40].try_into().ok()?) as u64;
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Start(pe_offset + 4)).ok()?;
    let mut machine = [0_u8; 2];
    file.read_exact(&mut machine).ok()?;
    match u16::from_le_bytes(machine) {
        0x8664 => Some("x64".to_string()),
        0x014c => Some("x86".to_string()),
        _ => Some("unknown".to_string()),
    }
}

fn hex_address(value: u64) -> String {
    format!("0x{value:X}")
}

fn unix_milliseconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fm26::{
        offsets::{embedded_entity_map_index, field_is_publishable, FieldDefinition},
        permissions::READ_ONLY_PROCESS_ACCESS,
    };

    #[test]
    fn connector_contract_never_requests_or_advertises_write_access() {
        assert_eq!(READ_ONLY_PROCESS_ACCESS, 0x1410);
        assert!(!empty_status().can_write_memory);
    }

    #[test]
    fn exact_build_profile_is_embedded_and_read_only() {
        let index = embedded_entity_map_index();
        assert_eq!(index.schema_version, 2);
        assert_eq!(index.profiles.len(), 1);
        assert_eq!(index.profiles[0].module, "game_plugin.dll");
        assert_eq!(
            index.profiles[0].executable_sha256,
            "3653C97F9CCEC2BE28EDC4FAAE67304B5B6C26733F2F07DEA3E7C591D3B9FF73"
        );
        let coverage = mapping_coverage(&index.profiles[0]);
        assert!(coverage
            .iter()
            .any(|section| section.section == "player" && section.validated >= 7));
        assert!(coverage
            .iter()
            .any(|section| section.section == "contract" && section.unmapped >= 4));
        assert!(coverage
            .iter()
            .any(|section| section.section == "recruitment" && section.candidate >= 2));
    }

    #[test]
    fn snapshot_has_no_entities_when_fm26_is_not_available() {
        #[cfg(not(target_os = "windows"))]
        {
            let snapshot = connector_snapshot();
            assert!(snapshot.players.is_empty());
            assert!(snapshot.clubs.is_empty());
            assert!(snapshot.tactic.is_none());
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn running_exact_build_extracts_live_entities_without_hidden_ability_fields() {
        if find_fm26_process().is_none() {
            return;
        }
        let snapshot = connector_snapshot();
        assert_eq!(
            snapshot.status.state, "connected",
            "{}",
            snapshot.status.message
        );
        assert_eq!(snapshot.manager_name.as_deref(), Some("Tobias Thorsen"));
        assert_eq!(
            snapshot
                .clubs
                .first()
                .and_then(|club| club["name"].as_str()),
            Some("Madla IL")
        );
        assert_eq!(snapshot.players.len(), 38);
        assert!(snapshot.tactic.is_none());
        assert_eq!(snapshot.tactic_source, "none");
        assert_eq!(snapshot.status.live_memory_tactic_read, "object_not_found");
        for player in &snapshot.players {
            assert!(player["currentAbility"].is_null());
            assert!(player["potentialAbility"].is_null());
            assert!(player["attributes"]
                .as_object()
                .is_some_and(|value| !value.is_empty()));
            assert_eq!(player["scoutKnowledge"], "fully_known");
        }
        let lars = snapshot
            .players
            .iter()
            .find(|player| player["id"] == "53179170")
            .expect("Lars Sveingard");
        assert_eq!(lars["age"], 30);
        assert_eq!(lars["dateOfBirth"], "1995-08-13");
        assert_eq!(lars["nationality"], "Norway");
        assert_eq!(lars["attributes"]["Aerial Reach"], 15);
        assert_eq!(lars["attributes"]["Communication"], 12);
        assert!(lars["attributes"].get("Consistency").is_none());
        assert!(lars["attributes"].get("Injury Proneness").is_none());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn full_save_index_exposes_identity_only_background_search_records() {
        if find_fm26_process().is_none() {
            return;
        }
        let snapshot = collect_snapshot(true, None);
        assert_eq!(
            snapshot.status.state, "connected",
            "{}",
            snapshot.status.message
        );
        assert_eq!(snapshot.status.database_index_status, "ready");
        assert_eq!(snapshot.status.database_scope, "full-save-index");
        assert!(snapshot.status.database_players_indexed >= snapshot.status.managed_squad_players);
        assert!(snapshot.status.background_players_indexed > 0);
        assert_eq!(
            snapshot.status.live_memory_tactic_read,
            "object_detected_unmapped"
        );
        assert!(snapshot.status.tactic_manager_pointer.is_some());
        assert_eq!(
            snapshot.status.visible_players_loaded,
            snapshot.players.len() as u32
        );
        let visible_results = search_indexed_players("Jøran".to_string());
        assert!(visible_results
            .iter()
            .all(|player| player["visibility"] == "known" || player["visibility"] == "unknown"));
    }

    #[test]
    fn fm_dates_and_age_match_the_current_save_calendar() {
        let birth = FmDate {
            year: 1995,
            day_of_year: 225,
        };
        let current = FmDate {
            year: 2026,
            day_of_year: 150,
        };
        assert_eq!(format_fm_date(birth).as_deref(), Some("1995-08-13"));
        assert_eq!(format_fm_date(current).as_deref(), Some("2026-05-30"));
        assert_eq!(calculate_age(birth, current), Some(30));
    }

    #[test]
    fn hidden_and_foot_storage_values_never_enter_visible_attributes() {
        let raw = vec![50_u8; PLAYER_ATTRIBUTE_NAMES.len()];
        let attributes = visible_attribute_map(&raw);
        assert_eq!(attributes.get("Crossing"), Some(&10));
        for hidden in [
            "Dirtiness",
            "Consistency",
            "Important Matches",
            "Injury Proneness",
            "Versatility",
            "Left Foot",
            "Right Foot",
        ] {
            assert!(!attributes.contains_key(hidden));
        }
    }

    #[test]
    fn currency_and_enum_transforms_reject_invalid_values() {
        assert_eq!(
            checked_currency_transform(125_000, 1, 1_000_000_000),
            Some(125_000)
        );
        assert_eq!(checked_currency_transform(-1, 1, 1_000_000_000), None);
        assert_eq!(
            checked_currency_transform(2_000_000, 1_000, 1_000_000_000),
            None
        );
        assert_eq!(
            validated_enum(1, &["unknown", "interested", "not_interested"]),
            Some("interested")
        );
        assert_eq!(validated_enum(4, &["unknown", "interested"]), None);
    }

    #[test]
    fn candidates_cannot_cross_the_publication_confidence_gate() {
        let candidate = FieldDefinition {
            offset: Some(32),
            source: "contract".to_string(),
            value_type: "currency".to_string(),
            transform: Some("identity".to_string()),
            status: "candidate".to_string(),
            confidence: 0.94,
            validations: vec!["one observation".to_string()],
        };
        assert!(!field_is_publishable(&candidate));
        let validated = FieldDefinition {
            status: "validated".to_string(),
            confidence: 0.97,
            validations: vec!["100 players".to_string(), "two saves".to_string()],
            ..candidate
        };
        assert!(field_is_publishable(&validated));
    }
}
