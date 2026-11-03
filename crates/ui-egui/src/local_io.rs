//! Bounded local file access; no interpretation of document text as instructions.
pub fn read(path: impl AsRef<std::path::Path>) -> std::io::Result<Vec<u8>> {
    if !pdfcraft_engine::local::ENABLED {
        return std::fs::read(path);
    }
    let path = path.as_ref().to_str().ok_or_else(|| std::io::Error::other("The path is not Unicode"))?;
    pdfcraft_engine::local::read_regular(path).map_err(std::io::Error::other)
}

pub fn open(path: impl AsRef<std::path::Path>) -> std::io::Result<std::fs::File> {
    let path = path.as_ref();
    if pdfcraft_engine::local::ENABLED {
        let name = path.to_str().ok_or_else(|| std::io::Error::other("The path is not Unicode"))?;
        pdfcraft_engine::local::check_path(name).map_err(std::io::Error::other)?;
        let meta = std::fs::metadata(path)?;
        if !meta.is_file() || meta.len() > pdfcraft_engine::local::INPUT_LIMIT {
            return Err(std::io::Error::other("Choose a regular local file up to 128 MiB"));
        }
    }
    let file = std::fs::File::open(path)?;
    if pdfcraft_engine::local::ENABLED {
        let meta = file.metadata()?;
        if !meta.is_file() || meta.len() > pdfcraft_engine::local::INPUT_LIMIT {
            return Err(std::io::Error::other("Choose a regular local file up to 128 MiB"));
        }
    }
    Ok(file)
}
