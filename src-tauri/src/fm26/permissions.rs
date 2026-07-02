pub(crate) const READ_ONLY_PROCESS_ACCESS: u32 = 0x1410;
pub(crate) const READ_ONLY_PROCESS_ACCESS_LABEL: &str =
    "PROCESS_QUERY_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ (0x1410)";

pub(crate) fn can_write_memory() -> bool {
    false
}
