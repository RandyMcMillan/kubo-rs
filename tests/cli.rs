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
    let output = kubo_rs().args(["ipfs", "version"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('.'), "version should contain a dot");
}

#[test]
fn cli_init_and_peer_id() {
    let repo = tmp_dir("init_and_peer_id").join("repo");

    let init = kubo_rs()
        .args(["ipfs", "init", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let peer_id = kubo_rs()
        .args(["ipfs", "peer-id", repo.to_str().unwrap()])
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
        .args(["ipfs", "init", repo.to_str().unwrap()])
        .output()
        .unwrap();

    let add = kubo_rs()
        .args([
            "ipfs",
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
        .args(["ipfs", "cat", "--repo", repo.to_str().unwrap(), &cid])
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
        .args(["ipfs", "init", repo.to_str().unwrap()])
        .output()
        .unwrap();

    let add = kubo_rs()
        .args([
            "ipfs",
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

#[test]
fn cli_block_put_get_stat() {
    let base = tmp_dir("block_put_get_stat");
    let repo = base.join("repo");
    let file = base.join("block.bin");
    std::fs::write(&file, b"raw block data").unwrap();

    kubo_rs()
        .args(["ipfs", "init", repo.to_str().unwrap()])
        .output()
        .unwrap();

    let put = kubo_rs()
        .args([
            "ipfs",
            "block-put",
            "--repo",
            repo.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(put.status.success());
    let cid = String::from_utf8_lossy(&put.stdout).trim().to_string();
    assert!(!cid.is_empty());

    let stat = kubo_rs()
        .args(["ipfs", "block-stat", "--repo", repo.to_str().unwrap(), &cid])
        .output()
        .unwrap();
    assert!(stat.status.success());
    let size = String::from_utf8_lossy(&stat.stdout)
        .trim()
        .parse::<usize>()
        .unwrap();
    assert_eq!(size, b"raw block data".len());

    let get = kubo_rs()
        .args(["ipfs", "block-get", "--repo", repo.to_str().unwrap(), &cid])
        .output()
        .unwrap();
    assert!(get.status.success());
    assert_eq!(get.stdout, b"raw block data");
}

#[test]
fn cli_config_json() {
    let base = tmp_dir("config_json");
    let repo = base.join("repo");

    kubo_rs()
        .args(["ipfs", "init", repo.to_str().unwrap()])
        .output()
        .unwrap();

    let set = kubo_rs()
        .args([
            "ipfs",
            "config",
            "--repo",
            repo.to_str().unwrap(),
            "--json",
            "API.HTTPHeaders.Access-Control-Allow-Origin",
            "[\"http://localhost:3000\", \"https://webui.ipfs.io\"]",
        ])
        .output()
        .unwrap();
    assert!(
        set.status.success(),
        "config set failed: {}",
        String::from_utf8_lossy(&set.stderr)
    );

    let get = kubo_rs()
        .args([
            "ipfs",
            "config",
            "--repo",
            repo.to_str().unwrap(),
            "--json",
            "API.HTTPHeaders.Access-Control-Allow-Origin",
        ])
        .output()
        .unwrap();
    assert!(get.status.success());
    let stdout = String::from_utf8_lossy(&get.stdout);
    assert!(stdout.contains("http://localhost:3000"));
    assert!(stdout.contains("https://webui.ipfs.io"));
}

#[test]
fn cli_pin_add_rm_ls() {
    let base = tmp_dir("pin_add_rm_ls");
    let repo = base.join("repo");
    let file = base.join("pin.txt");
    std::fs::write(&file, b"pin me").unwrap();

    kubo_rs()
        .args(["ipfs", "init", repo.to_str().unwrap()])
        .output()
        .unwrap();

    let add = kubo_rs()
        .args([
            "ipfs",
            "add",
            "--repo",
            repo.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(add.status.success());
    let cid = String::from_utf8_lossy(&add.stdout).trim().to_string();

    let pin = kubo_rs()
        .args(["ipfs", "pin-add", "--repo", repo.to_str().unwrap(), &cid])
        .output()
        .unwrap();
    assert!(
        pin.status.success(),
        "pin-add failed: {}",
        String::from_utf8_lossy(&pin.stderr)
    );

    let ls = kubo_rs()
        .args(["ipfs", "pin-ls", "--repo", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(ls.status.success());
    let ls_out = String::from_utf8_lossy(&ls.stdout);
    assert!(
        ls_out.contains(&cid),
        "pin-ls should contain the pinned cid"
    );

    let rm = kubo_rs()
        .args(["ipfs", "pin-rm", "--repo", repo.to_str().unwrap(), &cid])
        .output()
        .unwrap();
    assert!(
        rm.status.success(),
        "pin-rm failed: {}",
        String::from_utf8_lossy(&rm.stderr)
    );

    let ls_after = kubo_rs()
        .args(["ipfs", "pin-ls", "--repo", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(ls_after.status.success());
    let ls_after_out = String::from_utf8_lossy(&ls_after.stdout);
    assert!(
        !ls_after_out.contains(&cid),
        "pin-ls after rm should not contain the cid"
    );
}

#[test]
fn cli_p2p_peer_id_and_listen() {
    let repo = tmp_dir("p2p_peer_id_and_listen").join("repo");

    kubo_rs()
        .args(["ipfs", "init", repo.to_str().unwrap()])
        .output()
        .unwrap();

    let peer_id = kubo_rs()
        .args(["p2p", "peer-id", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(peer_id.status.success());
    let id = String::from_utf8_lossy(&peer_id.stdout).trim().to_string();
    assert!(!id.is_empty(), "peer_id should not be empty");

    let listen = kubo_rs()
        .args(["p2p", "listen", "--repo", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(listen.status.success());
    let addrs = String::from_utf8_lossy(&listen.stdout);
    assert!(!addrs.trim().is_empty(), "listen addrs should not be empty");
}
