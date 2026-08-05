use manifest_core::report::Report;
use manifest_core::sort::Pins;
use manifest_core::{config, pins, report, scan::scan_library};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct AppPaths {
    config_path: Option<PathBuf>,
    mods_dir: Option<PathBuf>,
}

type SharedPaths = Mutex<AppPaths>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredSettings {
    pub config_path: String,
    pub mods_dir: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModToggle {
    pub path: String,
    pub enabled: bool,
}

const SETTINGS_FILE: &str = "settings.json";

fn load_settings_at(dir: &Path) -> Option<StoredSettings> {
    let text = std::fs::read_to_string(dir.join(SETTINGS_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_settings_at(dir: &Path, settings: &StoredSettings) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(SETTINGS_FILE), text).map_err(|e| e.to_string())
}

fn scan_paths(config_path: &Path, mods_dir: &Path) -> Result<Report, String> {
    let context = |e: String| {
        format!(
            "{e} (config: {}, mods: {})",
            config_path.display(),
            mods_dir.display()
        )
    };
    if !mods_dir.is_dir() {
        return Err(context("mods directory not found".to_string()));
    }
    let mods = scan_library(mods_dir);
    let order = config::read_order(config_path).map_err(&context)?;
    let pin_rules = pins::read_pins(&pins::default_pins_path(config_path)).map_err(&context)?;
    Ok(report::build(&mods, &order, &pin_rules))
}

fn apply_sort_paths(config_path: &Path, mods_dir: &Path) -> Result<Report, String> {
    let before = scan_paths(config_path, mods_dir)?;
    config::write_order(config_path, &before.proposed_order)
        .map_err(|e| format!("{e} (config: {})", config_path.display()))?;
    scan_paths(config_path, mods_dir)
}

fn reorder_paths(
    config_path: &Path,
    mods_dir: &Path,
    fore: &[String],
    free: &[String],
    aft: &[String],
) -> Result<Report, String> {
    let context = |e: String| {
        format!(
            "{e} (config: {}, mods: {})",
            config_path.display(),
            mods_dir.display()
        )
    };
    if !mods_dir.is_dir() {
        return Err(context("mods directory not found".to_string()));
    }
    let mods = scan_library(mods_dir);
    let order = config::read_order(config_path).map_err(&context)?;
    let enabled: std::collections::BTreeSet<&str> = mods
        .iter()
        .filter(|m| m.enabled)
        .map(|m| m.name.as_str())
        .collect();
    // The loaded ledger: names that are both on disk enabled and in the
    // current order. Stale names fail this filter and are dropped below.
    let loaded: std::collections::BTreeSet<&str> = order
        .iter()
        .map(String::as_str)
        .filter(|n| enabled.contains(n))
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    for name in fore.iter().chain(free).chain(aft) {
        if !seen.insert(name.as_str()) {
            return Err(context(format!(
                "{name} appears twice in the new order; take inventory and retry"
            )));
        }
        if !loaded.contains(name.as_str()) {
            return Err(context(format!(
                "{name} is not in the loaded ledger; take inventory and retry"
            )));
        }
    }
    if let Some(missing) = loaded.iter().find(|n| !seen.contains(*n)) {
        return Err(context(format!(
            "{missing} is missing from the new order; take inventory and retry"
        )));
    }
    let new_order: Vec<String> = fore.iter().chain(free).chain(aft).cloned().collect();
    config::write_order(config_path, &new_order).map_err(&context)?;
    let pins_path = pins::default_pins_path(config_path);
    pins::write_pins(
        &pins_path,
        &Pins {
            top: fore.to_vec(),
            bottom: aft.to_vec(),
        },
    )
    .map_err(|e| format!("{e} (pins: {})", pins_path.display()))?;
    scan_paths(config_path, mods_dir)
}

fn set_mods_enabled_paths(
    config_path: &Path,
    mods_dir: &Path,
    changes: &[ModToggle],
) -> Result<Report, String> {
    let list: Vec<(PathBuf, bool)> = changes
        .iter()
        .map(|c| (PathBuf::from(&c.path), c.enabled))
        .collect();
    manifest_core::toggle::set_enabled_batch(&list)?;
    scan_paths(config_path, mods_dir)
}

fn stored_paths(state: &SharedPaths) -> Result<(PathBuf, PathBuf), String> {
    let guard = state.lock().map_err(|_| "state lock poisoned")?;
    match (&guard.config_path, &guard.mods_dir) {
        (Some(c), Some(m)) => Ok((c.clone(), m.clone())),
        _ => Err("no library loaded yet; scan first".to_string()),
    }
}

#[tauri::command]
fn locate_config() -> Option<String> {
    config::default_config_path()
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Option<StoredSettings> {
    let dir = app.path().app_config_dir().ok()?;
    load_settings_at(&dir)
}

#[tauri::command]
async fn scan(
    config_path: String,
    mods_dir: Option<String>,
    state: tauri::State<'_, SharedPaths>,
    app: tauri::AppHandle,
) -> Result<Report, String> {
    let config_path = PathBuf::from(config_path);
    let mods_dir = mods_dir.map(PathBuf::from).unwrap_or_else(|| {
        config_path
            .parent()
            .map(|p| p.join("mods"))
            .unwrap_or_else(|| PathBuf::from("mods"))
    });
    let (c, m) = (config_path.clone(), mods_dir.clone());
    let rpt = tauri::async_runtime::spawn_blocking(move || scan_paths(&c, &m))
        .await
        .map_err(|e| e.to_string())??;
    {
        let mut guard = state.lock().map_err(|_| "state lock poisoned")?;
        guard.config_path = Some(config_path.clone());
        guard.mods_dir = Some(mods_dir.clone());
    }
    if let Ok(dir) = app.path().app_config_dir() {
        if let Err(e) = save_settings_at(
            &dir,
            &StoredSettings {
                config_path: config_path.to_string_lossy().to_string(),
                mods_dir: mods_dir.to_string_lossy().to_string(),
            },
        ) {
            eprintln!("manifest-gui: failed to save settings: {e}");
        }
    }
    Ok(rpt)
}

#[tauri::command]
async fn apply_sort(state: tauri::State<'_, SharedPaths>) -> Result<Report, String> {
    let (config_path, mods_dir) = stored_paths(&state)?;
    tauri::async_runtime::spawn_blocking(move || apply_sort_paths(&config_path, &mods_dir))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn reorder(
    fore: Vec<String>,
    free: Vec<String>,
    aft: Vec<String>,
    state: tauri::State<'_, SharedPaths>,
) -> Result<Report, String> {
    let (config_path, mods_dir) = stored_paths(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        reorder_paths(&config_path, &mods_dir, &fore, &free, &aft)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_mods_enabled(
    changes: Vec<ModToggle>,
    state: tauri::State<'_, SharedPaths>,
) -> Result<Report, String> {
    let (config_path, mods_dir) = stored_paths(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        set_mods_enabled_paths(&config_path, &mods_dir, &changes)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pick_file(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_file()
        .map(|f| f.to_string())
}

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|f| f.to_string())
}

#[tauri::command]
async fn reveal_item(path: String) -> Result<(), String> {
    tauri_plugin_opener::reveal_item_in_dir(PathBuf::from(&path))
        .map_err(|e| format!("{e} (revealing {path})"))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(SharedPaths::default())
        .invoke_handler(tauri::generate_handler![
            locate_config,
            load_settings,
            scan,
            apply_sort,
            reorder,
            set_mods_enabled,
            pick_file,
            pick_folder,
            reveal_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running manifest-gui");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(dir: &std::path::Path) -> std::path::PathBuf {
        let config = dir.join("shipofharkinian.json");
        std::fs::write(
            &config,
            r#"{"CVars":{"gSettings":{"EnabledMods":"Small|Big"}}}"#,
        )
        .unwrap();
        config
    }

    fn write_zip(path: &std::path::Path, entries: &[&str]) {
        use std::io::Write;
        let file = std::fs::File::create(path).unwrap();
        let mut z = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default();
        for e in entries {
            z.start_file(*e, opts).unwrap();
            z.write_all(b"x").unwrap();
        }
        z.finish().unwrap();
    }

    fn fixture(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
        let mods = dir.join("mods");
        std::fs::create_dir_all(&mods).unwrap();
        write_zip(&mods.join("Big.o2r"), &["a", "b"]);
        write_zip(&mods.join("Small.o2r"), &["a"]);
        (write_config(dir), mods)
    }

    #[test]
    fn scan_paths_produces_current_schema_report() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let report = scan_paths(&config, &mods).unwrap();
        assert_eq!(report.schema_version, 3);
        assert_eq!(report.mods.len(), 2);
        assert_eq!(
            report.proposed_order.first().map(String::as_str),
            Some("Big")
        );
    }

    #[test]
    fn apply_sort_paths_writes_proposed_order() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let report = apply_sort_paths(&config, &mods).unwrap();
        let written = manifest_core::config::read_order(&config).unwrap();
        assert_eq!(written, report.current_order);
        assert_eq!(written, vec!["Big".to_string(), "Small".to_string()]);
    }

    #[test]
    fn settings_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let stored = StoredSettings {
            config_path: "/a/shipofharkinian.json".to_string(),
            mods_dir: "/a/mods".to_string(),
        };
        save_settings_at(dir.path(), &stored).unwrap();
        assert_eq!(load_settings_at(dir.path()), Some(stored));
        assert_eq!(load_settings_at(&dir.path().join("nope")), None);
    }

    #[test]
    fn scan_errors_name_the_paths_involved() {
        let dir = tempfile::tempdir().unwrap();
        let missing_config = dir.path().join("shipofharkinian.json");
        let mods = dir.path().join("mods");
        std::fs::create_dir_all(&mods).unwrap();
        let err = scan_paths(&missing_config, &mods).unwrap_err();
        assert!(
            err.contains("shipofharkinian.json"),
            "error should include the config path: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn apply_sort_errors_name_the_config_path() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
        let result = apply_sort_paths(&config, &mods);
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        let err = result.unwrap_err();
        assert!(
            err.contains("shipofharkinian.json"),
            "error should include the config path: {err}"
        );
    }

    #[test]
    fn scan_errors_when_mods_dir_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("shipofharkinian.json");
        std::fs::write(&config, r#"{"CVars":{"gSettings":{"EnabledMods":"A"}}}"#).unwrap();
        let missing = dir.path().join("mods");
        let err = scan_paths(&config, &missing).unwrap_err();
        assert!(
            err.contains("mods directory not found"),
            "error should say the mods dir is missing: {err}"
        );
        assert!(
            err.contains("mods"),
            "error should include the mods path: {err}"
        );
    }

    #[test]
    fn set_mods_enabled_paths_disables_and_rescans() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let small = mods.join("Small.o2r");
        let report = set_mods_enabled_paths(
            &config,
            &mods,
            &[ModToggle {
                path: small.to_string_lossy().to_string(),
                enabled: false,
            }],
        )
        .unwrap();
        let m = report.mods.iter().find(|m| m.name == "Small").unwrap();
        assert!(!m.enabled);
        assert!(mods.join("Small.di2abled").exists());
    }

    #[test]
    fn set_mods_enabled_paths_failure_names_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        std::fs::write(mods.join("junk.txt"), b"x").unwrap();
        let err = set_mods_enabled_paths(
            &config,
            &mods,
            &[ModToggle {
                path: mods.join("junk.txt").to_string_lossy().to_string(),
                enabled: false,
            }],
        )
        .unwrap_err();
        assert!(
            err.contains("junk.txt"),
            "error should name the path: {err}"
        );
    }

    #[test]
    fn reorder_paths_writes_concatenated_order_and_pins() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let report = reorder_paths(
            &config,
            &mods,
            &["Big".to_string()],
            &[],
            &["Small".to_string()],
        )
        .unwrap();
        let written = manifest_core::config::read_order(&config).unwrap();
        assert_eq!(written, vec!["Big".to_string(), "Small".to_string()]);
        assert_eq!(report.current_order, written);
        let pins_file = manifest_core::pins::read_pins(&pins::default_pins_path(&config)).unwrap();
        assert_eq!(pins_file.top, vec!["Big".to_string()]);
        assert_eq!(pins_file.bottom, vec!["Small".to_string()]);
        let big = report.mods.iter().find(|m| m.name == "Big").unwrap();
        assert_eq!(big.pinned.as_deref(), Some("top"));
    }

    #[test]
    fn reorder_paths_drops_stale_names_from_the_order() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        std::fs::write(
            &config,
            r#"{"CVars":{"gSettings":{"EnabledMods":"Small|Stale|Big"}}}"#,
        )
        .unwrap();
        reorder_paths(
            &config,
            &mods,
            &[],
            &["Big".to_string(), "Small".to_string()],
            &[],
        )
        .unwrap();
        let written = manifest_core::config::read_order(&config).unwrap();
        assert_eq!(written, vec!["Big".to_string(), "Small".to_string()]);
    }

    #[test]
    fn reorder_paths_rejects_unknown_and_duplicate_names() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let err = reorder_paths(
            &config,
            &mods,
            &[],
            &["Big".to_string(), "Nobody".to_string(), "Small".to_string()],
            &[],
        )
        .unwrap_err();
        assert!(err.contains("Nobody"), "error should name the mod: {err}");
        let err = reorder_paths(
            &config,
            &mods,
            &["Big".to_string()],
            &["Big".to_string(), "Small".to_string()],
            &[],
        )
        .unwrap_err();
        assert!(
            err.contains("Big"),
            "error should name the duplicate: {err}"
        );
        assert!(err.contains("twice"), "error should say twice: {err}");
    }

    #[test]
    fn reorder_paths_rejects_a_missing_loaded_mod() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let err = reorder_paths(&config, &mods, &[], &["Big".to_string()], &[]).unwrap_err();
        assert!(
            err.contains("Small"),
            "error should name the missing mod: {err}"
        );
        // Nothing was written on failure.
        let order = manifest_core::config::read_order(&config).unwrap();
        assert_eq!(order, vec!["Small".to_string(), "Big".to_string()]);
    }

    #[test]
    fn reorder_paths_errors_name_the_config_path() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let err = reorder_paths(&config, &mods, &[], &["Big".to_string()], &[]).unwrap_err();
        assert!(
            err.contains("shipofharkinian.json"),
            "error should include the config path: {err}"
        );
    }

    #[test]
    fn reorder_errors_name_the_pins_path() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let pins_dir = pins::default_pins_path(&config);
        // Make the pins path unwritable by occupying it with a directory.
        std::fs::create_dir(&pins_dir).unwrap();
        let err = reorder_paths(
            &config,
            &mods,
            &[],
            &["Small".to_string(), "Big".to_string()],
            &[],
        )
        .unwrap_err();
        assert!(
            err.contains("manifest-pins.json"),
            "error should include the pins path: {err}"
        );
    }
}
