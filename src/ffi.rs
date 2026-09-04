use std::ffi::{CStr, CString, c_char};
use std::slice;

#[link(name = "kubo_ffi", kind = "static")]
unsafe extern "C" {
    fn kubo_version() -> *mut c_char;
    fn kubo_free_string(s: *mut c_char);
    fn kubo_last_error() -> *mut c_char;
    fn kubo_init_repo(path: *const c_char) -> i64;
    fn kubo_node_start(path: *const c_char, online: u8) -> u64;
    fn kubo_node_stop(handle: u64) -> i64;
    fn kubo_node_peer_id(handle: u64) -> *mut c_char;
    fn kubo_unixfs_add_bytes(handle: u64, data: *const u8, length: usize) -> *mut c_char;
    fn kubo_unixfs_cat(
        handle: u64,
        cid_str: *const c_char,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> i64;
    fn kubo_free_buffer(buf: *mut u8);
}

fn check_err(code: i64) -> Result<(), String> {
    if code == 0 { Ok(()) } else { Err(last_error()) }
}

fn last_error() -> String {
    unsafe {
        let ptr = kubo_last_error();
        if ptr.is_null() {
            "unknown error".to_string()
        } else {
            let msg = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            kubo_free_string(ptr);
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
            kubo_free_string(ptr);
            Some(s)
        }
    }
}

pub fn version() -> String {
    unsafe { ptr_to_string(kubo_version()).unwrap_or_default() }
}

pub fn init_repo(path: &str) -> Result<(), String> {
    let c_path = CString::new(path).map_err(|e| e.to_string())?;
    unsafe { check_err(kubo_init_repo(c_path.as_ptr())) }
}

pub fn node_start(path: &str, online: bool) -> Result<u64, String> {
    let c_path = CString::new(path).map_err(|e| e.to_string())?;
    let handle = unsafe { kubo_node_start(c_path.as_ptr(), online as u8) };
    if handle == 0 {
        Err(last_error())
    } else {
        Ok(handle)
    }
}

pub fn node_stop(handle: u64) -> Result<(), String> {
    unsafe { check_err(kubo_node_stop(handle)) }
}

pub fn node_peer_id(handle: u64) -> Result<String, String> {
    unsafe { ptr_to_string(kubo_node_peer_id(handle)).ok_or_else(last_error) }
}

pub fn unixfs_add_bytes(handle: u64, data: &[u8]) -> Result<String, String> {
    unsafe {
        ptr_to_string(kubo_unixfs_add_bytes(handle, data.as_ptr(), data.len()))
            .ok_or_else(last_error)
    }
}

pub fn unixfs_cat(handle: u64, cid: &str) -> Result<Vec<u8>, String> {
    let c_cid = CString::new(cid).map_err(|e| e.to_string())?;
    unsafe {
        let mut out: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let code = kubo_unixfs_cat(handle, c_cid.as_ptr(), &mut out, &mut out_len);
        check_err(code)?;
        if out.is_null() || out_len == 0 {
            Ok(Vec::new())
        } else {
            let buf = slice::from_raw_parts(out, out_len).to_vec();
            kubo_free_buffer(out);
            Ok(buf)
        }
    }
}
