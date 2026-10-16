// testrust — standalone Rust test runner for the kubo FFI layer.
// Build the archive first:
//   cd go/ffi && go build -buildmode=c-archive -o tmp/libkubo_ffi.a .
// Then compile and run:
//   cd go/ffi/cmd/testrust
//   rustc main.rs -L ../../tmp -lkubo_ffi -o testrust
// On macOS add: -framework Security -framework CoreFoundation -lresolv -lpthread -ldl
// On Linux add: -lpthread -ldl

use std::ffi::{c_char, CStr, CString};
use std::slice;

#[link(name = "kubo_ffi", kind = "static")]
extern "C" {
    fn kubo_version() -> *mut c_char;
    fn kubo_ffi_last_error() -> *mut c_char;
    fn kubo_ffi_free_string(s: *mut c_char);
    fn kubo_init_repo(repoPath: *const c_char) -> i64;
    fn kubo_node_start(repoPath: *const c_char, online: u8) -> u64;
    fn kubo_node_stop(handle: u64) -> i64;
    fn kubo_node_peer_id(handle: u64) -> *mut c_char;
    fn kubo_node_listening_addrs(handle: u64) -> *mut c_char;
    fn kubo_node_connect(handle: u64, addr: *const c_char) -> i64;
    fn kubo_unixfs_add_bytes(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_unixfs_cat(
        handle: u64,
        cid_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_block_put(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_block_get(
        handle: u64,
        cid_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_block_stat(handle: u64, cid_str: *const c_char) -> i64;
    fn kubo_ffi_free_buffer(buf: *mut u8);
    fn kubo_dag_put(
        handle: u64,
        data: *const u8,
        length: usize,
        input_codec: *const c_char,
        store_codec: *const c_char,
    ) -> *mut c_char;
    fn kubo_dag_get(
        handle: u64,
        cid_str: *const c_char,
        output_codec: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_mfs_ls(handle: u64, path_str: *const c_char) -> *mut c_char;
    fn kubo_mfs_read(
        handle: u64,
        path_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_mfs_write(handle: u64, path_str: *const c_char, data: *const u8, length: usize) -> i64;
    fn kubo_mfs_mkdir(handle: u64, path_str: *const c_char) -> i64;
    fn kubo_mfs_rm(handle: u64, path_str: *const c_char) -> i64;
    fn kubo_mfs_flush(handle: u64, path_str: *const c_char) -> *mut c_char;
    fn kubo_mfs_stat(handle: u64, path_str: *const c_char) -> *mut c_char;
    fn kubo_nostr_generate_key() -> *mut c_char;
    fn kubo_nostr_get_public_key(sk: *const c_char) -> *mut c_char;
    fn kubo_nostr_event_sign(sk: *const c_char, content: *const c_char, kind: i32) -> *mut c_char;
    fn kubo_nostr_event_verify(json_str: *const c_char) -> i64;
    fn kubo_nostr_nip19_encode_pubkey(hex: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_decode_pubkey(bech32: *const c_char) -> *mut c_char;
}

use std::sync::atomic::{AtomicUsize, Ordering};

static FAILURES: AtomicUsize = AtomicUsize::new(0);

fn fail(msg: &str) {
    eprintln!("FAIL: {}", msg);
    FAILURES.fetch_add(1, Ordering::SeqCst);
}

fn ok(msg: &str) {
    println!("OK: {}", msg);
}

fn rmrf(path: &str) {
    let _ = std::fs::remove_dir_all(path);
}

fn test_version() {
    unsafe {
        let v = kubo_version();
        if v.is_null() {
            fail("kubo_version returned null");
            return;
        }
        let s = CStr::from_ptr(v).to_string_lossy();
        if s.is_empty() {
            kubo_ffi_free_string(v);
            fail("version is empty");
            return;
        }
        ok(&format!("version = {}", s));
        kubo_ffi_free_string(v);
    }
}

fn test_init_repo_and_node_lifecycle() {
    let tmp = "./tmp/kubo-rust-test-lifecycle";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            let err = kubo_ffi_last_error();
            let msg = if err.is_null() {
                "unknown".to_string()
            } else {
                let s = CStr::from_ptr(err).to_string_lossy().to_string();
                kubo_ffi_free_string(err);
                s
            };
            fail(&format!("init repo: {}", msg));
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            let err = kubo_ffi_last_error();
            let msg = if err.is_null() {
                "unknown".to_string()
            } else {
                let s = CStr::from_ptr(err).to_string_lossy().to_string();
                kubo_ffi_free_string(err);
                s
            };
            fail(&format!("node start: {}", msg));
            return;
        }

        let peer_id = kubo_node_peer_id(handle);
        if peer_id.is_null() {
            fail("peer_id returned null");
            kubo_node_stop(handle);
            return;
        }
        let id_str = CStr::from_ptr(peer_id).to_string_lossy();
        if id_str.is_empty() {
            kubo_ffi_free_string(peer_id);
            fail("peer_id is empty");
            kubo_node_stop(handle);
            return;
        }
        ok(&format!("peer_id = {}", id_str));
        kubo_ffi_free_string(peer_id);

        if kubo_node_stop(handle) != 0 {
            fail("node stop failed");
            return;
        }
        ok("node lifecycle");
    }
}

fn test_unixfs_add_and_cat() {
    let tmp = "./tmp/kubo-rust-test-unixfs";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let data = b"hello from ffi test";
        let cid = kubo_unixfs_add_bytes(handle, data.as_ptr(), data.len());
        if cid.is_null() {
            fail("add_bytes returned null");
            kubo_node_stop(handle);
            return;
        }
        let cid_str = CStr::from_ptr(cid).to_string_lossy().to_string();
        if cid_str.is_empty() {
            kubo_ffi_free_string(cid);
            fail("cid is empty");
            kubo_node_stop(handle);
            return;
        }

        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        if kubo_unixfs_cat(handle, cid, &mut out, &mut out_len) != 0 {
            kubo_ffi_free_string(cid);
            fail("cat failed");
            kubo_node_stop(handle);
            return;
        }

        let got = if out.is_null() {
            &[][..]
        } else {
            slice::from_raw_parts(out, out_len)
        };
        if got != data {
            fail(&format!(
                "cat: expected {:?}, got {:?}",
                String::from_utf8_lossy(data),
                String::from_utf8_lossy(got)
            ));
        } else {
            ok("unixfs add/cat roundtrip");
        }

        if !out.is_null() {
            kubo_ffi_free_buffer(out);
        }
        kubo_ffi_free_string(cid);
        kubo_node_stop(handle);
    }
}

fn test_block_put_get_stat() {
    let tmp = "./tmp/kubo-rust-test-block";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let data = b"raw block data";
        let cid = kubo_block_put(handle, data.as_ptr(), data.len());
        if cid.is_null() {
            fail("block_put returned null");
            kubo_node_stop(handle);
            return;
        }
        let cid_str = CStr::from_ptr(cid).to_string_lossy().to_string();
        if cid_str.is_empty() {
            kubo_ffi_free_string(cid);
            fail("cid is empty");
            kubo_node_stop(handle);
            return;
        }

        let size = kubo_block_stat(handle, cid);
        if size != data.len() as i64 {
            fail(&format!(
                "block_stat: expected {}, got {}",
                data.len(),
                size
            ));
        }

        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        if kubo_block_get(handle, cid, &mut out, &mut out_len) != 0 {
            kubo_ffi_free_string(cid);
            fail("block_get failed");
            kubo_node_stop(handle);
            return;
        }

        let got = if out.is_null() {
            &[][..]
        } else {
            slice::from_raw_parts(out, out_len)
        };
        if got != data {
            fail(&format!(
                "block_get: expected {:?}, got {:?}",
                String::from_utf8_lossy(data),
                String::from_utf8_lossy(got)
            ));
        } else {
            ok("block put/get/stat roundtrip");
        }

        if !out.is_null() {
            kubo_ffi_free_buffer(out);
        }
        kubo_ffi_free_string(cid);
        kubo_node_stop(handle);
    }
}

fn test_listening_addrs() {
    let tmp = "./tmp/kubo-rust-test-addrs";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 1);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let addrs = kubo_node_listening_addrs(handle);
        if addrs.is_null() {
            fail("listening_addrs returned null");
            kubo_node_stop(handle);
            return;
        }
        let s = CStr::from_ptr(addrs).to_string_lossy();
        if s.is_empty() {
            kubo_ffi_free_string(addrs);
            fail("listening_addrs is empty");
            kubo_node_stop(handle);
            return;
        }
        ok(&format!("listening_addrs = {}", s));
        kubo_ffi_free_string(addrs);
        kubo_node_stop(handle);
    }
}

fn test_hello_world_cidv0_alignment() {
    let tmp = "./tmp/kubo-rust-test-cidv0";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let data = b"hello world";
        let cid = kubo_unixfs_add_bytes(handle, data.as_ptr(), data.len());
        if cid.is_null() {
            fail("add_bytes returned null");
            kubo_node_stop(handle);
            return;
        }
        let cid_str = CStr::from_ptr(cid).to_string_lossy();
        if cid_str != "Qmf412jQZiuVUtdgnB36FXFX7xg5V6KEbSJ4dpQuhkLyfD" {
            fail(&format!(
                "CID mismatch: expected Qmf412jQZiuVUtdgnB36FXFX7xg5V6KEbSJ4dpQuhkLyfD, got {}",
                cid_str
            ));
        } else {
            ok("CIDv0 alignment");
        }

        kubo_ffi_free_string(cid);
        kubo_node_stop(handle);
    }
}

