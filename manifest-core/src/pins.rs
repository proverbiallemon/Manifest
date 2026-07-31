use crate::sort::Pins;
use std::path::{Path, PathBuf};

pub const PINS_SCHEMA_VERSION: u32 = 1;

pub fn default_pins_path(config_path: &Path) -> PathBuf {
    config_path.with_file_name("manifest-pins.json")
}

pub fn read_pins(path: &Path) -> Result<Pins, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Pins::default()),
        Err(e) => return Err(e.to_string()),
    };
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let root = value.as_object().ok_or("pins root is not a JSON object")?;
    if let Some(version) = root.get("schema_version") {
        if !version.is_u64() {
            return Err("pins schema_version is not a number".into());
        }
    }
    let list = |key: &str| -> Result<Vec<String>, String> {
        match root.get(key) {
            None => Ok(Vec::new()),
            Some(serde_json::Value::Array(items)) => items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(String::from)
                        .ok_or_else(|| format!("pins {key} contains a non-string entry"))
                })
                .collect(),
            Some(_) => Err(format!("pins {key} is not an array")),
        }
    };
    Ok(Pins {
        top: list("top")?,
        bottom: list("bottom")?,
    })
}

pub fn write_pins(path: &Path, pins: &Pins) -> Result<(), String> {
    let value = serde_json::json!({
        "schema_version": PINS_SCHEMA_VERSION,
        "top": pins.top,
        "bottom": pins.bottom,
    });
    let dir = path.parent().ok_or("pins path has no parent")?;
    let tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&tmp, &value).map_err(|e| e.to_string())?;
    tmp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pins_path_is_sibling_of_config() {
        let config = Path::new("/some/dir/shipofharkinian.json");
        assert_eq!(
            default_pins_path(config),
            PathBuf::from("/some/dir/manifest-pins.json")
        );
    }

    #[test]
    fn read_missing_file_is_empty_pins() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest-pins.json");
        assert_eq!(read_pins(&path).unwrap(), Pins::default());
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest-pins.json");
        let pins = Pins {
            top: vec!["Alpha".to_string()],
            bottom: vec!["Zulu".to_string(), "Yankee".to_string()],
        };
        write_pins(&path, &pins).unwrap();
        assert_eq!(read_pins(&path).unwrap(), pins);
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["schema_version"], 1);
    }

    #[test]
    fn read_rejects_non_object_root() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest-pins.json");
        std::fs::write(&path, "[1, 2]").unwrap();
        let err = read_pins(&path).unwrap_err();
        assert!(err.contains("root"), "error should name the root: {err}");
    }

    #[test]
    fn read_rejects_non_array_top() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest-pins.json");
        std::fs::write(&path, r#"{"schema_version":1,"top":"Alpha"}"#).unwrap();
        let err = read_pins(&path).unwrap_err();
        assert!(err.contains("top"), "error should name top: {err}");
    }

    #[test]
    fn read_rejects_non_string_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest-pins.json");
        std::fs::write(&path, r#"{"schema_version":1,"bottom":[7]}"#).unwrap();
        let err = read_pins(&path).unwrap_err();
        assert!(err.contains("bottom"), "error should name bottom: {err}");
    }

    #[test]
    fn read_rejects_non_number_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest-pins.json");
        std::fs::write(&path, r#"{"schema_version":"one","top":[]}"#).unwrap();
        let err = read_pins(&path).unwrap_err();
        assert!(
            err.contains("schema_version"),
            "error should name schema_version: {err}"
        );
    }

    #[test]
    fn missing_keys_read_as_empty_lists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest-pins.json");
        std::fs::write(&path, r#"{"schema_version":1}"#).unwrap();
        assert_eq!(read_pins(&path).unwrap(), Pins::default());
    }
}
