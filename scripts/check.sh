#!/usr/bin/env bash
# Runs the same checks CI does. Run this before every commit, not after
# a red CI run tells you to.
set -euo pipefail

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings

echo "==> cargo test"
cargo test

echo "All checks passed."