fn test_add_cat_empty() {
    let tmp = "./tmp/kubo-rust-test-empty";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let data: &[u8] = b"";
        let cid = kubo_unixfs_add_bytes(handle, data.as_ptr(), data.len());
        if cid.is_null() {
            fail("add_bytes returned null");
            kubo_node_stop(handle);
            return;
        }

        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        if kubo_unixfs_cat(handle, cid, &mut out, &mut out_len) != 0 {
            kubo_ffi_free_string(cid);
            fail("cat failed");
            kubo_node_stop(handle);
            return;
        }

        let got = if out.is_null() {
            &[][..]
        } else {
            slice::from_raw_parts(out, out_len)
        };
        if got != data {
            fail(&format!(
                "cat: expected empty, got {:?}",
                String::from_utf8_lossy(got)
            ));
        } else {
            ok("empty add/cat roundtrip");
        }

        if !out.is_null() {
            kubo_ffi_free_buffer(out);
        }
        kubo_ffi_free_string(cid);
        kubo_node_stop(handle);
    }
}

fn test_two_nodes_exchange_data() {
    let tmp_a = "./tmp/kubo-rust-test-p2p-a";
    let tmp_b = "./tmp/kubo-rust-test-p2p-b";
    rmrf(tmp_a);
    rmrf(tmp_b);

    unsafe {
        let path_a = CString::new(tmp_a).unwrap();
        let path_b = CString::new(tmp_b).unwrap();
        if kubo_init_repo(path_a.as_ptr()) != 0 || kubo_init_repo(path_b.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }

        let handle_a = kubo_node_start(path_a.as_ptr(), 1);
        let handle_b = kubo_node_start(path_b.as_ptr(), 1);
        if handle_a == 0 || handle_b == 0 {
            fail("node start failed");
            if handle_a != 0 {
                kubo_node_stop(handle_a);
            }
            if handle_b != 0 {
                kubo_node_stop(handle_b);
            }
            return;
        }

        let peer_id_a = kubo_node_peer_id(handle_a);
        let addrs_a = kubo_node_listening_addrs(handle_a);
        if peer_id_a.is_null() || addrs_a.is_null() {
            fail("node_a info missing");
            if !peer_id_a.is_null() {
                kubo_ffi_free_string(peer_id_a);
            }
            if !addrs_a.is_null() {
                kubo_ffi_free_string(addrs_a);
            }
            kubo_node_stop(handle_a);
            kubo_node_stop(handle_b);
            return;
        }

        let peer_id_a_str = CStr::from_ptr(peer_id_a).to_string_lossy().to_string();
        let addrs_a_str = CStr::from_ptr(addrs_a).to_string_lossy();
        let first_addr = addrs_a_str.lines().next().unwrap_or("");
        if first_addr.is_empty() {
            fail("node_a has no addresses");
            kubo_ffi_free_string(peer_id_a);
            kubo_ffi_free_string(addrs_a);
            kubo_node_stop(handle_a);
            kubo_node_stop(handle_b);
            return;
        }
        let dial_addr = format!("{}/p2p/{}", first_addr, peer_id_a_str);
        kubo_ffi_free_string(peer_id_a);
        kubo_ffi_free_string(addrs_a);

        let dial = CString::new(dial_addr).unwrap();
        if kubo_node_connect(handle_b, dial.as_ptr()) != 0 {
            fail("connect b->a failed");
            kubo_node_stop(handle_a);
            kubo_node_stop(handle_b);
            return;
        }

        let data = b"peer-to-peer hello";
        let cid = kubo_unixfs_add_bytes(handle_a, data.as_ptr(), data.len());
        if cid.is_null() {
            fail("add_bytes returned null");
            kubo_node_stop(handle_a);
            kubo_node_stop(handle_b);
            return;
        }

        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        if kubo_unixfs_cat(handle_b, cid, &mut out, &mut out_len) != 0 {
            kubo_ffi_free_string(cid);
            fail("cat from node_b failed");
            kubo_node_stop(handle_a);
            kubo_node_stop(handle_b);
            return;
        }

        let got = if out.is_null() {
            &[][..]
        } else {
            slice::from_raw_parts(out, out_len)
        };
        if got != data {
            fail(&format!(
                "p2p data mismatch: expected {:?}, got {:?}",
                String::from_utf8_lossy(data),
                String::from_utf8_lossy(got)
            ));
        } else {
            ok("two nodes exchange data");
        }

        if !out.is_null() {
            kubo_ffi_free_buffer(out);
        }
        kubo_ffi_free_string(cid);
        kubo_node_stop(handle_a);
        kubo_node_stop(handle_b);
    }
}

