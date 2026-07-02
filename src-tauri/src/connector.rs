use serde::{Deserialize, Serialize};
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

const READ_ONLY_PROCESS_ACCESS: u32 = 0x1410;
const MAX_STRING_BYTES: usize = 192;
const POSITION_NAMES: [&str; 15] = [
    "GK", "SW", "DL", "DC", "DR", "DM", "ML", "MC", "MR", "AML", "AMC", "AMR", "ST", "WBL", "WBR",
];
const PLAYER_ATTRIBUTE_NAMES: [&str; 54] = [
    "Crossing",
    "Dribbling",
    "Finishing",
    "Heading",
    "Long Shots",
    "Marking",
    "Off the Ball",
    "Passing",
    "Penalty Taking",
    "Tackling",
    "Vision",
    "Handling",
    "Aerial Reach",
    "Command of Area",
    "Communication",
    "Kicking",
    "Throwing",
    "Anticipation",
    "Decisions",
    "One on Ones",
    "Positioning",
    "Reflexes",
    "First Touch",
    "Technique",
    "Left Foot",
    "Right Foot",
    "Flair",
    "Corners",
    "Teamwork",
    "Work Rate",
    "Long Throws",
    "Eccentricity",
    "Rushing Out",
    "Punching",
    "Acceleration",
    "Free Kick Taking",
    "Strength",
    "Stamina",
    "Pace",
    "Jumping Reach",
    "Leadership",
    "Dirtiness",
    "Balance",
    "Bravery",
    "Consistency",
    "Aggression",
    "Agility",
    "Important Matches",
    "Injury Proneness",
    "Versatility",
    "Natural Fitness",
    "Determination",
    "Composure",
    "Concentration",
];
const HIDDEN_ATTRIBUTE_INDEXES: [usize; 5] = [41, 44, 47, 48, 49];

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
    pointer_validation: &'static str,
    handle_access_flags: &'static str,
    entity_root: Option<String>,
    save_pointer: Option<String>,
    managed_club_pointer: Option<String>,
    player_collection_pointer: Option<String>,
    live_memory_tactic_read: &'static str,
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
    tactic_file_status: &'static str,
    tactic_file_name: Option<String>,
    tactic_file_warnings: Vec<String>,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntityMapIndex {
    schema_version: u32,
    profiles: Vec<EntityMapProfile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntityMapProfile {
    id: String,
    file_version: String,
    product_version: String,
    executable_sha256: String,
    architecture: String,
    module: String,
    signatures: Vec<EntitySignature>,
    pointer_chains: Vec<PointerChain>,
    constants: MapConstants,
}

#[derive(Deserialize)]
struct EntitySignature {
    name: String,
    pattern: String,
}

#[derive(Deserialize)]
struct PointerChain {
    name: String,
    root: String,
    offsets: Vec<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MapConstants {
    manager_registry_vector_offset: u64,
    human_person_offset: u64,
    person_first_name_offset: u64,
    person_second_name_offset: u64,
    person_common_name_offset: u64,
    person_contract_offset: u64,
    contract_team_offset: u64,
    team_vtable_rva: u64,
    team_club_offset: u64,
    team_players_start_offset: u64,
    team_players_end_offset: u64,
    club_vtable_rva: u64,
    club_name_offset: u64,
    entity_uid_offset: u64,
    player_person_offset: u64,
    player_positions_offset: u64,
    player_attributes_offset: u64,
    player_current_date_offset: u64,
    person_nationality_offset: u64,
    nation_name_offset: u64,
    person_birth_date_offset: u64,
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
}

#[derive(Clone)]
struct IndexedPlayerRecord {
    id: String,
    name: String,
    positions: Vec<String>,
    managed_squad: bool,
    visibility_safe: bool,
}

#[derive(Clone, Copy)]
struct FmDate {
    year: u16,
    day_of_year: u16,
}

#[derive(Default)]
struct PlayerDatabaseIndex {
    process_id: u32,
    save_pointer: u64,
    records: HashMap<String, IndexedPlayerRecord>,
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
    if normalized.is_empty() {
        return Vec::new();
    }
    let Some(index) = PLAYER_DATABASE_INDEX.get() else {
        return Vec::new();
    };
    let Ok(index) = index.read() else {
        return Vec::new();
    };
    let _identity = (index.process_id, index.save_pointer);
    index
        .records
        .values()
        .filter(|record| record.visibility_safe)
        .filter(|record| {
            record.name.to_lowercase().contains(&normalized)
                || record
                    .positions
                    .iter()
                    .any(|position| position.to_lowercase().contains(&normalized))
        })
        .take(100)
        .map(|record| {
            json!({
                "id": record.id,
                "name": record.name,
                "positions": record.positions,
                "managedSquad": record.managed_squad,
                "visibility": "club-visible"
            })
        })
        .collect()
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
        can_write_memory: false,
        game_build: None,
        product_version: None,
        executable_sha256: None,
        architecture: None,
        module_base: None,
        entity_map_status: "not_checked",
        entity_map_profile_id: None,
        pointer_validation: "not_run",
        handle_access_flags: "PROCESS_QUERY_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ (0x1410)",
        entity_root: None,
        save_pointer: None,
        managed_club_pointer: None,
        player_collection_pointer: None,
        live_memory_tactic_read: "disabled",
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
        tactic_file_status: "not_imported",
        tactic_file_name: None,
        tactic_file_warnings: Vec::new(),
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

    let Some(profile) = find_entity_map(&identity) else {
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
                tactic_file_status: "not_imported",
                tactic_file_name: None,
                tactic_file_warnings: Vec::new(),
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
    let pattern = parse_pattern(&signature.pattern)?;
    let hits = reader.scan_module(module, &pattern)?;
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
    validate_vtable(
        reader,
        team,
        module.base + profile.constants.team_vtable_rva,
        "managed_team",
    )?;
    let club = reader
        .read_pointer(team + profile.constants.team_club_offset)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            ExtractionFailure::new(
                "managed_club",
                "The managed FM26 club could not be resolved.",
            )
        })?;
    validate_vtable(
        reader,
        club,
        module.base + profile.constants.club_vtable_rva,
        "managed_club",
    )?;
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
        let player_id = uid.to_string();
        managed_player_ids.insert(player_id.clone());
        managed_index_records.push(IndexedPlayerRecord {
            id: player_id.clone(),
            name: name.clone(),
            positions: positions.clone(),
            managed_squad: true,
            visibility_safe: true,
        });
        players.push(json!({
            "id": player_id,
            "name": name,
            "_season": season,
            "age": age,
            "dateOfBirth": date_of_birth,
            "nationality": nationality,
            "secondNationality": null,
            "positions": positions,
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
        }));
    }
    diagnostics.last_successful_read = Some("extract_managed_squad".to_string());

    let (
        database_players_indexed,
        background_players_indexed,
        database_index_status,
        database_scope,
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
                )
            }
            Err(_) => (player_count as u32, 0, "partial", "managed-squad"),
        }
    } else {
        store_player_index(
            process_id,
            diagnostics.save_pointer.unwrap_or_default(),
            managed_index_records,
        );
        (player_count as u32, 0, "not_run", "managed-squad")
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
        "Live-memory tactic reading is disabled. Tactic Evaluation uses only a user-selected local FMF file.".to_string(),
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
    })
}

