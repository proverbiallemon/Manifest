use crate::formats::{list_mpq_assets, list_zip_assets};
use crate::model::ModFile;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn folder_gamebanana_id(path: &Path) -> Option<u64> {
    let sidecar = path.parent()?.join(".sailswift.json");
    let text = std::fs::read_to_string(sidecar).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value.get("gameBananaModId")?.as_u64()
}

pub fn scan_library(mods_dir: &Path) -> Vec<ModFile> {
    let mut files = Vec::new();
    collect_files(mods_dir, &mut files);

    let mut mods: Vec<ModFile> = files
        .par_iter()
        .filter_map(|path| {
            let ext = path.extension()?.to_str()?.to_ascii_lowercase();
            let (enabled, is_mpq) = match ext.as_str() {
                "otr" => (true, true),
                "o2r" => (true, false),
                "disabled" => (false, true),
                "di2abled" => (false, false),
                _ => return None,
            };
            let name = path.file_stem()?.to_string_lossy().to_string();
            let listed = if is_mpq { list_mpq_assets(path) } else { list_zip_assets(path) };
            let (assets, error) = match listed {
                Ok(list) => (list.into_iter().collect(), None),
                Err(e) => (Default::default(), Some(e.to_string())),
            };
            Some(ModFile {
                path: path.clone(),
                name,
                enabled,
                assets,
                error,
                gamebanana_mod_id: folder_gamebanana_id(path),
            })
        })
        .collect();
    mods.sort_by(|a, b| a.path.cmp(&b.path));
    mods
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::mpq_fixture::build_mpq;
    use std::io::Write;

    fn write_zip(path: &std::path::Path, entries: &[&str]) {
        let file = std::fs::File::create(path).unwrap();
        let mut z = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default();
        for e in entries {
            z.start_file(*e, opts).unwrap();
            z.write_all(b"x").unwrap();
        }
        z.finish().unwrap();
    }

    #[test]
    fn scans_mixed_library_with_errors_isolated() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("Some Mod");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("Sword.otr"), build_mpq(&[("alt/gA", b"a")])).unwrap();
        write_zip(&folder.join("Shield.o2r"), &["alt/gB"]);
        write_zip(&folder.join("Parked.di2abled"), &["alt/gC"]);
        std::fs::write(folder.join("Broken.otr"), b"garbage").unwrap();
        std::fs::write(
            folder.join(".sailswift.json"),
            br#"{"gameBananaName":"Some Mod","gameBananaModId":123,"downloadedAt":"2026-01-01T00:00:00Z"}"#,
        ).unwrap();

        let mods = scan_library(dir.path());
        assert_eq!(mods.len(), 4);
        let by_name: std::collections::HashMap<_, _> =
            mods.iter().map(|m| (m.name.clone(), m)).collect();
        assert!(by_name["Sword"].enabled);
        assert_eq!(by_name["Sword"].assets.len(), 1);
        assert_eq!(by_name["Sword"].gamebanana_mod_id, Some(123));
        assert!(!by_name["Parked"].enabled);
        assert!(by_name["Broken"].error.is_some());
        assert!(by_name["Broken"].assets.is_empty());
    }
}
