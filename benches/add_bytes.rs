use criterion::{Criterion, criterion_group, criterion_main};
use kubo_rs::{Node, init_repo};
use std::path::PathBuf;

fn tmp_dir(name: &str) -> PathBuf {
    let path = PathBuf::from("tmp").join("bench").join(name);
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn bench_add_bytes(c: &mut Criterion) {
    let repo = tmp_dir("add_bytes_1mb").join("repo");
    init_repo(&repo).unwrap();
    let node = Node::start(&repo, false).unwrap();

    let data = vec![0u8; 1024 * 1024]; // 1 MiB

    c.bench_function("add_bytes_1mb", |b| {
        b.iter(|| {
            let _cid = node.add_bytes(&data).unwrap();
        })
    });

    let _ = node.stop();
}

fn bench_add_bytes_small(c: &mut Criterion) {
    let repo = tmp_dir("add_bytes_11b").join("repo");
    init_repo(&repo).unwrap();
    let node = Node::start(&repo, false).unwrap();

    let data = b"hello world";

    c.bench_function("add_bytes_11b", |b| {
        b.iter(|| {
            let _cid = node.add_bytes(data).unwrap();
        })
    });

    let _ = node.stop();
}

criterion_group!(benches, bench_add_bytes, bench_add_bytes_small);
criterion_main!(benches);
