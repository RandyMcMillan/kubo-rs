#!/usr/bin/env python3
"""Cross-platform test runner for kubo-rs."""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent


def run(cmd: list[str], env: dict | None = None) -> None:
    print(f"=== {' '.join(cmd)} ===")
    merged = env.copy() if env else None
    subprocess.run(cmd, cwd=ROOT, check=True, env=merged)


def main() -> int:
    try:
        run(["cargo", "fmt", "--", "--check"])
        run(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"])
        run(["cargo", "build", "--bin", "kubo-rs"])
        run(["cargo", "build", "--examples"])
        run(["cargo", "test"])
        env = {"RUSTDOCFLAGS": "-D warnings"}
        run(["cargo", "doc", "--no-deps", "--document-private-items"], env=env)
        print("=== All checks passed ===")
        print()
        print("For cross-language FFI alignment tests, run:")
        print("  make test-ffi")
        print("  ./scripts/cross-test.sh")
        return 0
    except subprocess.CalledProcessError as e:
        print(f"FAILED: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
