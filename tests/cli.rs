use std::process::Command;

fn kubo_rs() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "kubo-rs", "--"]);
    cmd
}

#[test]
fn cli_version() {
    let output = kubo_rs().arg("version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('.'), "version should contain a dot");
}

#[test]
fn cli_init_and_peer_id() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");

    let init = kubo_rs()
        .args(["init", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let peer_id = kubo_rs()
        .args(["peer-id", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(peer_id.status.success());
    let stdout = String::from_utf8_lossy(&peer_id.stdout);
    assert!(!stdout.trim().is_empty(), "peer_id should not be empty");
}

#[test]
fn cli_add_and_cat() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let file = tmp.path().join("data.txt");
    std::fs::write(&file, b"cli test data").unwrap();

    kubo_rs()
        .args(["init", repo.to_str().unwrap()])
        .output()
        .unwrap();

    let add = kubo_rs()
        .args([
            "add",
            "--repo",
            repo.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(add.status.success());
    let cid = String::from_utf8_lossy(&add.stdout).trim().to_string();
    assert!(!cid.is_empty());

    let cat = kubo_rs()
        .args(["cat", "--repo", repo.to_str().unwrap(), &cid])
        .output()
        .unwrap();
    assert!(cat.status.success());
    assert_eq!(cat.stdout, b"cli test data");
}