fn test_dag_and_mfs() {
    let tmp = "./tmp/kubo-rust-test-dag-mfs";
    rmrf(tmp);

    unsafe {
        let path = CString::new(tmp).unwrap();
        if kubo_init_repo(path.as_ptr()) != 0 {
            fail("init repo failed");
            return;
        }
        let handle = kubo_node_start(path.as_ptr(), 0);
        if handle == 0 {
            fail("node start failed");
            return;
        }

        let json = CString::new(r#"{"hello":"dag"}"#).unwrap();
        let input_codec = CString::new("json").unwrap();
        let store_codec = CString::new("dag-cbor").unwrap();
        let cid = kubo_dag_put(
            handle,
            json.as_ptr() as *const u8,
            json.as_bytes().len(),
            input_codec.as_ptr(),
            store_codec.as_ptr(),
        );
        if cid.is_null() {
            fail("dag_put returned null");
            kubo_node_stop(handle);
            return;
        }

        let output_codec = CString::new("json").unwrap();
        let mut dag_out: *mut u8 = std::ptr::null_mut();
        let mut dag_out_len: usize = 0;
        if kubo_dag_get(
            handle,
            cid,
            output_codec.as_ptr(),
            &mut dag_out,
            &mut dag_out_len,
        ) != 0
        {
            fail("dag_get failed");
            kubo_ffi_free_string(cid);
            kubo_node_stop(handle);
            return;
        }
        let dag_json = if dag_out.is_null() {
            String::new()
        } else {
            String::from_utf8_lossy(slice::from_raw_parts(dag_out, dag_out_len)).into_owned()
        };
        if !dag_json.contains("\"hello\"") || !dag_json.contains("\"dag\"") {
            fail("dag_get returned unexpected JSON");
        } else {
            ok("DAG put/get roundtrip");
        }
        if !dag_out.is_null() {
            kubo_ffi_free_buffer(dag_out);
        }
        kubo_ffi_free_string(cid);

        let mfs_dir = CString::new("/docs").unwrap();
        if kubo_mfs_mkdir(handle, mfs_dir.as_ptr()) != 0 {
            fail("mfs_mkdir failed");
            kubo_node_stop(handle);
            return;
        }
        let mfs_path = CString::new("/docs/readme.txt").unwrap();
        let data = b"created through MFS";
        if kubo_mfs_write(handle, mfs_path.as_ptr(), data.as_ptr(), data.len()) != 0 {
            fail("mfs_write failed to create a file");
            kubo_node_stop(handle);
            return;
        }

        let mut mfs_out: *mut u8 = std::ptr::null_mut();
        let mut mfs_out_len: usize = 0;
        if kubo_mfs_read(handle, mfs_path.as_ptr(), &mut mfs_out, &mut mfs_out_len) != 0 {
            fail("mfs_read failed");
            kubo_node_stop(handle);
            return;
        }
        let got = if mfs_out.is_null() {
            &[][..]
        } else {
            slice::from_raw_parts(mfs_out, mfs_out_len)
        };
        if got != data {
            fail("MFS read returned unexpected content");
        }
        if !mfs_out.is_null() {
            kubo_ffi_free_buffer(mfs_out);
        }

        let updated = b"updated through MFS";
        if kubo_mfs_write(handle, mfs_path.as_ptr(), updated.as_ptr(), updated.len()) != 0 {
            fail("mfs_write failed to update a file");
            kubo_node_stop(handle);
            return;
        }
        mfs_out = std::ptr::null_mut();
        mfs_out_len = 0;
        if kubo_mfs_read(handle, mfs_path.as_ptr(), &mut mfs_out, &mut mfs_out_len) != 0 {
            fail("mfs_read after update failed");
            kubo_node_stop(handle);
            return;
        }
        let got = if mfs_out.is_null() {
            &[][..]
        } else {
            slice::from_raw_parts(mfs_out, mfs_out_len)
        };
        if got != updated {
            fail("MFS update returned unexpected content");
        }
        if !mfs_out.is_null() {
            kubo_ffi_free_buffer(mfs_out);
        }

        let stat = kubo_mfs_stat(handle, mfs_path.as_ptr());
        if stat.is_null() {
            fail("mfs_stat failed");
            kubo_node_stop(handle);
            return;
        }
        kubo_ffi_free_string(stat);

        let entries = kubo_mfs_ls(handle, mfs_dir.as_ptr());
        if entries.is_null()
            || !CStr::from_ptr(entries)
                .to_string_lossy()
                .contains("readme.txt")
        {
            fail("mfs_ls did not list the created file");
            if !entries.is_null() {
                kubo_ffi_free_string(entries);
            }
            kubo_node_stop(handle);
            return;
        }
        kubo_ffi_free_string(entries);

        let flushed = kubo_mfs_flush(handle, mfs_path.as_ptr());
        if flushed.is_null() {
            fail("mfs_flush failed");
            kubo_node_stop(handle);
            return;
        }
        kubo_ffi_free_string(flushed);

        if kubo_mfs_rm(handle, mfs_path.as_ptr()) != 0 {
            fail("mfs_rm failed");
            kubo_node_stop(handle);
            return;
        }
        ok("MFS create/read/update/list/flush/remove");
        kubo_node_stop(handle);
    }
}

fn test_nostr_key_and_event() {
    unsafe {
        let sk = kubo_nostr_generate_key();
        if sk.is_null() {
            fail("nostr key generation failed");
            return;
        }
        let pk = kubo_nostr_get_public_key(sk);
        if pk.is_null() {
            fail("nostr public key derivation failed");
            kubo_ffi_free_string(sk);
            return;
        }
        let npub = kubo_nostr_nip19_encode_pubkey(pk);
        if npub.is_null() {
            fail("NIP-19 public key encoding failed");
            kubo_ffi_free_string(pk);
            kubo_ffi_free_string(sk);
            return;
        }
        let decoded_pk = kubo_nostr_nip19_decode_pubkey(npub);
        if decoded_pk.is_null()
            || CStr::from_ptr(decoded_pk).to_bytes() != CStr::from_ptr(pk).to_bytes()
        {
            fail("NIP-19 public key roundtrip failed");
        }

        let content = CString::new("ffi smoke test").unwrap();
        let event = kubo_nostr_event_sign(sk, content.as_ptr(), 1);
        if event.is_null() || kubo_nostr_event_verify(event) != 1 {
            fail("Nostr event sign/verify failed");
        } else {
            ok("Nostr key, NIP-19, sign/verify");
        }

        if !event.is_null() {
            kubo_ffi_free_string(event);
        }
        if !decoded_pk.is_null() {
            kubo_ffi_free_string(decoded_pk);
        }
        kubo_ffi_free_string(npub);
        kubo_ffi_free_string(pk);
        kubo_ffi_free_string(sk);
    }
}

fn main() {
    println!("=== Rust FFI Test Runner ===");

    test_version();
    test_init_repo_and_node_lifecycle();
    test_unixfs_add_and_cat();
    test_block_put_get_stat();
    test_listening_addrs();
    test_hello_world_cidv0_alignment();
    test_add_cat_empty();
    test_two_nodes_exchange_data();
    test_dag_and_mfs();
    test_nostr_key_and_event();

    println!();
    let failures = FAILURES.load(Ordering::SeqCst);
    if failures > 0 {
        println!("=== {} FAILURE(S) ===", failures);
        std::process::exit(1);
    }
    println!("=== ALL TESTS PASSED ===");
}
