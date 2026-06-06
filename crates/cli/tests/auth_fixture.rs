use serde_json::Value;
use std::path::Path;
use std::process::Command;

#[test]
fn validate_auth_fixture_end_to_end() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/auth-ts");
    copy_project(&fixture, temp.path());

    let output = Command::new(env!("CARGO_BIN_EXE_sbe"))
        .arg("validate")
        .arg(temp.path())
        .arg("--query")
        .arg("jwt to passport")
        .arg("--json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        Path::new(report["report_path"].as_str().unwrap()),
        temp.path().join(".sbe/reports/validation-latest.json")
    );
    assert!(temp.path().join(".sbe/index.bin").exists());
    assert!(temp
        .path()
        .join(".sbe/reports/validation-latest.json")
        .exists());

    let layers = report["benchmark"]["impacted_layers"].as_array().unwrap();
    assert!(layers.iter().any(|layer| layer["layer"] == "Auth"));
    assert!(layers.iter().any(|layer| layer["layer"] == "Middleware"));
    assert!(layers.iter().any(|layer| layer["layer"] == "Controller"));
    assert!(
        report["benchmark"]["token_estimate"]["without_sbe_tokens"]
            .as_u64()
            .unwrap()
            > 0
    );
}

fn copy_project(source: &Path, destination: &Path) {
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_name() == ".sbe" {
            continue;
        }
        if source_path.is_dir() {
            std::fs::create_dir_all(&destination_path).unwrap();
            copy_project(&source_path, &destination_path);
        } else {
            std::fs::copy(&source_path, &destination_path).unwrap();
        }
    }
}
