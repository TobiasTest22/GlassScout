#[cfg(target_os = "windows")]
use super::memory::{ModuleInfo, ProcessReader};
#[cfg(target_os = "windows")]
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct PatternParseError;

#[derive(Debug)]
pub(crate) struct ScanError {
    message: String,
}

impl ScanError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

pub(crate) fn parse_pattern(pattern: &str) -> Result<Vec<Option<u8>>, PatternParseError> {
    pattern
        .split_whitespace()
        .map(|token| {
            if token == "??" || token == "?" {
                Ok(None)
            } else {
                u8::from_str_radix(token, 16)
                    .map(Some)
                    .map_err(|_| PatternParseError)
            }
        })
        .collect()
}

#[cfg(target_os = "windows")]
pub(crate) fn scan_module(
    reader: &mut ProcessReader,
    module: ModuleInfo,
    pattern: &[Option<u8>],
) -> Result<Vec<u64>, ScanError> {
    if pattern.is_empty() || module.size < pattern.len() {
        return Err(ScanError::new(
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
        let bytes = reader
            .read_bytes(module.base + read_start as u64, read_size)
            .ok_or_else(|| {
                ScanError::new("The FM26 game module could not be scanned with read-only access.")
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

#[cfg(target_os = "windows")]
pub(crate) fn scan_private_memory_for_pointers(
    reader: &mut ProcessReader,
    pointers: &[u64],
) -> Result<HashMap<u64, Vec<u64>>, ScanError> {
    const CHUNK_SIZE: usize = 8 * 1024 * 1024;
    const MAX_SCAN_BYTES: usize = 8 * 1024 * 1024 * 1024;

    let regions = reader.readable_private_regions(MAX_SCAN_BYTES);
    if regions.is_empty() {
        return Err(ScanError::new(
            "No readable FM26 private-memory regions were available for player indexing.",
        ));
    }

    let needles: Vec<(u64, [u8; 8])> = pointers
        .iter()
        .copied()
        .map(|pointer| (pointer, pointer.to_le_bytes()))
        .collect();
    let mut hits: HashMap<u64, Vec<u64>> = pointers
        .iter()
        .copied()
        .map(|pointer| (pointer, Vec::new()))
        .collect();
    for region in regions {
        let mut offset = 0_usize;
        while offset < region.size {
            let size = CHUNK_SIZE.min(region.size - offset);
            let Some(bytes) = reader.read_bytes(region.base + offset as u64, size) else {
                offset = offset.saturating_add(size);
                continue;
            };
            let base = region.base + offset as u64;
            let first_aligned = ((8 - (base as usize & 7)) & 7).min(bytes.len());
            let mut position = first_aligned;
            while position + 8 <= bytes.len() {
                for (pointer, needle) in &needles {
                    if bytes[position..position + 8] == *needle {
                        hits.entry(*pointer)
                            .or_default()
                            .push(base + position as u64);
                    }
                }
                position += 8;
            }
            offset = offset.saturating_add(size);
        }
    }
    Ok(hits)
}
