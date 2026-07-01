use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const MAX_FMF_FILE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_INSPECTION_BYTES: usize = 2 * 1024 * 1024;
const FMF_CONTAINER_MAGIC: [u8; 6] = [0x02, 0x01, b'a', b'f', b'e', b'.'];
const STATUS_FILE_NAME: &str = "current-tactic.json";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticParserStatus {
    NotImported,
    ImportedUnparsed,
    PartiallyParsed,
    Parsed,
    UnsupportedFormat,
    InvalidFile,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TacticFileResult {
    pub success: bool,
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
    pub imported_at: Option<String>,
    pub parser_status: TacticParserStatus,
    pub parsed_formation: Option<String>,
    pub parsed_roles: Vec<String>,
    pub parsed_duties: Vec<String>,
    pub detected_format: Option<String>,
    pub compressed: Option<bool>,
    pub encoded: Option<bool>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl TacticFileResult {
    fn not_imported() -> Self {
        Self {
            success: false,
            file_name: None,
            file_size: None,
            imported_at: None,
            parser_status: TacticParserStatus::NotImported,
            parsed_formation: None,
            parsed_roles: Vec::new(),
            parsed_duties: Vec::new(),
            detected_format: None,
            compressed: None,
            encoded: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn invalid(message: impl Into<String>) -> Self {
        Self {
            parser_status: TacticParserStatus::InvalidFile,
            errors: vec![message.into()],
            ..Self::not_imported()
        }
    }
}

#[tauri::command]
pub fn import_tactic_file(app: AppHandle, path: String) -> TacticFileResult {
    let source = PathBuf::from(path);
    if !has_fmf_extension(&source) {
        return TacticFileResult::invalid("Only Football Manager .fmf tactic files are accepted.");
    }

    let source = match source.canonicalize() {
        Ok(value) => value,
        Err(error) => {
            return TacticFileResult::invalid(format!(
                "The selected tactic file could not be opened: {error}"
            ));
        }
    };
    let metadata = match source.metadata() {
        Ok(value) if value.is_file() => value,
        Ok(_) => return TacticFileResult::invalid("The selected path is not a file."),
        Err(error) => {
            return TacticFileResult::invalid(format!(
                "The selected tactic file could not be inspected: {error}"
            ));
        }
    };
    if metadata.len() == 0 {
        return TacticFileResult::invalid("The selected FMF tactic file is empty.");
    }
    if metadata.len() > MAX_FMF_FILE_BYTES {
        return TacticFileResult::invalid(format!(
            "The selected FMF tactic is too large. The safe import limit is {} MB.",
            MAX_FMF_FILE_BYTES / 1024 / 1024
        ));
    }

    let file_name = match source.file_name().and_then(|value| value.to_str()) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => {
            return TacticFileResult::invalid("The selected tactic file name is not valid Unicode.")
        }
    };
    let inspection = match read_bounded(&source, metadata.len()) {
        Ok(value) => value,
        Err(error) => {
            return TacticFileResult::invalid(format!(
                "The selected FMF tactic could not be read safely: {error}"
            ));
        }
    };
    let mut result = inspect_fmf(&inspection, metadata.len());
    result.file_name = Some(file_name.clone());
    result.file_size = Some(metadata.len());
    result.imported_at = Some(unix_milliseconds());

    let app_data = match app.path().app_data_dir() {
        Ok(value) => value,
        Err(error) => {
            result.success = false;
            result.errors.push(format!(
                "The GlassScout app-data folder is unavailable: {error}"
            ));
            return result;
        }
    };
    let tactic_dir = app_data.join("tactics");
    if let Err(error) = fs::create_dir_all(&tactic_dir) {
        result.success = false;
        result.errors.push(format!(
            "GlassScout could not create its tactic storage folder: {error}"
        ));
        return result;
    }

    let digest = sha256_file(&source).unwrap_or_else(|| "unhashed".to_string());
    let stored_name = format!(
        "{}-{}-{}",
        result.imported_at.as_deref().unwrap_or("unknown"),
        &digest[..digest.len().min(12)],
        file_name
    );
    let destination = tactic_dir.join(stored_name);
    match fs::copy(&source, &destination) {
        Ok(copied) if copied == metadata.len() => {}
        Ok(copied) => {
            result.success = false;
            result.errors.push(format!(
                "The tactic copy was incomplete: expected {} bytes, stored {copied} bytes.",
                metadata.len()
            ));
            return result;
        }
        Err(error) => {
            result.success = false;
            result.errors.push(format!(
                "GlassScout could not store the tactic in app data: {error}"
            ));
            return result;
        }
    }

    let status_path = tactic_dir.join(STATUS_FILE_NAME);
    let serialized = match serde_json::to_vec_pretty(&result) {
        Ok(value) => value,
        Err(error) => {
            result.success = false;
            result.errors.push(format!(
                "The tactic import status could not be serialized: {error}"
            ));
            return result;
        }
    };
    if let Err(error) = fs::write(status_path, serialized) {
        result.success = false;
        result.errors.push(format!(
            "The tactic import status could not be stored: {error}"
        ));
    }
    result
}

fn has_fmf_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("fmf"))
}

#[tauri::command]
pub fn tactic_file_status(app: AppHandle) -> TacticFileResult {
    let app_data = match app.path().app_data_dir() {
        Ok(value) => value,
        Err(error) => {
            return TacticFileResult::invalid(format!(
                "The GlassScout app-data folder is unavailable: {error}"
            ))
        }
    };
    let path = app_data.join("tactics").join(STATUS_FILE_NAME);
    if !path.exists() {
        return TacticFileResult::not_imported();
    }
    match fs::read(&path)
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<TacticFileResult>(&bytes).map_err(|error| error.to_string())
        }) {
        Ok(result) => result,
        Err(error) => TacticFileResult::invalid(format!(
            "The saved tactic import status is unreadable: {error}"
        )),
    }
}

