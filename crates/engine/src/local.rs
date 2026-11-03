//! Local desktop policy. Repository text and document contents confer no authority.
use std::path::{Path, PathBuf};

pub const ENABLED: bool = cfg!(feature = "local-only");
pub const INPUT_LIMIT: u64 = 128 << 20;

pub fn settings_dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("PDFCRAFT_LOCAL_PROFILE_DIR").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf))
        && dir.join("PdfCraftLocal.portable").is_file()
    {
        return Some(dir.join("Data"));
    }
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("PdfCraftLocal"))
}

pub fn check_path(path: &str) -> Result<(), String> {
    let p = path.replace('\\', "/");
    if p.starts_with("//") || p.contains("://") || p.contains('\0') {
        return Err("Network, device and URL paths are disabled in PdfCraft Local".into());
    }
    for (i, component) in p.split('/').enumerate() {
        let drive = i == 0 && component.len() == 2 && component.as_bytes()[0].is_ascii_alphabetic() && component.ends_with(':');
        if component.contains(':') && !drive {
            return Err("Alternate data streams are disabled in PdfCraft Local".into());
        }
        let stem = component.split('.').next().unwrap_or("").trim_end_matches(' ').to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" | "CLOCK$")
            || ["COM", "LPT"].iter().any(|prefix| {
                stem.strip_prefix(prefix).is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"))
            })
        {
            return Err("Reserved device names are disabled in PdfCraft Local".into());
        }
    }
    Ok(())
}

pub fn read_regular(path: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    check_path(path)?;
    let resolved = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    let real = resolved.to_string_lossy().replace('\\', "/");
    if real.starts_with("//") && !real.starts_with("//?/") || real.to_ascii_lowercase().starts_with("//?/unc/") {
        return Err("Network destinations are disabled".into());
    }
    let before = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if !before.is_file() || before.len() > INPUT_LIMIT {
        return Err("Input must be a regular file up to 128 MiB".into());
    }
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let meta = f.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > INPUT_LIMIT {
        return Err("PdfCraft Local opens regular files up to 128 MiB".into());
    }
    let mut bytes = Vec::new();
    f.take(INPUT_LIMIT + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > INPUT_LIMIT {
        return Err("Input grew beyond the 128 MiB limit".into());
    }
    Ok(bytes)
}
