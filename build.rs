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
    let host_vendor = host_parts.get(1).copied().unwrap_or("unknown");
    let target_vendor = target_parts.get(1).copied().unwrap_or("unknown");

    let both_darwin = host_vendor == "apple" && target_vendor == "apple";
    let both_linux = host_os == "linux" && target_os == "linux";
    let both_windows = host_os == "windows" && target_os == "windows";

    if target != host && !both_darwin && !both_linux && !both_windows {
        panic!(
            "kubo-rs FFI build script does not support cross-compilation \
             for this target/host pair. TARGET ({}) != HOST ({}). \
             To cross-compile, set up a C cross-toolchain and \
             manually compile go/ffi/ with CGO_ENABLED=1.",
            target, host
        );
    }

    // Verify the submodules are present.
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

    // Pin GOTOOLCHAIN to the Go version declared in go/ffi/go.mod so that
    // the build is reproducible even when the host has a newer Go installed.
    let ffi_dir = PathBuf::from("go/ffi");
    let go_mod_text =
        std::fs::read_to_string(ffi_dir.join("go.mod")).expect("failed to read go/ffi/go.mod");
    let toolchain = go_mod_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("go ")
                .map(|ver| format!("go{}", ver.trim()))
        })
        .expect("no 'go' directive found in go/ffi/go.mod");

    // Map Rust target to Go os/arch.
    let (goos, goarch) = parse_target(&target);

    // Go's c-archive naming depends on the platform toolchain:
    //   Unix / MinGW: libkubo_ffi.a
    //   Windows MSVC: kubo_ffi.lib
    // The header is always named after the stem (without lib/.lib suffix).
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let (archive_name, header_name) = if os == "windows" && env == "msvc" {
        ("kubo_ffi.lib", "kubo_ffi.h")
    } else {
        ("libkubo_ffi.a", "kubo_ffi.h")
    };
    let archive = out_dir.join(archive_name);
    let header = out_dir.join(header_name);

    // Only rebuild when Go sources change.
    println!("cargo:rerun-if-changed=go/ffi/kubo.go");
    println!("cargo:rerun-if-changed=go/ffi/libp2p.go");
    println!("cargo:rerun-if-changed=go/ffi/nostr.go");
    println!("cargo:rerun-if-changed=go/ffi/git.go");
    println!("cargo:rerun-if-changed=go/ffi/error.go");
    println!("cargo:rerun-if-changed=go/ffi/main.go");
    println!("cargo:rerun-if-changed=go/ffi/go.mod");
    println!("cargo:rerun-if-changed=go/ffi/go.sum");

    let mut go_build = Command::new(&go);
    go_build
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
        ]);

    // When cross-compiling for iOS (or Catalyst) the Go toolchain needs an
    // Apple-specific C cross-compiler. Without this, `clang` defaults to the
    // host macOS target and the linker rejects the resulting object files.
    if let Some((cc, cxx)) = apple_compiler(&target) {
        go_build.env("CC", &cc).env("CXX", &cxx);

        // Align the Go C compiler's deployment target with the Rust linker so
        // that generated object files use symbols available on the target OS
        // version. Defaults to 14.0 (override with IPHONEOS_DEPLOYMENT_TARGET).
        let deployment_target = env::var("IPHONEOS_DEPLOYMENT_TARGET")
            .or_else(|_| env::var("MACOSX_DEPLOYMENT_TARGET"))
            .unwrap_or_else(|_| "14.0".to_string());

        let min_flag = if target.contains("ios-sim")
            || (target.contains("ios") && target.starts_with("x86_64"))
        {
            format!("-mios-simulator-version-min={}", deployment_target)
        } else if target.contains("ios") {
            format!("-miphoneos-version-min={}", deployment_target)
        } else if target.contains("darwin") {
            format!("-mmacosx-version-min={}", deployment_target)
        } else {
            String::new()
        };

        if !min_flag.is_empty() {
            go_build.env("CGO_CFLAGS", &min_flag);
            go_build.env("CGO_LDFLAGS", &min_flag);
        }
    }

    let status = go_build.status().expect("failed to run `go build` for FFI");

    if !status.success() {
        panic!("`go build` for go/ffi failed");
    }

    if !archive.exists() {
        panic!("FFI archive not created: {:?}", archive);
    }

    // Go's c-archive header naming varies by toolchain version:
    // some produce kubo_ffi.h, others libkubo_ffi.h. Accept either.
    let _header = if header.exists() {
        header
    } else {
        let alt = out_dir.join("libkubo_ffi.h");
        if alt.exists() {
            alt
        } else {
            panic!("FFI header not created: expected {:?} or {:?}", header, alt);
        }
    };

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

    if os == "macos" || os == "ios" {
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=resolv");
    }
}

fn parse_target(target: &str) -> (&str, &str) {
    let parts: Vec<&str> = target.split('-').collect();
    let arch = parts[0];

    // Apple iOS variants (ios, ios-sim, ios-macabi) all map to Go's "ios".
    let os = if parts.len() >= 3 && parts[1] == "apple" && parts[2].starts_with("ios") {
        "ios"
    } else {
        parts.get(2).copied().unwrap_or("unknown")
    };

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

/// Return the (CC, CXX) commands for Apple cross-compilation when the host
/// is macOS and the target is a non-macOS Apple platform (iOS, simulator,
/// or Mac Catalyst). Uses `xcrun` to select the correct SDK and target.
fn apple_compiler(target: &str) -> Option<(String, String)> {
    if std::env::consts::OS != "macos" {
        return None;
    }

    let parts: Vec<&str> = target.split('-').collect();
    if parts.len() < 3 || parts[1] != "apple" {
        return None;
    }

    let arch = parts[0];
    let os = parts[2];
    let variant = parts.get(3).copied();

    // (SDK, optional explicit -target flag)
    let (sdk, target_flag): (&str, &str) = match (arch, os, variant) {
        // iOS Device
        ("aarch64", "ios", None) => ("iphoneos", ""),
        // iOS Simulator (x86_64 iOS is always simulator)
        ("x86_64", "ios", None) => ("iphonesimulator", ""),
        ("aarch64", "ios", Some("sim")) => ("iphonesimulator", ""),
        // Mac Catalyst
        ("aarch64", "ios", Some("macabi")) => ("macosx", "-target arm64-apple-ios-macabi"),
        ("x86_64", "ios", Some("macabi")) => ("macosx", "-target x86_64-apple-ios-macabi"),
        // macOS – Go can use the host compiler for same-family builds
        (_, "darwin", _) => return None,
        _ => return None,
    };

    let cc = format!("xcrun -sdk {} clang {}", sdk, target_flag)
        .trim()
        .to_string();
    let cxx = format!("xcrun -sdk {} clang++ {}", sdk, target_flag)
        .trim()
        .to_string();

    Some((cc, cxx))
}
