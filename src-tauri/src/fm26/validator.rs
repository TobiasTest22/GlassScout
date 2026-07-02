#[cfg(target_os = "windows")]
use super::memory::ProcessReader;

#[cfg(target_os = "windows")]
pub(crate) fn validate_vtable(
    reader: &mut ProcessReader,
    object: u64,
    expected: u64,
) -> Result<(), String> {
    let actual = reader
        .read_pointer(object)
        .ok_or_else(|| "A required FM26 object could not be read.".to_string())?;
    if actual != expected {
        return Err(format!(
            "A required FM26 object failed type validation (expected {}, found {}). No data was shown.",
            hex_address(expected),
            hex_address(actual)
        ));
    }
    Ok(())
}

fn hex_address(value: u64) -> String {
    format!("0x{value:X}")
}
