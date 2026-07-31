use std::path::{Path, PathBuf};

pub fn default_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from(std::env::var_os("HOME")?)
            .join("Library/Application Support/com.shipofharkinian.soh/shipofharkinian.json"))
    }
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(std::env::var_os("APPDATA")?)
            .join("com.shipofharkinian.soh").join("shipofharkinian.json"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".config")))?;
        Some(base.join("com.shipofharkinian.soh").join("shipofharkinian.json"))
    }
}

pub fn read_order(path: &Path) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let root = value.as_object().ok_or("config root is not a JSON object")?;
    let Some(cvars) = root.get("CVars") else { return Ok(Vec::new()) };
    let cvars = cvars.as_object().ok_or("config CVars is not an object")?;
    let Some(gsettings) = cvars.get("gSettings") else { return Ok(Vec::new()) };
    let gsettings = gsettings.as_object().ok_or("config CVars.gSettings is not an object")?;
    match gsettings.get("EnabledMods") {
        None => Ok(Vec::new()),
        Some(serde_json::Value::String(s)) => {
            Ok(s.split('|').filter(|p| !p.is_empty()).map(String::from).collect())
        }
        Some(_) => Err("config CVars.gSettings.EnabledMods is not a string".into()),
    }
}

pub fn write_order(path: &Path, order: &[String]) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("refusing to write, unparseable config: {e}"))?;
    let joined = order
        .iter()
        .map(|s| s.replace('|', "-"))
        .collect::<Vec<_>>()
        .join("|");
    let gsettings = value
        .pointer_mut("/CVars/gSettings")
        .ok_or("config has no CVars.gSettings object")?;
    let obj = gsettings
        .as_object_mut()
        .ok_or("config CVars.gSettings is not an object")?;
    obj.insert("EnabledMods".to_string(), serde_json::Value::String(joined));

    let dir = path.parent().ok_or("config path has no parent")?;
    let tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&tmp, &value).map_err(|e| e.to_string())?;
    tmp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = r#"{
        "ConfigVersion": "1.0",
        "Window": {"Width": 1920},
        "CVars": {
            "gSettings": {"AltAssets": 1, "EnabledMods": "A|B", "BootSequence": 2},
            "gEnhancements": {"FastText": 1}
        },
        "Audio": {"Master": 0.8}
    }"#;

    #[test]
    fn read_order_parses_pipe_list() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, FULL).unwrap();
        assert_eq!(read_order(&path).unwrap(), vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn write_order_preserves_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, FULL).unwrap();
        write_order(&path, &["B".to_string(), "A".to_string()]).unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["CVars"]["gSettings"]["EnabledMods"], "B|A");
        assert_eq!(value["ConfigVersion"], "1.0");
        assert_eq!(value["Window"]["Width"], 1920);
        assert_eq!(value["CVars"]["gSettings"]["BootSequence"], 2);
        assert_eq!(value["CVars"]["gEnhancements"]["FastText"], 1);
        assert_eq!(value["Audio"]["Master"], 0.8);
    }

    #[test]
    fn write_order_refuses_unparseable_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(write_order(&path, &["A".to_string()]).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
    }

    #[test]
    fn write_order_sanitizes_pipes_in_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, FULL).unwrap();
        write_order(&path, &["Bad|Name".to_string(), "Good".to_string()]).unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["CVars"]["gSettings"]["EnabledMods"], "Bad-Name|Good");
    }

    #[test]
    fn read_order_missing_enabledmods_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, r#"{"CVars":{"gSettings":{"AltAssets":1}}}"#).unwrap();
        assert_eq!(read_order(&path).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn write_order_refuses_missing_gsettings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        let original = r#"{"CVars":{}}"#;
        std::fs::write(&path, original).unwrap();
        assert!(write_order(&path, &["A".to_string()]).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn write_order_refuses_non_object_gsettings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        let original = r#"{"CVars":{"gSettings":"oops"}}"#;
        std::fs::write(&path, original).unwrap();
        assert!(write_order(&path, &["A".to_string()]).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn write_order_preserves_key_order_in_raw_text() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, FULL).unwrap();
        write_order(&path, &["B".to_string(), "A".to_string()]).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let config_idx = text.find("ConfigVersion").expect("ConfigVersion not found");
        let window_idx = text.find("Window").expect("Window not found");
        let cvars_idx = text.find("CVars").expect("CVars not found");
        let audio_idx = text.find("Audio").expect("Audio not found");
        assert!(config_idx < window_idx, "ConfigVersion should come before Window");
        assert!(window_idx < cvars_idx, "Window should come before CVars");
        assert!(cvars_idx < audio_idx, "CVars should come before Audio");
    }

    #[test]
    fn read_order_rejects_non_object_root() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, r#"[1, 2, 3]"#).unwrap();
        let err = read_order(&path).unwrap_err();
        assert!(err.contains("root"), "error should name the root: {err}");
    }

    #[test]
    fn read_order_rejects_non_object_cvars() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, r#"{"CVars": "oops"}"#).unwrap();
        let err = read_order(&path).unwrap_err();
        assert!(err.contains("CVars"), "error should name CVars: {err}");
    }

    #[test]
    fn read_order_rejects_non_object_gsettings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, r#"{"CVars":{"gSettings": 7}}"#).unwrap();
        let err = read_order(&path).unwrap_err();
        assert!(err.contains("gSettings"), "error should name gSettings: {err}");
    }

    #[test]
    fn read_order_rejects_non_string_enabledmods() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, r#"{"CVars":{"gSettings":{"EnabledMods": 42}}}"#).unwrap();
        let err = read_order(&path).unwrap_err();
        assert!(err.contains("EnabledMods"), "error should name EnabledMods: {err}");
    }

    #[test]
    fn read_order_missing_cvars_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("shipofharkinian.json");
        std::fs::write(&path, r#"{"Window":{"Width":1920}}"#).unwrap();
        assert_eq!(read_order(&path).unwrap(), Vec::<String>::new());
    }
}
