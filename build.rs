use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let target = env::var("TARGET").expect("TARGET not set");
    let host = env::var("HOST").expect("HOST not set");

    // Only support host builds for now. CGO cross-compilation requires a
    // matching C cross-toolchain, which is beyond the scope of this build
    // script. If you need cross-compilation, set up a proper cross toolchain
    // and override this script.
    if target != host {
        panic!(
            "kubo-rs FFI build script does not support cross-compilation. \
             TARGET ({}) != HOST ({}). \
             To cross-compile, you must set up a C cross-toolchain and \
             manually compile kubo-sys/ffi/ with CGO_ENABLED=1.",
            target, host
        );
    }

    // Verify the submodule is present.
    let kubo_sys = PathBuf::from("kubo-sys");
    if !kubo_sys.join("go.mod").exists() {
        panic!(
            "kubo-sys/go.mod not found. \
             Run: git submodule update --init --recursive"
        );
    }

    // Find Go binary.
    let go = env::var("GO").unwrap_or_else(|_| "go".to_string());
    let go_version = Command::new(&go)
        .args(["version"])
        .output()
        .expect("failed to run `go version`");
    if !go_version.status.success() {
        panic!(
            "`go version` failed. Make sure Go is installed and in PATH. \
             Kubo requires Go >= 1.26.5."
        );
    }

    // Pin GOTOOLCHAIN to the Go version declared in kubo-sys/go.mod so that
    // the build is reproducible even when the host has a newer Go installed.
    let go_mod_text =
        std::fs::read_to_string(kubo_sys.join("go.mod")).expect("failed to read kubo-sys/go.mod");
    let toolchain = go_mod_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("go ")
                .map(|ver| format!("go{}", ver.trim()))
        })
        .expect("no 'go' directive found in kubo-sys/go.mod");

    // Map Rust target to Go os/arch.
    let (goos, goarch) = parse_target(&target);

    let ffi_dir = kubo_sys.join("ffi");
    let archive = out_dir.join("libkubo_ffi.a");
    let header = out_dir.join("libkubo_ffi.h");

    // Only rebuild when Go sources change.
    println!("cargo:rerun-if-changed=kubo-sys/ffi/ffi.go");
    println!("cargo:rerun-if-changed=kubo-sys/ffi/go.mod");
    println!("cargo:rerun-if-changed=kubo-sys/ffi/go.sum");

    let status = Command::new(&go)
        .current_dir(&ffi_dir)
        .env("CGO_ENABLED", "1")
        .env("GOOS", goos)
        .env("GOARCH", goarch)
        .env("GOTOOLCHAIN", &toolchain)
        .args([
            "build",
            "-buildmode=c-archive",
            "-o",
            archive.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run `go build` for FFI");

    if !status.success() {
        panic!("`go build` for kubo-sys/ffi failed");
    }

    if !archive.exists() {
        panic!("FFI archive not created: {:?}", archive);
    }
    if !header.exists() {
        panic!("FFI header not created: {:?}", header);
    }

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static=kubo_ffi");

    // Platform-specific system libraries required by the Go runtime.
    let family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if family == "unix" {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
    }

    if os == "macos" {
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=resolv");
    }
}

fn parse_target(target: &str) -> (&str, &str) {
    let parts: Vec<&str> = target.split('-').collect();
    let arch = parts[0];
    let os = parts.get(2).copied().unwrap_or("unknown");

    let goarch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "i686" => "386",
        "arm" => "arm",
        "armv7" => "arm",
        "riscv64" => "riscv64",
        "wasm32" => panic!("wasm32 is not supported by CGO"),
        _ => arch,
    };

    (os, goarch)
}
