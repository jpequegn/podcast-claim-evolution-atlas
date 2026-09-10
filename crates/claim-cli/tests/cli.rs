use std::{fs, process::Command};
#[test]
fn complete_cli_flow_and_no_overwrite() {
    let root = tempfile::tempdir().unwrap();
    let doc = root.path().join("doc.json");
    let bin = env!("CARGO_BIN_EXE_atlas");
    let bundle = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/basic.json");
    assert!(Command::new(bin)
        .args(["init", bundle, "--out", doc.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    assert!(!Command::new(bin)
        .args(["init", bundle, "--out", doc.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    for args in [
        vec!["validate", doc.to_str().unwrap()],
        vec!["analyze", doc.to_str().unwrap()],
        vec!["timeline", doc.to_str().unwrap(), "task"],
        vec!["report", doc.to_str().unwrap(), "--as-of", "2026-09-10"],
    ] {
        assert!(Command::new(bin)
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
    let request = root.path().join("request.json");
    fs::write(&request,r#"{"revision":0,"action":"approve_claim","target":"claim-1","relation":null,"reviewer":"operator","note":"Evidence checked","evidence_ids":["ev-1"],"at":"2026-09-10","elapsed_seconds":12}"#).unwrap();
    let after = root.path().join("after.json");
    assert!(Command::new(bin)
        .args([
            "review",
            doc.to_str().unwrap(),
            request.to_str().unwrap(),
            "--out",
            after.to_str().unwrap()
        ])
        .status()
        .unwrap()
        .success());
    assert!(Command::new(bin)
        .args(["diff", doc.to_str().unwrap(), after.to_str().unwrap()])
        .output()
        .unwrap()
        .status
        .success());
}
