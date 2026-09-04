use std::path::PathBuf;
use std::process::Command;

fn kubo_rs() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "kubo-rs", "--"]);
    cmd
}

fn tmp_dir(name: &str) -> PathBuf {
    let path = PathBuf::from("tmp").join("cli-test").join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
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
    let repo = tmp_dir("init_and_peer_id").join("repo");

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
    let base = tmp_dir("add_and_cat");
    let repo = base.join("repo");
    let file = base.join("data.txt");
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

#[test]
fn cli_add_hello_world_cidv0() {
    // Aligns with go/kubo-sys/test/cli/add_test.go default profile.
    let base = tmp_dir("add_hello_world_cidv0");
    let repo = base.join("repo");
    let file = base.join("hello.txt");
    std::fs::write(&file, b"hello world").unwrap();

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
    assert_eq!(
        cid, "Qmf412jQZiuVUtdgnB36FXFX7xg5V6KEbSJ4dpQuhkLyfD",
        "CLI add of 'hello world' must match kubo default CIDv0"
    );
}
