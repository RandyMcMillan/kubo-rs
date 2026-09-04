#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== Cross-language alignment test suite ==="
echo

echo "=== 1. Rust library tests ==="
cargo test --verbose

echo
echo "=== 2. Rust CLI tests ==="
cargo test --test cli --verbose

echo
echo "=== 3. Build FFI archive ==="
cd kubo-sys/ffi
go build -buildmode=c-archive -o ./tmp/libkubo_ffi.a ./ffi.go
cd ../..

echo
echo "=== 4. C FFI tests ==="
if [ "$(uname)" = "Darwin" ]; then
  cc -o kubo-sys/ffi/cmd/testffi/testffi \
    kubo-sys/ffi/cmd/testffi/main.c \
    -Ikubo-sys/ffi/tmp \
    kubo-sys/ffi/tmp/libkubo_ffi.a \
    -lpthread -ldl -framework Security -framework CoreFoundation -lresolv
else
  cc -o kubo-sys/ffi/cmd/testffi/testffi \
    kubo-sys/ffi/cmd/testffi/main.c \
    -Ikubo-sys/ffi/tmp \
    kubo-sys/ffi/tmp/libkubo_ffi.a \
    -lpthread -ldl
fi
./kubo-sys/ffi/cmd/testffi/testffi

echo
echo "=== 5. Rust raw-FFI tests ==="
if [ "$(uname)" = "Darwin" ]; then
  rustc kubo-sys/ffi/cmd/testrust/main.rs \
    -L kubo-sys/ffi/tmp -lkubo_ffi \
    -o kubo-sys/ffi/cmd/testrust/testrust \
    -C link-arg="-framework" -C link-arg="Security" \
    -C link-arg="-framework" -C link-arg="CoreFoundation" \
    -lresolv -lpthread -ldl
else
  rustc kubo-sys/ffi/cmd/testrust/main.rs \
    -L kubo-sys/ffi/tmp -lkubo_ffi \
    -o kubo-sys/ffi/cmd/testrust/testrust \
    -lpthread -ldl
fi
./kubo-sys/ffi/cmd/testrust/testrust

echo
echo "=== ALL CROSS-TESTS PASSED ==="
