use serde::Serialize;
use serde_json::Value;

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

#[tauri::command]
pub fn connector_status() -> ConnectorStatus {
    build_status()
}

#[tauri::command]
pub fn connector_snapshot() -> ConnectorSnapshot {
    let status = build_status();
    let data_error = if status.process_detected && status.executable_header_valid {
        "FM26 process memory is readable, but this installed build has no verified entity map. Active-save, managed-team, player, club and tactic extraction are therefore disabled.".to_string()
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
        (Some(pid), true, true) => (
            "parser_unverified",
            "read_only_handle_open",
            format!(
                "FM26 process {pid} is readable and its executable header was verified. Save entities are not exposed because this build has no verified entity map."
            ),
        ),
        (Some(pid), true, false) => (
            "parser_unverified",
            "read_only_handle_open",
            format!(
                "FM26 process {pid} opened read-only, but the executable memory probe did not validate."
            ),
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
        message,
        warnings: vec![
            "Active-save detection is unavailable until FM26 entity offsets are verified for the installed build.".to_string(),
            "No player, club, managed-team or tactic values are fabricated when extraction is unavailable.".to_string(),
        ],
    }
}

#[derive(Default)]
struct MemoryProbe {
    handle_open: bool,
    bytes_read: usize,
    header_valid: bool,
    process_path: Option<String>,
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
        let status = connector_status();
        assert!(!status.can_write_memory);
    }

    #[test]
    fn unavailable_parser_never_returns_fake_entities() {
        let snapshot = connector_snapshot();
        assert!(snapshot.players.is_empty());
        assert!(snapshot.clubs.is_empty());
        assert!(snapshot.tactic.is_none());
        assert!(snapshot.managed_club_id.is_none());
    }
}
