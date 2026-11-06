use std::{path::PathBuf, process::Command};

struct OutputDirectory(PathBuf);
impl OutputDirectory {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!("mincost-flow-test-{}-{stamp}", std::process::id())))
    }
}
impl Drop for OutputDirectory {
    fn drop(&mut self) {
        if self.0.exists() {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
}
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mincost-flow-potentials"))
}

#[test]
fn demo_exports_trace_and_removes_only_stale_frames() {
    let out = OutputDirectory::new();
    let result = cli()
        .args(["demo", "reassignment", "--out"])
        .arg(&out.0)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let solution: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.0.join("solution.json")).unwrap()).unwrap();
    assert_eq!(solution["cost"], 4);
    assert_eq!(solution["flow"], 2);
    let trace: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.0.join("trace.json")).unwrap()).unwrap();
    assert_eq!(trace["frames"].as_array().unwrap().len(), 5);
    std::fs::write(out.0.join("frame-999.svg"), "stale").unwrap();
    std::fs::write(out.0.join("notes.txt"), "preserve").unwrap();
    let result = cli()
        .args(["demo", "reassignment", "--out"])
        .arg(&out.0)
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(!out.0.join("frame-999.svg").exists());
    assert_eq!(
        std::fs::read_to_string(out.0.join("notes.txt")).unwrap(),
        "preserve"
    );
    assert!(
        std::fs::read_to_string(out.0.join("frame-003.svg"))
            .unwrap()
            .contains("e2R r=1")
    );
}
#[test]
fn verification_reports_skips_and_rejects_bad_arguments() {
    let result = cli()
        .args(["verify", "--cases", "10", "--budget", "0"])
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("skipped=10"));
    for args in [
        vec!["verify", "--cases", "0"],
        vec!["demo", "unknown"],
        vec!["verify", "--unknown", "1"],
        vec!["verify", "--seed", "1", "--seed", "2"],
    ] {
        assert!(!cli().args(args).output().unwrap().status.success());
    }
}
#[test]
fn json_input_and_validation_errors() {
    let out = OutputDirectory::new();
    let result = cli()
        .args(["solve", "examples/transport.json", "--out"])
        .arg(&out.0)
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("cost=17"));
    let input = out.0.join("invalid.json");
    std::fs::write(&input,r#"{"vertices":2,"source":0,"sink":1,"requested_flow":1,"edges":[{"from":0,"to":1,"capacity":1,"cost":-1}]}"#).unwrap();
    assert!(
        !cli()
            .arg("solve")
            .arg(input)
            .output()
            .unwrap()
            .status
            .success()
    );
}
