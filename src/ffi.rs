use crate::error::Error;
use std::ffi::{CStr, CString, c_char};
use std::slice;

#[link(name = "kubo_ffi", kind = "static")]
unsafe extern "C" {
    // Shared utilities
    fn kubo_ffi_last_error() -> *mut c_char;
    fn kubo_ffi_free_string(s: *mut c_char);
    fn kubo_ffi_free_buffer(buf: *mut u8);

    // Kubo
    fn kubo_version() -> *mut c_char;
    fn kubo_init_repo(path: *const c_char) -> i64;
    fn kubo_node_start(path: *const c_char, online: u8) -> u64;
    fn kubo_node_stop(handle: u64) -> i64;
    fn kubo_node_peer_id(handle: u64) -> *mut c_char;
    fn kubo_node_listening_addrs(handle: u64) -> *mut c_char;
    fn kubo_node_connect(handle: u64, addr: *const c_char) -> i64;
    fn kubo_swarm_peers(handle: u64) -> *mut c_char;
    fn kubo_node_id(handle: u64) -> *mut c_char;
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

    // libp2p
    fn kubo_libp2p_host_new() -> u64;
    fn kubo_libp2p_host_close(handle: u64) -> i64;
    fn kubo_libp2p_host_peer_id(handle: u64) -> *mut c_char;
    fn kubo_libp2p_host_listening_addrs(handle: u64) -> *mut c_char;
    fn kubo_libp2p_host_connect(handle: u64, addr: *const c_char) -> i64;
    fn kubo_libp2p_host_ping(handle: u64, peer_id: *const c_char) -> i64;
    fn kubo_libp2p_host_protocols(handle: u64) -> *mut c_char;

    // nostr
    fn kubo_nostr_generate_key() -> *mut c_char;
    fn kubo_nostr_get_public_key(sk: *const c_char) -> *mut c_char;
    fn kubo_nostr_event_sign(sk: *const c_char, content: *const c_char, kind: i32) -> *mut c_char;
    fn kubo_nostr_event_verify(json_str: *const c_char) -> i64;
    fn kubo_nostr_nip19_encode_pubkey(hex: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_decode_pubkey(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_encode_seckey(hex: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_decode_seckey(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_encode_note(hex: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_decode_note(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip19_encode_entity(
        pubkey: *const c_char,
        kind: i32,
        identifier: *const c_char,
        relays: *const c_char,
    ) -> *mut c_char;
    fn kubo_nostr_nip19_decode_entity(bech32: *const c_char) -> *mut c_char;
    fn kubo_nostr_nip05_verify(identifier: *const c_char, pubkey: *const c_char) -> i64;
    fn kubo_nostr_nip05_query(identifier: *const c_char) -> *mut c_char;
    fn kubo_nostr_relay_connect(url: *const c_char) -> u64;
    fn kubo_nostr_relay_close(handle: u64) -> i64;
    fn kubo_nostr_relay_publish(handle: u64, event_json: *const c_char) -> i64;

    // git
    fn kubo_git_clone(url: *const c_char, path: *const c_char, bare: u8) -> i64;
    fn kubo_git_init(path: *const c_char, bare: u8) -> i64;
    fn kubo_git_open(path: *const c_char) -> u64;
    fn kubo_git_repo_head(handle: u64) -> *mut c_char;
    fn kubo_git_repo_free(handle: u64) -> i64;
    fn kubo_git_repo_is_bare(handle: u64) -> i64;
    fn kubo_git_repo_branches(handle: u64) -> *mut c_char;
    fn kubo_git_repo_remotes(handle: u64) -> *mut c_char;
    fn kubo_git_repo_create_branch(
        handle: u64,
        name: *const c_char,
        commit_hash: *const c_char,
    ) -> i64;
    fn kubo_git_repo_commit_lookup(handle: u64, hash: *const c_char) -> *mut c_char;
    fn kubo_git_repo_tree_entries(handle: u64, hash: *const c_char) -> *mut c_char;
    fn kubo_git_repo_blob_read(
        handle: u64,
        hash: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_git_repo_status(handle: u64) -> *mut c_char;
    fn kubo_git_repo_diff_trees(
        handle: u64,
        old_hash: *const c_char,
        new_hash: *const c_char,
    ) -> *mut c_char;
}

fn check_err(code: i64) -> Result<(), Error> {
    if code == 0 {
        Ok(())
    } else {
        Err(Error::Go(last_error()))
    }
}

fn last_error() -> String {
    unsafe {
        let ptr = kubo_ffi_last_error();
        if ptr.is_null() {
            "unknown error".to_string()
        } else {
            let msg = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            kubo_ffi_free_string(ptr);
            msg
        }
    }
}

fn ptr_to_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        unsafe {
            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            kubo_ffi_free_string(ptr);
            Some(s)
        }
    }
}

// ---------------------------------------------------------------------------
// Kubo
// ---------------------------------------------------------------------------

pub fn version() -> String {
    unsafe { ptr_to_string(kubo_version()).unwrap_or_default() }
}

pub fn init_repo(path: &str) -> Result<(), Error> {
    let c_path = CString::new(path)?;
    unsafe { check_err(kubo_init_repo(c_path.as_ptr())) }
}

pub fn node_start(path: &str, online: bool) -> Result<u64, Error> {
    let c_path = CString::new(path)?;
    let handle = unsafe { kubo_node_start(c_path.as_ptr(), online as u8) };
    if handle == 0 {
        Err(Error::Go(last_error()))
    } else {
        Ok(handle)
    }
}

pub fn node_stop(handle: u64) -> Result<(), Error> {
    unsafe { check_err(kubo_node_stop(handle)) }
}

pub fn node_peer_id(handle: u64) -> Result<String, Error> {
    unsafe { ptr_to_string(kubo_node_peer_id(handle)).ok_or_else(|| Error::Go(last_error())) }
}

pub fn node_listening_addrs(handle: u64) -> Result<Vec<String>, Error> {
    let raw = unsafe {
        ptr_to_string(kubo_node_listening_addrs(handle)).ok_or_else(|| Error::Go(last_error()))?
    };
    Ok(raw.lines().map(|s| s.to_string()).collect())
}

pub fn node_connect(handle: u64, addr: &str) -> Result<(), Error> {
    let c_addr = CString::new(addr)?;
    unsafe { check_err(kubo_node_connect(handle, c_addr.as_ptr())) }
}

pub fn swarm_peers(handle: u64) -> Result<Vec<(String, String)>, Error> {
    let raw =
        unsafe { ptr_to_string(kubo_swarm_peers(handle)).ok_or_else(|| Error::Go(last_error()))? };
    Ok(raw
        .lines()
        .map(|s| {
            let mut parts = s.splitn(2, '\t');
            let id = parts.next().unwrap_or("").to_string();
            let addr = parts.next().unwrap_or("").to_string();
            (id, addr)
        })
        .collect())
}

pub fn node_id(handle: u64) -> Result<String, Error> {
    unsafe { ptr_to_string(kubo_node_id(handle)).ok_or_else(|| Error::Go(last_error())) }
}

pub fn unixfs_add_bytes(handle: u64, data: &[u8]) -> Result<String, Error> {
    unsafe {
        ptr_to_string(kubo_unixfs_add_bytes(handle, data.as_ptr(), data.len()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn unixfs_cat(handle: u64, cid: &str) -> Result<Vec<u8>, Error> {
    let c_cid = CString::new(cid)?;
    unsafe {
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let code = kubo_unixfs_cat(handle, c_cid.as_ptr(), &mut out, &mut out_len);
        check_err(code)?;
        if out.is_null() || out_len == 0 {
            Ok(Vec::new())
        } else {
            let buf = slice::from_raw_parts(out, out_len).to_vec();
            kubo_ffi_free_buffer(out);
            Ok(buf)
        }
    }
}

pub fn block_put(handle: u64, data: &[u8]) -> Result<String, Error> {
    unsafe {
        ptr_to_string(kubo_block_put(handle, data.as_ptr(), data.len()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn block_get(handle: u64, cid: &str) -> Result<Vec<u8>, Error> {
    let c_cid = CString::new(cid)?;
    unsafe {
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let code = kubo_block_get(handle, c_cid.as_ptr(), &mut out, &mut out_len);
        check_err(code)?;
        if out.is_null() || out_len == 0 {
            Ok(Vec::new())
        } else {
            let buf = slice::from_raw_parts(out, out_len).to_vec();
            kubo_ffi_free_buffer(out);
            Ok(buf)
        }
    }
}

pub fn block_stat(handle: u64, cid: &str) -> Result<usize, Error> {
    let c_cid = CString::new(cid)?;
    let size = unsafe { kubo_block_stat(handle, c_cid.as_ptr()) };
    if size < 0 {
        Err(Error::Go(last_error()))
    } else {
        Ok(size as usize)
    }
}

// ---------------------------------------------------------------------------
// libp2p
// ---------------------------------------------------------------------------

pub fn host_new() -> Result<u64, Error> {
    let handle = unsafe { kubo_libp2p_host_new() };
    if handle == 0 {
        Err(Error::Go(last_error()))
    } else {
        Ok(handle)
    }
}

pub fn host_close(handle: u64) -> Result<(), Error> {
    unsafe { check_err(kubo_libp2p_host_close(handle)) }
}

pub fn host_peer_id(handle: u64) -> Result<String, Error> {
    unsafe {
        ptr_to_string(kubo_libp2p_host_peer_id(handle)).ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn host_listening_addrs(handle: u64) -> Result<Vec<String>, Error> {
    let raw = unsafe {
        ptr_to_string(kubo_libp2p_host_listening_addrs(handle))
            .ok_or_else(|| Error::Go(last_error()))?
    };
    Ok(raw.lines().map(|s| s.to_string()).collect())
}

pub fn host_connect(handle: u64, addr: &str) -> Result<(), Error> {
    let c_addr = CString::new(addr)?;
    unsafe { check_err(kubo_libp2p_host_connect(handle, c_addr.as_ptr())) }
}

pub fn host_ping(handle: u64, peer_id: &str) -> Result<i64, Error> {
    let c_peer_id = CString::new(peer_id)?;
    let ms = unsafe { kubo_libp2p_host_ping(handle, c_peer_id.as_ptr()) };
    if ms < 0 {
        Err(Error::Go(last_error()))
    } else {
        Ok(ms)
    }
}

pub fn host_protocols(handle: u64) -> Result<Vec<String>, Error> {
    let raw = unsafe {
        ptr_to_string(kubo_libp2p_host_protocols(handle)).ok_or_else(|| Error::Go(last_error()))?
    };
    Ok(raw.lines().map(|s| s.to_string()).collect())
}

// ---------------------------------------------------------------------------
// nostr
// ---------------------------------------------------------------------------

pub fn generate_key() -> Result<String, Error> {
    unsafe { ptr_to_string(kubo_nostr_generate_key()).ok_or_else(|| Error::Go(last_error())) }
}

pub fn get_public_key(sk: &str) -> Result<String, Error> {
    let c_sk = CString::new(sk)?;
    unsafe {
        ptr_to_string(kubo_nostr_get_public_key(c_sk.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn event_sign(sk: &str, content: &str, kind: i32) -> Result<String, Error> {
    let c_sk = CString::new(sk)?;
    let c_content = CString::new(content)?;
    unsafe {
        ptr_to_string(kubo_nostr_event_sign(
            c_sk.as_ptr(),
            c_content.as_ptr(),
            kind,
        ))
        .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn event_verify(json: &str) -> Result<bool, Error> {
    let c_json = CString::new(json)?;
    match unsafe { kubo_nostr_event_verify(c_json.as_ptr()) } {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(Error::Go(last_error())),
    }
}

pub fn nip19_encode_pubkey(hex: &str) -> Result<String, Error> {
    let c_hex = CString::new(hex)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_encode_pubkey(c_hex.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip19_decode_pubkey(bech32: &str) -> Result<String, Error> {
    let c_bech32 = CString::new(bech32)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_decode_pubkey(c_bech32.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip19_encode_seckey(hex: &str) -> Result<String, Error> {
    let c_hex = CString::new(hex)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_encode_seckey(c_hex.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip19_decode_seckey(bech32: &str) -> Result<String, Error> {
    let c_bech32 = CString::new(bech32)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_decode_seckey(c_bech32.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip19_encode_note(hex: &str) -> Result<String, Error> {
    let c_hex = CString::new(hex)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_encode_note(c_hex.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip19_decode_note(bech32: &str) -> Result<String, Error> {
    let c_bech32 = CString::new(bech32)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_decode_note(c_bech32.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip19_encode_entity(
    pubkey: &str,
    kind: i32,
    identifier: &str,
    relays: &str,
) -> Result<String, Error> {
    let c_pubkey = CString::new(pubkey)?;
    let c_identifier = CString::new(identifier)?;
    let c_relays = CString::new(relays)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_encode_entity(
            c_pubkey.as_ptr(),
            kind,
            c_identifier.as_ptr(),
            c_relays.as_ptr(),
        ))
        .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip19_decode_entity(bech32: &str) -> Result<String, Error> {
    let c_bech32 = CString::new(bech32)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip19_decode_entity(c_bech32.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn nip05_verify(identifier: &str, pubkey: &str) -> Result<bool, Error> {
    let c_identifier = CString::new(identifier)?;
    let c_pubkey = CString::new(pubkey)?;
    match unsafe { kubo_nostr_nip05_verify(c_identifier.as_ptr(), c_pubkey.as_ptr()) } {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(Error::Go(last_error())),
    }
}

pub fn nip05_query(identifier: &str) -> Result<String, Error> {
    let c_identifier = CString::new(identifier)?;
    unsafe {
        ptr_to_string(kubo_nostr_nip05_query(c_identifier.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn relay_connect(url: &str) -> Result<u64, Error> {
    let c_url = CString::new(url)?;
    let handle = unsafe { kubo_nostr_relay_connect(c_url.as_ptr()) };
    if handle == 0 {
        Err(Error::Go(last_error()))
    } else {
        Ok(handle)
    }
}

pub fn relay_close(handle: u64) -> Result<(), Error> {
    unsafe { check_err(kubo_nostr_relay_close(handle)) }
}

pub fn relay_publish(handle: u64, event_json: &str) -> Result<(), Error> {
    let c_json = CString::new(event_json)?;
    unsafe { check_err(kubo_nostr_relay_publish(handle, c_json.as_ptr())) }
}

// ---------------------------------------------------------------------------
// git
// ---------------------------------------------------------------------------

pub fn git_clone_repo(url: &str, path: &str, bare: bool) -> Result<(), Error> {
    let c_url = CString::new(url)?;
    let c_path = CString::new(path)?;
    unsafe { check_err(kubo_git_clone(c_url.as_ptr(), c_path.as_ptr(), bare as u8)) }
}

pub fn git_init_repo(path: &str, bare: bool) -> Result<(), Error> {
    let c_path = CString::new(path)?;
    unsafe { check_err(kubo_git_init(c_path.as_ptr(), bare as u8)) }
}

pub fn git_open_repo(path: &str) -> Result<u64, Error> {
    let c_path = CString::new(path)?;
    let handle = unsafe { kubo_git_open(c_path.as_ptr()) };
    if handle == 0 {
        Err(Error::Go(last_error()))
    } else {
        Ok(handle)
    }
}

pub fn git_repo_head_hash(handle: u64) -> Result<String, Error> {
    unsafe { ptr_to_string(kubo_git_repo_head(handle)).ok_or_else(|| Error::Go(last_error())) }
}

pub fn git_repo_release(handle: u64) -> Result<(), Error> {
    unsafe { check_err(kubo_git_repo_free(handle)) }
}

pub fn git_repo_is_bare(handle: u64) -> Result<bool, Error> {
    match unsafe { kubo_git_repo_is_bare(handle) } {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(Error::Go(last_error())),
    }
}

pub fn git_repo_branches(handle: u64) -> Result<Vec<String>, Error> {
    let raw = unsafe {
        ptr_to_string(kubo_git_repo_branches(handle)).ok_or_else(|| Error::Go(last_error()))?
    };
    Ok(raw.lines().map(|s| s.to_string()).collect())
}

pub fn git_repo_remotes(handle: u64) -> Result<Vec<String>, Error> {
    let raw = unsafe {
        ptr_to_string(kubo_git_repo_remotes(handle)).ok_or_else(|| Error::Go(last_error()))?
    };
    Ok(raw.lines().map(|s| s.to_string()).collect())
}

pub fn git_repo_create_branch(handle: u64, name: &str, commit_hash: &str) -> Result<(), Error> {
    let c_name = CString::new(name)?;
    let c_hash = CString::new(commit_hash)?;
    unsafe {
        check_err(kubo_git_repo_create_branch(
            handle,
            c_name.as_ptr(),
            c_hash.as_ptr(),
        ))
    }
}

pub fn git_repo_commit_lookup(handle: u64, hash: &str) -> Result<String, Error> {
    let c_hash = CString::new(hash)?;
    unsafe {
        ptr_to_string(kubo_git_repo_commit_lookup(handle, c_hash.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))
    }
}

pub fn git_repo_tree_entries(handle: u64, hash: &str) -> Result<Vec<(String, String)>, Error> {
    let c_hash = CString::new(hash)?;
    let raw = unsafe {
        ptr_to_string(kubo_git_repo_tree_entries(handle, c_hash.as_ptr()))
            .ok_or_else(|| Error::Go(last_error()))?
    };
    Ok(raw
        .lines()
        .map(|s| {
            let mut parts = s.splitn(2, '\t');
            let name = parts.next().unwrap_or("").to_string();
            let hash = parts.next().unwrap_or("").to_string();
            (name, hash)
        })
        .collect())
}

pub fn git_repo_blob_read(handle: u64, hash: &str) -> Result<Vec<u8>, Error> {
    let c_hash = CString::new(hash)?;
    unsafe {
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let code = kubo_git_repo_blob_read(handle, c_hash.as_ptr(), &mut out, &mut out_len);
        check_err(code)?;
        if out.is_null() || out_len == 0 {
            Ok(Vec::new())
        } else {
            let buf = slice::from_raw_parts(out, out_len).to_vec();
            kubo_ffi_free_buffer(out);
            Ok(buf)
        }
    }
}

pub fn git_repo_status(handle: u64) -> Result<String, Error> {
    unsafe { ptr_to_string(kubo_git_repo_status(handle)).ok_or_else(|| Error::Go(last_error())) }
}

pub fn git_repo_diff_trees(handle: u64, old_hash: &str, new_hash: &str) -> Result<String, Error> {
    let c_old = CString::new(old_hash)?;
    let c_new = CString::new(new_hash)?;
    unsafe {
        ptr_to_string(kubo_git_repo_diff_trees(
            handle,
            c_old.as_ptr(),
            c_new.as_ptr(),
        ))
        .ok_or_else(|| Error::Go(last_error()))
    }
}
