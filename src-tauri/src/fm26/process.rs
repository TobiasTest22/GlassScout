#[cfg(target_os = "windows")]
pub(crate) fn find_fm26_process() -> Option<(u32, Option<String>)> {
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
pub(crate) fn find_fm26_process() -> Option<(u32, Option<String>)> {
    None
}
