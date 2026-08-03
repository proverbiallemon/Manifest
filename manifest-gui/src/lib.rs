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
    let mods = scan_library(mods_dir);
    let order = config::read_order(config_path)?;
    let pin_rules = pins::read_pins(&pins::default_pins_path(config_path))?;
    Ok(report::build(&mods, &order, &pin_rules))
}

fn apply_sort_paths(config_path: &Path, mods_dir: &Path) -> Result<Report, String> {
    let before = scan_paths(config_path, mods_dir)?;
    config::write_order(config_path, &before.proposed_order)?;
    scan_paths(config_path, mods_dir)
}

fn set_pin_paths(
    config_path: &Path,
    mods_dir: &Path,
    mod_name: &str,
    position: Option<&str>,
) -> Result<Report, String> {
    let pins_path = pins::default_pins_path(config_path);
    let current = pins::read_pins(&pins_path)?;
    let mut top: Vec<String> = current.top.into_iter().filter(|n| n != mod_name).collect();
    let mut bottom: Vec<String> = current
        .bottom
        .into_iter()
        .filter(|n| n != mod_name)
        .collect();
    match position {
        Some("top") => top.push(mod_name.to_string()),
        Some("bottom") => bottom.push(mod_name.to_string()),
        None => {}
        Some(other) => return Err(format!("unknown pin position: {other}")),
    }
    pins::write_pins(&pins_path, &Pins { top, bottom })?;
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
fn scan(
    config_path: String,
    mods_dir: Option<String>,
    state: tauri::State<SharedPaths>,
    app: tauri::AppHandle,
) -> Result<Report, String> {
    let config_path = PathBuf::from(config_path);
    let mods_dir = mods_dir.map(PathBuf::from).unwrap_or_else(|| {
        config_path
            .parent()
            .map(|p| p.join("mods"))
            .unwrap_or_else(|| PathBuf::from("mods"))
    });
    let rpt = scan_paths(&config_path, &mods_dir)?;
    {
        let mut guard = state.lock().map_err(|_| "state lock poisoned")?;
        guard.config_path = Some(config_path.clone());
        guard.mods_dir = Some(mods_dir.clone());
    }
    if let Ok(dir) = app.path().app_config_dir() {
        let _ = save_settings_at(
            &dir,
            &StoredSettings {
                config_path: config_path.to_string_lossy().to_string(),
                mods_dir: mods_dir.to_string_lossy().to_string(),
            },
        );
    }
    Ok(rpt)
}

#[tauri::command]
fn apply_sort(state: tauri::State<SharedPaths>) -> Result<Report, String> {
    let (config_path, mods_dir) = stored_paths(&state)?;
    apply_sort_paths(&config_path, &mods_dir)
}

#[tauri::command]
fn set_pin(
    mod_name: String,
    position: Option<String>,
    state: tauri::State<SharedPaths>,
) -> Result<Report, String> {
    let (config_path, mods_dir) = stored_paths(&state)?;
    set_pin_paths(&config_path, &mods_dir, &mod_name, position.as_deref())
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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(SharedPaths::default())
        .invoke_handler(tauri::generate_handler![
            locate_config,
            load_settings,
            scan,
            apply_sort,
            set_pin,
            pick_file,
            pick_folder
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
    fn scan_paths_produces_v2_report() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let report = scan_paths(&config, &mods).unwrap();
        assert_eq!(report.schema_version, 2);
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
    fn set_pin_paths_round_trips_through_pins_file() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let report = set_pin_paths(&config, &mods, "Small", Some("top")).unwrap();
        let small = report.mods.iter().find(|m| m.name == "Small").unwrap();
        assert_eq!(small.pinned.as_deref(), Some("top"));
        assert_eq!(
            report.proposed_order.first().map(String::as_str),
            Some("Small")
        );
        let report = set_pin_paths(&config, &mods, "Small", None).unwrap();
        let small = report.mods.iter().find(|m| m.name == "Small").unwrap();
        assert_eq!(small.pinned, None);
    }

    #[test]
    fn set_pin_paths_rejects_bad_position() {
        let dir = tempfile::tempdir().unwrap();
        let (config, mods) = fixture(dir.path());
        let err = set_pin_paths(&config, &mods, "Small", Some("sideways")).unwrap_err();
        assert!(
            err.contains("position"),
            "error should name position: {err}"
        );
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
}
