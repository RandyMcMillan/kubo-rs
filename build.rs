use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let target = env::var("TARGET").expect("TARGET not set");
    let host = env::var("HOST").expect("HOST not set");

    // Cross-compilation with CGO requires a matching C cross-toolchain.
    // Allow macOS universal builds (arm64 ↔ x86_64) because the Xcode
    // toolchain ships both SDKs. For everything else, bail out early
    // with a clear message rather than letting `go build` fail cryptically.
    let host_parts: Vec<&str> = host.split('-').collect();
    let target_parts: Vec<&str> = target.split('-').collect();
    let host_os = host_parts.get(2).copied().unwrap_or("unknown");
    let target_os = target_parts.get(2).copied().unwrap_or("unknown");

    let both_darwin = host_os == "apple" && target_os == "apple";

    if target != host && !both_darwin {
        panic!(
            "kubo-rs FFI build script does not support cross-compilation \
             for this target/host pair. TARGET ({}) != HOST ({}). \
             To cross-compile, set up a C cross-toolchain and \
             manually compile go/kubo-sys/ffi/ with CGO_ENABLED=1.",
            target, host
        );
    }

    // Verify the submodule is present.
    let kubo_sys = PathBuf::from("go/kubo-sys");
    if !kubo_sys.join("go.mod").exists() {
        panic!(
            "go/kubo-sys/go.mod not found. \
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

    // Pin GOTOOLCHAIN to the Go version declared in go/kubo-sys/go.mod so that
    // the build is reproducible even when the host has a newer Go installed.
    let go_mod_text = std::fs::read_to_string(kubo_sys.join("go.mod"))
        .expect("failed to read go/kubo-sys/go.mod");
    let toolchain = go_mod_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("go ")
                .map(|ver| format!("go{}", ver.trim()))
        })
        .expect("no 'go' directive found in go/kubo-sys/go.mod");

    // Map Rust target to Go os/arch.
    let (goos, goarch) = parse_target(&target);

    let ffi_dir = kubo_sys.join("ffi");

    // Go's c-archive naming depends on the platform toolchain:
    //   Unix / MinGW: libkubo_ffi.a
    //   Windows MSVC: kubo_ffi.lib
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let (archive_name, header_name) = if os == "windows" && env == "msvc" {
        ("kubo_ffi.lib", "kubo_ffi.h")
    } else {
        ("libkubo_ffi.a", "libkubo_ffi.h")
    };
    let archive = out_dir.join(archive_name);
    let header = out_dir.join(header_name);

    // Only rebuild when Go sources change.
    println!("cargo:rerun-if-changed=go/kubo-sys/ffi/ffi.go");
    println!("cargo:rerun-if-changed=go/kubo-sys/ffi/go.mod");
    println!("cargo:rerun-if-changed=go/kubo-sys/ffi/go.sum");

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
        panic!("`go build` for go/kubo-sys/ffi failed");
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
