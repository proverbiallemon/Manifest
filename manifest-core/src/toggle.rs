//! Enable/disable mod files by renaming between the community extension
//! convention: `otr <-> disabled` (MPQ), `o2r <-> di2abled` (ZIP). The
//! disabled extension encodes the original format, so renames round-trip
//! with no side records. This module is the ONLY place mod files are ever
//! renamed; they are never deleted, moved, or content-edited.

use std::path::{Path, PathBuf};

fn target_extension(ext: &str, enabled: bool) -> Option<&'static str> {
    match (ext, enabled) {
        ("otr", false) => Some("disabled"),
        ("o2r", false) => Some("di2abled"),
        ("disabled", true) => Some("otr"),
        ("di2abled", true) => Some("o2r"),
        ("otr", true) | ("o2r", true) | ("disabled", false) | ("di2abled", false) => None,
        _ => None,
    }
}

fn is_known(ext: &str) -> bool {
    matches!(ext, "otr" | "o2r" | "disabled" | "di2abled")
}

/// Rename one mod file to the requested enabled state. Returns the path the
/// file now lives at (unchanged if it was already in that state).
pub fn set_enabled(path: &Path, enabled: bool) -> Result<PathBuf, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !is_known(&ext) {
        return Err(format!(
            "not a mod file (expected otr/o2r/disabled/di2abled): {}",
            path.display()
        ));
    }
    let Some(new_ext) = target_extension(&ext, enabled) else {
        return Ok(path.to_path_buf());
    };
    let target = path.with_extension(new_ext);
    // Check-then-rename is not atomic: on Unix a file created at the target
    // between this check and the rename would be silently overwritten. This
    // is a single-user desktop tool; accepted, not defended.
    if target.exists() {
        return Err(format!(
            "refusing to rename, target already exists: {}",
            target.display()
        ));
    }
    std::fs::rename(path, &target).map_err(|e| format!("{e} (renaming {})", path.display()))?;
    Ok(target)
}

/// Apply a list of (path, enabled) changes in order, stopping at the first
/// failure. The error names the failing path and how many changes applied.
pub fn set_enabled_batch(changes: &[(PathBuf, bool)]) -> Result<Vec<PathBuf>, String> {
    let mut done = Vec::with_capacity(changes.len());
    for (i, (path, enabled)) in changes.iter().enumerate() {
        match set_enabled(path, *enabled) {
            Ok(p) => done.push(p),
            Err(e) => {
                return Err(format!(
                    "{e}; {i} of {} changes applied before this failure",
                    changes.len()
                ))
            }
        }
    }
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn touch(dir: &std::path::Path, name: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, b"x").unwrap();
        p
    }

    #[test]
    fn otr_round_trips_through_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let p = touch(dir.path(), "Sword.otr");
        let off = set_enabled(&p, false).unwrap();
        assert_eq!(off, dir.path().join("Sword.disabled"));
        assert!(!p.exists() && off.exists());
        let on = set_enabled(&off, true).unwrap();
        assert_eq!(on, p);
        assert!(on.exists() && !off.exists());
    }

    #[test]
    fn o2r_round_trips_through_di2abled() {
        let dir = tempfile::tempdir().unwrap();
        let p = touch(dir.path(), "Pack.o2r");
        let off = set_enabled(&p, false).unwrap();
        assert_eq!(off, dir.path().join("Pack.di2abled"));
        let on = set_enabled(&off, true).unwrap();
        assert_eq!(on, dir.path().join("Pack.o2r"));
    }

    #[test]
    fn already_in_requested_state_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let p = touch(dir.path(), "Sword.otr");
        let same = set_enabled(&p, true).unwrap();
        assert_eq!(same, p);
        assert!(p.exists());
    }

    #[test]
    fn unknown_extension_is_rejected_with_path() {
        let dir = tempfile::tempdir().unwrap();
        let p = touch(dir.path(), "notes.txt");
        let err = set_enabled(&p, false).unwrap_err();
        assert!(
            err.contains("notes.txt"),
            "error should name the path: {err}"
        );
    }

    #[test]
    fn existing_target_is_rejected_with_path() {
        let dir = tempfile::tempdir().unwrap();
        let p = touch(dir.path(), "Sword.otr");
        touch(dir.path(), "Sword.disabled");
        let err = set_enabled(&p, false).unwrap_err();
        assert!(
            err.contains("Sword.disabled"),
            "error should name the target: {err}"
        );
        assert!(p.exists(), "source must be untouched on refusal");
    }

    #[test]
    fn batch_stops_at_first_failure_and_reports_progress() {
        let dir = tempfile::tempdir().unwrap();
        let a = touch(dir.path(), "A.otr");
        let bad = touch(dir.path(), "bad.txt");
        let c = touch(dir.path(), "C.o2r");
        let err =
            set_enabled_batch(&[(a.clone(), false), (bad.clone(), false), (c.clone(), false)])
                .unwrap_err();
        assert!(
            err.contains("bad.txt"),
            "error should name the failing path: {err}"
        );
        assert!(
            err.contains("1 of 3"),
            "error should say how many applied: {err}"
        );
        assert!(
            dir.path().join("A.disabled").exists(),
            "first change applied"
        );
        assert!(c.exists(), "changes after the failure must not apply");
    }

    #[test]
    fn batch_happy_path_returns_new_paths() {
        let dir = tempfile::tempdir().unwrap();
        let a = touch(dir.path(), "A.otr");
        let b = touch(dir.path(), "B.o2r");
        let out = set_enabled_batch(&[(a, false), (b, false)]).unwrap();
        assert_eq!(
            out,
            vec![dir.path().join("A.disabled"), dir.path().join("B.di2abled")]
        );
    }
}
