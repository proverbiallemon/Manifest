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
    Ok(value
        .pointer("/CVars/gSettings/EnabledMods")
        .and_then(|v| v.as_str())
        .map(|s| s.split('|').filter(|p| !p.is_empty()).map(String::from).collect())
        .unwrap_or_default())
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
    gsettings["EnabledMods"] = serde_json::Value::String(joined);

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
}
