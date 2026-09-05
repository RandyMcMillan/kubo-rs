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
cd go/ffi
go build -buildmode=c-archive -o ./tmp/libkubo_ffi.a .
cd ../..

echo
echo "=== 4. C FFI tests ==="
if [ "$(uname)" = "Darwin" ]; then
  cc -o go/ffi/cmd/testffi/testffi \
    go/ffi/cmd/testffi/main.c \
    -Igo/ffi/tmp \
    go/ffi/tmp/libkubo_ffi.a \
    -lpthread -ldl -framework Security -framework CoreFoundation -lresolv
else
  cc -o go/ffi/cmd/testffi/testffi \
    go/ffi/cmd/testffi/main.c \
    -Igo/ffi/tmp \
    go/ffi/tmp/libkubo_ffi.a \
    -lpthread -ldl
fi
./go/ffi/cmd/testffi/testffi

echo
echo "=== 5. Rust raw-FFI tests ==="
if [ "$(uname)" = "Darwin" ]; then
  rustc go/ffi/cmd/testrust/main.rs \
    -L go/ffi/tmp -lkubo_ffi \
    -o go/ffi/cmd/testrust/testrust \
    -C link-arg="-framework" -C link-arg="Security" \
    -C link-arg="-framework" -C link-arg="CoreFoundation" \
    -lresolv -lpthread -ldl
else
  rustc go/ffi/cmd/testrust/main.rs \
    -L go/ffi/tmp -lkubo_ffi \
    -o go/ffi/cmd/testrust/testrust \
    -lpthread -ldl
fi
./go/ffi/cmd/testrust/testrust

echo
echo "=== ALL CROSS-TESTS PASSED ==="
