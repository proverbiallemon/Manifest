use std::path::Path;

pub mod mpq;
pub mod mpq_crypt;
#[cfg(test)]
pub mod mpq_fixture;

pub use mpq::list_mpq_assets;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    Io(String),
    NotAnArchive(String),
    Unlistable(String),
    Corrupt(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Io(s) => write!(f, "io error: {s}"),
            FormatError::NotAnArchive(s) => write!(f, "not an archive: {s}"),
            FormatError::Unlistable(s) => write!(f, "unlistable: {s}"),
            FormatError::Corrupt(s) => write!(f, "corrupt: {s}"),
        }
    }
}

pub fn list_zip_assets(path: &Path) -> Result<Vec<String>, FormatError> {
    let file = std::fs::File::open(path).map_err(|e| FormatError::Io(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| FormatError::Corrupt(e.to_string()))?;
    let mut out = Vec::new();
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| FormatError::Corrupt(e.to_string()))?;
        if !entry.is_dir() {
            out.push(entry.name().to_string());
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

#[cfg(test)]
mod zip_tests {
    use super::*;
    use std::io::Write;

    fn fixture_zip(dir: &std::path::Path) -> std::path::PathBuf {
        let path = dir.join("fixture.o2r");
        let file = std::fs::File::create(&path).unwrap();
        let mut z = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default();
        z.add_directory("alt/", opts).unwrap();
        z.start_file("alt/objects/gSwordTex", opts).unwrap();
        z.write_all(b"x").unwrap();
        z.start_file("alt/objects/gShieldTex", opts).unwrap();
        z.write_all(b"y").unwrap();
        z.finish().unwrap();
        path
    }

    #[test]
    fn lists_zip_file_entries_without_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = fixture_zip(dir.path());
        let assets = list_zip_assets(&path).unwrap();
        assert_eq!(
            assets,
            vec![
                "alt/objects/gShieldTex".to_string(),
                "alt/objects/gSwordTex".to_string(),
            ]
        );
    }

    #[test]
    fn corrupt_zip_reports_error_not_panic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.o2r");
        std::fs::write(&path, b"PK\x03\x04 not really a zip").unwrap();
        assert!(matches!(
            list_zip_assets(&path),
            Err(FormatError::Corrupt(_))
        ));
    }
}