fn read_bounded(path: &Path, file_size: u64) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let bounded_size = file_size.min(MAX_INSPECTION_BYTES as u64) as usize;
    let mut buffer = vec![0_u8; bounded_size];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn inspect_fmf(bytes: &[u8], file_size: u64) -> TacticFileResult {
    if bytes.is_empty() || file_size == 0 {
        return TacticFileResult::invalid("The selected FMF tactic file is empty.");
    }

    if bytes.starts_with(&FMF_CONTAINER_MAGIC) {
        return TacticFileResult {
            success: true,
            file_name: None,
            file_size: Some(file_size),
            imported_at: None,
            parser_status: TacticParserStatus::ImportedUnparsed,
            parsed_formation: None,
            parsed_roles: Vec::new(),
            parsed_duties: Vec::new(),
            detected_format: Some("sports-interactive-fmf-container".to_string()),
            compressed: Some(true),
            encoded: Some(true),
            warnings: vec![
                "A Sports Interactive FMF container was detected.".to_string(),
                "Tactic file imported, but this FMF format is not fully decoded yet.".to_string(),
            ],
            errors: Vec::new(),
        };
    }

    let (format, compressed) = if bytes.starts_with(&[0x1f, 0x8b]) {
        ("gzip-data", true)
    } else if bytes.starts_with(b"PK\x03\x04") {
        ("zip-data", true)
    } else if bytes.len() >= 2 && bytes[0] == 0x78 && matches!(bytes[1], 0x01 | 0x5e | 0x9c | 0xda)
    {
        ("zlib-data", true)
    } else {
        ("unknown-fmf-bytes", false)
    };
    TacticFileResult {
        success: true,
        file_name: None,
        file_size: Some(file_size),
        imported_at: None,
        parser_status: TacticParserStatus::UnsupportedFormat,
        parsed_formation: None,
        parsed_roles: Vec::new(),
        parsed_duties: Vec::new(),
        detected_format: Some(format.to_string()),
        compressed: Some(compressed),
        encoded: Some(!compressed),
        warnings: vec![
            "The file was stored, but its FMF payload format is not supported safely.".to_string(),
            "No formation, role or duty data was inferred.".to_string(),
        ],
        errors: Vec::new(),
    }
}

fn sha256_file(path: &Path) -> Option<String> {
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
    Some(format!("{:x}", hasher.finalize()))
}

fn unix_milliseconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognized_fmf_container_is_imported_but_never_guessed() {
        let mut bytes = FMF_CONTAINER_MAGIC.to_vec();
        bytes.extend_from_slice(&[0x08, 0, 0, 0xef, 0x02, 0, 0]);
        let result = inspect_fmf(&bytes, bytes.len() as u64);
        assert!(result.success);
        assert!(matches!(
            result.parser_status,
            TacticParserStatus::ImportedUnparsed
        ));
        assert!(result.parsed_formation.is_none());
        assert!(result.parsed_roles.is_empty());
        assert!(result.parsed_duties.is_empty());
    }

    #[test]
    fn unknown_fmf_bytes_do_not_crash_or_create_tactic_data() {
        let result = inspect_fmf(&[0xde, 0xad, 0xbe, 0xef, 0x00, 0xff], 6);
        assert!(result.success);
        assert!(matches!(
            result.parser_status,
            TacticParserStatus::UnsupportedFormat
        ));
        assert!(result.parsed_formation.is_none());
    }

    #[test]
    fn empty_input_is_invalid() {
        let result = inspect_fmf(&[], 0);
        assert!(!result.success);
        assert!(matches!(
            result.parser_status,
            TacticParserStatus::InvalidFile
        ));
    }

    #[test]
    fn only_fmf_extensions_are_accepted() {
        assert!(has_fmf_extension(Path::new("Madla.fmf")));
        assert!(has_fmf_extension(Path::new("Madla.FMF")));
        assert!(!has_fmf_extension(Path::new("Madla.csv")));
        assert!(!has_fmf_extension(Path::new("Madla")));
    }
}
