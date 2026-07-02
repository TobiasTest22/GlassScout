#[cfg(target_os = "windows")]
use super::permissions::READ_ONLY_PROCESS_ACCESS;

const MAX_STRING_BYTES: usize = 192;

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
pub(crate) struct ModuleInfo {
    pub(crate) base: u64,
    pub(crate) size: usize,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
pub(crate) struct MemoryRegion {
    pub(crate) base: u64,
    pub(crate) size: usize,
}

#[cfg(target_os = "windows")]
pub(crate) struct ProcessReader {
    handle: Handle,
    pub(crate) bytes_read: usize,
    pub(crate) last_error: Option<u32>,
}

#[cfg(target_os = "windows")]
impl ProcessReader {
    pub(crate) fn open(process_id: u32) -> Result<Self, u32> {
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

    pub(crate) fn process_path(&mut self) -> Option<String> {
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

    pub(crate) fn module(&mut self, wanted_name: &str) -> Option<ModuleInfo> {
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

    pub(crate) fn read_bytes(&mut self, address: u64, size: usize) -> Option<Vec<u8>> {
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

    pub(crate) fn read_pointer(&mut self, address: u64) -> Option<u64> {
        self.read_bytes(address, 8)
            .map(|bytes| u64::from_le_bytes(bytes.try_into().expect("eight bytes")))
    }

    pub(crate) fn read_u32(&mut self, address: u64) -> Option<u32> {
        self.read_bytes(address, 4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    pub(crate) fn read_i32(&mut self, address: u64) -> Option<i32> {
        self.read_bytes(address, 4)
            .map(|bytes| i32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    pub(crate) fn read_length_prefixed_string(&mut self, address: u64) -> Option<String> {
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

    pub(crate) fn readable_private_regions(&mut self, max_scan_bytes: usize) -> Vec<MemoryRegion> {
        const MEM_COMMIT: u32 = 0x1000;
        const MEM_PRIVATE: u32 = 0x20000;
        const PAGE_NOACCESS: u32 = 0x01;
        const PAGE_GUARD: u32 = 0x100;

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
            if readable && considered.saturating_add(size) <= max_scan_bytes {
                regions.push(MemoryRegion { base, size });
                considered = considered.saturating_add(size);
            }
            let next = base.saturating_add(size as u64);
            if next <= address {
                break;
            }
            address = next;
        }
        regions
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
