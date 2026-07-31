use assert_cmd::Command;
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

fn fixture(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let mods = dir.join("mods");
    std::fs::create_dir_all(mods.join("Pack")).unwrap();
    write_zip(&mods.join("Pack/Big.o2r"), &["a", "b"]);
    write_zip(&mods.join("Pack/Small.o2r"), &["a"]);
    let config = dir.join("shipofharkinian.json");
    std::fs::write(
        &config,
        r#"{"CVars":{"gSettings":{"EnabledMods":"Small|Big","AltAssets":1}},"Window":{"Width":1}}"#,
    )
    .unwrap();
    (mods, config)
}

#[test]
fn scan_reports_conflicts_with_exit_code_3() {
    let dir = tempfile::tempdir().unwrap();
    let (mods, config) = fixture(dir.path());
    let output = Command::cargo_bin("manifest")
        .unwrap()
        .args(["scan", "--json"])
        .args(["--mods-dir", mods.to_str().unwrap()])
        .args(["--config", config.to_str().unwrap()])
        .assert()
        .code(3);
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["conflicts"][0]["asset"], "a");
}

#[test]
fn sort_write_fixes_order_and_preserves_config_keys() {
    let dir = tempfile::tempdir().unwrap();
    let (mods, config) = fixture(dir.path());
    Command::cargo_bin("manifest")
        .unwrap()
        .args(["sort", "--write"])
        .args(["--mods-dir", mods.to_str().unwrap()])
        .args(["--config", config.to_str().unwrap()])
        .assert()
        .success();
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
    assert_eq!(value["CVars"]["gSettings"]["EnabledMods"], "Big|Small");
    assert_eq!(value["CVars"]["gSettings"]["AltAssets"], 1);
    assert_eq!(value["Window"]["Width"], 1);
}

#[test]
fn clean_library_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let mods = dir.path().join("mods");
    std::fs::create_dir_all(&mods).unwrap();
    write_zip(&mods.join("Solo.o2r"), &["only"]);
    let config = dir.path().join("shipofharkinian.json");
    std::fs::write(&config, r#"{"CVars":{"gSettings":{"EnabledMods":"Solo"}}}"#).unwrap();
    Command::cargo_bin("manifest")
        .unwrap()
        .args([
            "scan",
            "--mods-dir",
            mods.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
        ])
        .assert()
        .code(0);
}

#[test]
fn sort_without_mode_flag_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let (mods, config) = fixture(dir.path());
    let before = std::fs::read_to_string(&config).unwrap();
    let output = Command::cargo_bin("manifest")
        .unwrap()
        .args(["sort"])
        .args(["--mods-dir", mods.to_str().unwrap()])
        .args(["--config", config.to_str().unwrap()])
        .assert()
        .code(2);
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--dry-run"),
        "usage error should name --dry-run: {stderr}"
    );
    assert!(
        stderr.contains("--write"),
        "usage error should name --write: {stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        before,
        "config must be untouched"
    );
}

#[test]
fn sort_rejects_both_mode_flags() {
    let dir = tempfile::tempdir().unwrap();
    let (mods, config) = fixture(dir.path());
    Command::cargo_bin("manifest")
        .unwrap()
        .args(["sort", "--dry-run", "--write"])
        .args(["--mods-dir", mods.to_str().unwrap()])
        .args(["--config", config.to_str().unwrap()])
        .assert()
        .code(2);
}

#[test]
fn sort_dry_run_proposes_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let (mods, config) = fixture(dir.path());
    let before = std::fs::read_to_string(&config).unwrap();
    let output = Command::cargo_bin("manifest")
        .unwrap()
        .args(["sort", "--dry-run"])
        .args(["--mods-dir", mods.to_str().unwrap()])
        .args(["--config", config.to_str().unwrap()])
        .assert()
        .code(0);
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("dry run"),
        "expected dry run banner: {stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        before,
        "dry run must not write"
    );
}
