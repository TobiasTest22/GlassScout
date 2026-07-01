use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path, sync::OnceLock};

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
    data_error: Option<String>,
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

#[tauri::command]
pub fn connector_status() -> ConnectorStatus {
    build_status()
}

#[tauri::command]
pub fn connector_snapshot() -> ConnectorSnapshot {
    let status = build_status();
    let data_error = if status.process_detected && status.executable_header_valid {
        match status.entity_map_status {
            "missing" => format!(
                "FM26 {} is readable, but no verified entity-map profile matches this exact executable. Use Export Watcher for real visible data while a profile is researched.",
                status.game_build.as_deref().unwrap_or("build unknown")
            ),
            "invalid" => "The matching entity map failed signature or pointer validation. Live extraction was stopped before reading entities.".to_string(),
            _ => "The active save could not be validated safely.".to_string(),
        }
    } else {
        status.message.clone()
    };

    ConnectorSnapshot {
        status,
        managed_club_id: None,
        manager_name: None,
        season: None,
        clubs: Vec::new(),
        players: Vec::new(),
        tactic: None,
        data_error: Some(data_error),
    }
}

fn build_status() -> ConnectorStatus {
    let process = find_fm26_process();
    let process_id = process.as_ref().map(|item| item.0);
    let mut process_path = process.as_ref().and_then(|item| item.1.clone());
    let probe = process_id.map(probe_process_memory).unwrap_or_default();
    if process_path.is_none() {
        process_path = probe.process_path.clone();
    }

    let identity = process_path
        .as_deref()
        .map(read_executable_identity)
        .unwrap_or_default();
    let entity_map = find_entity_map(&identity);
    let entity_map_status = if entity_map.is_some() {
        "invalid"
    } else {
        "missing"
    };

    let (state, memory_access, message) = match (process_id, probe.handle_open, probe.header_valid) {
        (None, _, _) => (
            "process_not_found",
            "not_checked",
            "FM26 is not running. Start the game and load a save, then run diagnostics.".to_string(),
        ),
        (Some(_), false, _) => (
            "access_denied",
            "denied",
            "FM26 is running, but GlassScout could not open a read-only process handle.".to_string(),
        ),
        (Some(pid), true, true) if entity_map.is_none() => (
            "parser_unverified",
            "read_only_handle_open",
            format!(
                "FM26 process {pid} is readable. Build {} has no matching verified entity map; live entity traversal is disabled safely.",
                identity.file_version.as_deref().unwrap_or("unknown")
            ),
        ),
        (Some(pid), true, true) => (
            "parser_unverified",
            "read_only_handle_open",
            format!(
                "FM26 process {pid} is readable and a profile matched, but signature and pointer validation are not implemented for that profile."
            ),
        ),
        (Some(pid), true, false) => (
            "parser_unverified",
            "read_only_handle_open",
            format!("FM26 process {pid} opened read-only, but the executable memory probe did not validate."),
        ),
    };

    ConnectorStatus {
        process_detected: process_id.is_some(),
        process_id,
        process_path,
        save_detected: None,
        memory_access,
        parser_status: "unverified",
        state,
        players_loaded: 0,
        clubs_loaded: 0,
        last_sync: None,
        bytes_read: probe.bytes_read,
        executable_header_valid: probe.header_valid,
        can_write_memory: false,
        game_build: identity.file_version,
        product_version: identity.product_version,
        executable_sha256: identity.sha256,
        architecture: identity.architecture,
        module_base: probe.module_base,
        entity_map_status,
        entity_map_profile_id: entity_map.map(|profile| profile.id.clone()),
        pointer_validation: "not_run",
        message,
        warnings: vec![
            "No verified profile means no save-root, player, club or tactic pointers are followed.".to_string(),
            "Export Watcher accepts only user-exported visible columns and blocks hidden-value columns.".to_string(),
        ],
    }
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
        let _declared_shape = (
            profile.module.as_str(),
            profile
                .signatures
                .iter()
                .all(|signature| !signature.name.is_empty() && !signature.pattern.is_empty()),
            profile.pointer_chains.iter().all(|chain| {
                !chain.name.is_empty() && !chain.root.is_empty() && !chain.offsets.is_empty()
            }),
        );
        identity.file_version.as_deref() == Some(profile.file_version.as_str())
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
                    .filter(|v| !v.is_empty())
                    .map(str::to_string);
                identity.product_version = parts
                    .next()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
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

#[derive(Default)]
struct MemoryProbe {
    handle_open: bool,
    bytes_read: usize,
    header_valid: bool,
    process_path: Option<String>,
    module_base: Option<String>,
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

#[cfg(target_os = "windows")]
fn probe_process_memory(process_id: u32) -> MemoryProbe {
    use std::ffi::c_void;
    type Handle = *mut c_void;
    const PROCESS_VM_READ: u32 = 0x0010;
    const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> Handle;
        fn CloseHandle(handle: Handle) -> i32;
        fn QueryFullProcessImageNameW(
            process: Handle,
            flags: u32,
            file_name: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn ReadProcessMemory(
            process: Handle,
            base_address: *const c_void,
            buffer: *mut c_void,
            size: usize,
            bytes_read: *mut usize,
        ) -> i32;
    }
    #[link(name = "psapi")]
    unsafe extern "system" {
        fn EnumProcessModules(
            process: Handle,
            modules: *mut Handle,
            size: u32,
            needed: *mut u32,
        ) -> i32;
    }

    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            0,
            process_id,
        )
    };
    if handle.is_null() {
        return MemoryProbe::default();
    }
    let mut probe = MemoryProbe {
        handle_open: true,
        ..MemoryProbe::default()
    };
    let mut path_buffer = vec![0_u16; 32_768];
    let mut path_size = path_buffer.len() as u32;
    if unsafe { QueryFullProcessImageNameW(handle, 0, path_buffer.as_mut_ptr(), &mut path_size) }
        != 0
    {
        probe.process_path = Some(String::from_utf16_lossy(&path_buffer[..path_size as usize]));
    }
    let mut module: Handle = std::ptr::null_mut();
    let mut needed = 0_u32;
    let module_found = unsafe {
        EnumProcessModules(
            handle,
            &mut module,
            std::mem::size_of::<Handle>() as u32,
            &mut needed,
        )
    } != 0;
    if module_found && !module.is_null() {
        probe.module_base = Some(format!("0x{:X}", module as usize));
        let mut header = [0_u8; 2];
        let mut bytes_read = 0_usize;
        let read_ok = unsafe {
            ReadProcessMemory(
                handle,
                module.cast_const(),
                header.as_mut_ptr().cast(),
                header.len(),
                &mut bytes_read,
            )
        } != 0;
        probe.bytes_read = bytes_read;
        probe.header_valid = read_ok && bytes_read == header.len() && header == *b"MZ";
    }
    unsafe {
        CloseHandle(handle);
    }
    probe
}

#[cfg(not(target_os = "windows"))]
fn probe_process_memory(_process_id: u32) -> MemoryProbe {
    MemoryProbe::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_contract_never_advertises_write_access() {
        assert!(!connector_status().can_write_memory);
    }

    #[test]
    fn unavailable_parser_never_returns_fake_entities() {
        let snapshot = connector_snapshot();
        assert!(snapshot.players.is_empty());
        assert!(snapshot.clubs.is_empty());
        assert!(snapshot.tactic.is_none());
        assert!(snapshot.managed_club_id.is_none());
    }

    #[test]
    fn entity_map_index_has_expected_schema_and_no_unverified_profiles() {
        let index: EntityMapIndex =
            serde_json::from_str(include_str!("../entity-maps/index.json")).unwrap();
        assert_eq!(index.schema_version, 1);
        assert!(index.profiles.is_empty());
    }
}