#[cfg(target_os = "windows")]
struct IndexSummary {
    total: u32,
    background: u32,
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
    managed_player_ids: &HashSet<String>,
    managed_records: Vec<IndexedPlayerRecord>,
) -> Result<IndexSummary, ExtractionFailure> {
    let mut records: HashMap<String, IndexedPlayerRecord> = managed_records
        .into_iter()
        .map(|record| (record.id.clone(), record))
        .collect();
    let candidate_addresses = reader.scan_private_memory_for_pointer(player_vtable)?;

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
    Ok(IndexSummary { total, background })
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
    })
}

fn display_attribute(raw: u8) -> u8 {
    ((raw.saturating_add(4)) / 5).clamp(1, 20)
}

fn visible_attribute_map(raw: &[u8]) -> HashMap<String, u8> {
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

fn preferred_foot_label(left: u8, right: u8) -> &'static str {
    let difference = i16::from(left) - i16::from(right);
    if difference >= 20 {
        "Left"
    } else if difference <= -20 {
        "Right"
    } else {
        "Both"
    }
}

fn default_role_for_position(position: &str) -> String {
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

#[cfg(target_os = "windows")]
fn validate_vtable(
    reader: &mut ProcessReader,
    object: u64,
    expected: u64,
    stage: &'static str,
) -> Result<(), ExtractionFailure> {
    let actual = reader.read_pointer(object).ok_or_else(|| {
        ExtractionFailure::new(stage, "A required FM26 object could not be read.")
    })?;
    if actual != expected {
        return Err(ExtractionFailure::new(
            stage,
            format!(
                "A required FM26 object failed type validation (expected {}, found {}). No data was shown.",
                hex_address(expected),
                hex_address(actual)
            ),
        ));
    }
    Ok(())
}

fn parse_pattern(pattern: &str) -> Result<Vec<Option<u8>>, ExtractionFailure> {
    pattern
        .split_whitespace()
        .map(|token| {
            if token == "??" || token == "?" {
                Ok(None)
            } else {
                u8::from_str_radix(token, 16).map(Some).map_err(|_| {
                    ExtractionFailure::new(
                        "entity_map",
                        "The embedded signature pattern is invalid.",
                    )
                })
            }
        })
        .collect()
}

fn find_entity_map(identity: &ExecutableIdentity) -> Option<&'static EntityMapProfile> {
    static INDEX: OnceLock<EntityMapIndex> = OnceLock::new();
    let index = INDEX.get_or_init(|| {
        serde_json::from_str(include_str!("../entity-maps/index.json"))
            .expect("embedded entity-map index must be valid JSON")
    });
    if index.schema_version != 1 {
        return None;
    }
    index.profiles.iter().find(|profile| {
        let declared_shape_is_valid = !profile.module.is_empty()
            && profile
                .signatures
                .iter()
                .all(|signature| !signature.name.is_empty() && !signature.pattern.is_empty())
            && profile.pointer_chains.iter().all(|chain| {
                !chain.name.is_empty() && !chain.root.is_empty() && !chain.offsets.is_empty()
            });
        declared_shape_is_valid
            && identity.file_version.as_deref() == Some(profile.file_version.as_str())
            && identity.product_version.as_deref() == Some(profile.product_version.as_str())
            && identity.sha256.as_deref() == Some(profile.executable_sha256.as_str())
            && identity.architecture.as_deref() == Some(profile.architecture.as_str())
    })
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

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct ModuleInfo {
    base: u64,
    size: usize,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct MemoryRegion {
    base: u64,
    size: usize,
}

#[cfg(target_os = "windows")]
struct ProcessReader {
    handle: Handle,
    bytes_read: usize,
    last_error: Option<u32>,
}

#[cfg(target_os = "windows")]
impl ProcessReader {
    fn open(process_id: u32) -> Result<Self, u32> {
        let handle = unsafe { OpenProcess(READ_ONLY_PROCESS_ACCESS, 0, process_id) };
        if handle.is_null() {
            return Err(unsafe { GetLastError() });
        }
        Ok(Self {
            handle,
            bytes_read: 0,
            last_error: None,
        })
    }

    fn process_path(&mut self) -> Option<String> {
        let mut buffer = vec![0_u16; 32_768];
        let mut size = buffer.len() as u32;
        if unsafe { QueryFullProcessImageNameW(self.handle, 0, buffer.as_mut_ptr(), &mut size) }
            == 0
        {
            self.last_error = Some(unsafe { GetLastError() });
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..size as usize]))
    }

    fn module(&mut self, wanted_name: &str) -> Option<ModuleInfo> {
        let mut modules = vec![std::ptr::null_mut(); 2048];
        let mut needed = 0_u32;
        if unsafe {
            EnumProcessModulesEx(
                self.handle,
                modules.as_mut_ptr(),
                (modules.len() * std::mem::size_of::<Handle>()) as u32,
                &mut needed,
                0x03,
            )
        } == 0
        {
            self.last_error = Some(unsafe { GetLastError() });
            return None;
        }
        let count = (needed as usize / std::mem::size_of::<Handle>()).min(modules.len());
        for module in modules.into_iter().take(count) {
            let mut name = [0_u16; 1024];
            let name_length = unsafe {
                GetModuleBaseNameW(self.handle, module, name.as_mut_ptr(), name.len() as u32)
            };
            if name_length == 0 {
                continue;
            }
            if String::from_utf16_lossy(&name[..name_length as usize])
                .eq_ignore_ascii_case(wanted_name)
            {
                let mut info = NativeModuleInfo::default();
                if unsafe {
                    GetModuleInformation(
                        self.handle,
                        module,
                        &mut info,
                        std::mem::size_of::<NativeModuleInfo>() as u32,
                    )
                } == 0
                {
                    self.last_error = Some(unsafe { GetLastError() });
                    return None;
                }
                return Some(ModuleInfo {
                    base: info.base_of_dll as u64,
                    size: info.size_of_image as usize,
                });
            }
        }
        None
    }

    fn read_bytes(&mut self, address: u64, size: usize) -> Option<Vec<u8>> {
        let mut buffer = vec![0_u8; size];
        let mut bytes_read = 0_usize;
        let result = unsafe {
            ReadProcessMemory(
                self.handle,
                address as *const std::ffi::c_void,
                buffer.as_mut_ptr().cast(),
                size,
                &mut bytes_read,
            )
        };
        self.bytes_read = self.bytes_read.saturating_add(bytes_read);
        if result == 0 || bytes_read != size {
            self.last_error = Some(unsafe { GetLastError() });
            return None;
        }
        Some(buffer)
    }

    fn read_pointer(&mut self, address: u64) -> Option<u64> {
        self.read_bytes(address, 8)
            .map(|bytes| u64::from_le_bytes(bytes.try_into().expect("eight bytes")))
    }

    fn read_u32(&mut self, address: u64) -> Option<u32> {
        self.read_bytes(address, 4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    fn read_i32(&mut self, address: u64) -> Option<i32> {
        self.read_bytes(address, 4)
            .map(|bytes| i32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    fn read_length_prefixed_string(&mut self, address: u64) -> Option<String> {
        let length = self.read_u32(address)? as usize;
        if length == 0 || length > MAX_STRING_BYTES {
            return None;
        }
        let bytes = self.read_bytes(address + 4, length)?;
        let value = String::from_utf8(bytes).ok()?;
        if value.chars().any(char::is_control) {
            return None;
        }
        Some(value)
    }

    fn scan_module(
        &mut self,
        module: ModuleInfo,
        pattern: &[Option<u8>],
    ) -> Result<Vec<u64>, ExtractionFailure> {
        if pattern.is_empty() || module.size < pattern.len() {
            return Err(ExtractionFailure::new(
                "manager_signature",
                "The manager signature is empty or larger than the game module.",
            ));
        }
        const CHUNK_SIZE: usize = 4 * 1024 * 1024;
        let overlap = pattern.len().saturating_sub(1);
        let mut hits = Vec::new();
        let mut offset = 0_usize;
        while offset < module.size {
            let read_start = offset.saturating_sub(if offset == 0 { 0 } else { overlap });
            let read_size = CHUNK_SIZE
                .saturating_add(if offset == 0 { 0 } else { overlap })
                .min(module.size - read_start);
            let bytes = self
                .read_bytes(module.base + read_start as u64, read_size)
                .ok_or_else(|| {
                    ExtractionFailure::new(
                        "manager_signature",
                        "The FM26 game module could not be scanned with read-only access.",
                    )
                })?;
            for position in 0..=bytes.len().saturating_sub(pattern.len()) {
                let absolute_offset = read_start + position;
                if offset != 0 && absolute_offset < offset {
                    continue;
                }
                if pattern.iter().enumerate().all(|(index, expected)| {
                    expected.is_none_or(|value| bytes[position + index] == value)
                }) {
                    hits.push(module.base + absolute_offset as u64);
                }
            }
            offset = offset.saturating_add(CHUNK_SIZE);
        }
        Ok(hits)
    }

    fn scan_private_memory_for_pointer(
        &mut self,
        pointer: u64,
    ) -> Result<Vec<u64>, ExtractionFailure> {
        const MEM_COMMIT: u32 = 0x1000;
        const MEM_PRIVATE: u32 = 0x20000;
        const PAGE_NOACCESS: u32 = 0x01;
        const PAGE_GUARD: u32 = 0x100;
        const CHUNK_SIZE: usize = 8 * 1024 * 1024;
        const MAX_SCAN_BYTES: usize = 8 * 1024 * 1024 * 1024;

        let mut regions = Vec::new();
        let mut address = 0_u64;
        let mut considered = 0_usize;
        loop {
            let mut info = NativeMemoryInfo::default();
            let queried = unsafe {
                VirtualQueryEx(
                    self.handle,
                    address as *const std::ffi::c_void,
                    &mut info,
                    std::mem::size_of::<NativeMemoryInfo>(),
                )
            };
            if queried == 0 {
                break;
            }
            let base = info.base_address as u64;
            let size = info.region_size;
            if size == 0 {
                break;
            }
            let readable = info.state == MEM_COMMIT
                && info.memory_type == MEM_PRIVATE
                && info.protect & (PAGE_NOACCESS | PAGE_GUARD) == 0;
            if readable && considered.saturating_add(size) <= MAX_SCAN_BYTES {
                regions.push(MemoryRegion { base, size });
                considered = considered.saturating_add(size);
            }
            let next = base.saturating_add(size as u64);
            if next <= address {
                break;
            }
            address = next;
        }
        if regions.is_empty() {
            return Err(ExtractionFailure::new(
                "player_database",
                "No readable FM26 private-memory regions were available for player indexing.",
            ));
        }

        let needle = pointer.to_le_bytes();
        let mut hits = Vec::new();
        for region in regions {
            let mut offset = 0_usize;
            while offset < region.size {
                let size = CHUNK_SIZE.min(region.size - offset);
                let Some(bytes) = self.read_bytes(region.base + offset as u64, size) else {
                    offset = offset.saturating_add(size);
                    continue;
                };
                let base = region.base + offset as u64;
                let first_aligned = ((8 - (base as usize & 7)) & 7).min(bytes.len());
                let mut position = first_aligned;
                while position + needle.len() <= bytes.len() {
                    if bytes[position..position + needle.len()] == needle {
                        hits.push(base + position as u64);
                    }
                    position += 8;
                }
                offset = offset.saturating_add(size);
            }
        }
        Ok(hits)
    }
}

#[cfg(target_os = "windows")]
impl Drop for ProcessReader {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[cfg(target_os = "windows")]
type Handle = *mut std::ffi::c_void;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Default)]
struct NativeModuleInfo {
    base_of_dll: *mut std::ffi::c_void,
    size_of_image: u32,
    entry_point: *mut std::ffi::c_void,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Default)]
struct NativeMemoryInfo {
    base_address: *mut std::ffi::c_void,
    allocation_base: *mut std::ffi::c_void,
    allocation_protect: u32,
    partition_id: u16,
    alignment: u16,
    region_size: usize,
    state: u32,
    protect: u32,
    memory_type: u32,
    alignment_two: u32,
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetLastError() -> u32;
    fn QueryFullProcessImageNameW(
        process: Handle,
        flags: u32,
        file_name: *mut u16,
        size: *mut u32,
    ) -> i32;
    fn ReadProcessMemory(
        process: Handle,
        base_address: *const std::ffi::c_void,
        buffer: *mut std::ffi::c_void,
        size: usize,
        bytes_read: *mut usize,
    ) -> i32;
    fn VirtualQueryEx(
        process: Handle,
        address: *const std::ffi::c_void,
        buffer: *mut NativeMemoryInfo,
        length: usize,
    ) -> usize;
}

#[cfg(target_os = "windows")]
#[link(name = "psapi")]
unsafe extern "system" {
    fn EnumProcessModulesEx(
        process: Handle,
        modules: *mut Handle,
        size: u32,
        needed: *mut u32,
        filter: u32,
    ) -> i32;
    fn GetModuleBaseNameW(process: Handle, module: Handle, base_name: *mut u16, size: u32) -> u32;
    fn GetModuleInformation(
        process: Handle,
        module: Handle,
        module_info: *mut NativeModuleInfo,
        size: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
fn find_fm26_process() -> Option<(u32, Option<String>)> {
    let output = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq fm.exe", "/FO", "CSV", "/NH"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let row = stdout
        .lines()
        .find(|line| line.to_ascii_lowercase().contains("fm.exe"))?;
    let columns: Vec<&str> = row.trim().trim_matches('"').split("\",\"").collect();
    let pid = columns.get(1)?.replace(',', "").parse::<u32>().ok()?;
    Some((pid, None))
}

#[cfg(not(target_os = "windows"))]
fn find_fm26_process() -> Option<(u32, Option<String>)> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_contract_never_requests_or_advertises_write_access() {
        assert_eq!(READ_ONLY_PROCESS_ACCESS, 0x1410);
        assert!(!empty_status().can_write_memory);
    }

    #[test]
    fn exact_build_profile_is_embedded_and_read_only() {
        let index: EntityMapIndex =
            serde_json::from_str(include_str!("../entity-maps/index.json")).unwrap();
        assert_eq!(index.schema_version, 1);
        assert_eq!(index.profiles.len(), 1);
        assert_eq!(index.profiles[0].module, "game_plugin.dll");
        assert_eq!(
            index.profiles[0].executable_sha256,
            "3653C97F9CCEC2BE28EDC4FAAE67304B5B6C26733F2F07DEA3E7C591D3B9FF73"
        );
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
        assert_eq!(snapshot.status.live_memory_tactic_read, "disabled");
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
    fn full_save_index_keeps_unvalidated_background_players_hidden() {
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
            snapshot.status.visible_players_loaded,
            snapshot.players.len() as u32
        );
        let visible_results = search_indexed_players("Jøran".to_string());
        assert!(visible_results
            .iter()
            .all(|player| player["visibility"] == "club-visible"));
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
}
