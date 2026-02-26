#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOML_TEST_DIR="${TOML_TEST_DIR:-$ROOT_DIR/.cache/toml-test}"
TOML_TEST_REPO="${TOML_TEST_REPO:-https://github.com/toml-lang/toml-test}"
TOML_VERSION="${TOML_VERSION:-1.0}"

if ! command -v go >/dev/null 2>&1; then
	echo "go コマンドが必要です" >&2
	exit 1
fi

if [ ! -d "$TOML_TEST_DIR/.git" ]; then
	mkdir -p "$(dirname "$TOML_TEST_DIR")"
	git clone --depth 1 "$TOML_TEST_REPO" "$TOML_TEST_DIR"
elif [ "${TOML_TEST_UPDATE:-0}" = "1" ]; then
	git -C "$TOML_TEST_DIR" pull --ff-only
fi

cargo build --quiet --manifest-path "$ROOT_DIR/tools/toml-test-adapter/Cargo.toml"

DECODER="$ROOT_DIR/tools/toml-test-adapter/target/debug/toml-test-decoder"
ENCODER="$ROOT_DIR/tools/toml-test-adapter/target/debug/toml-test-encoder"

(
	cd "$TOML_TEST_DIR"
	go run ./cmd/toml-test test \
		-toml "$TOML_VERSION" \
		-decoder "$DECODER" \
		-encoder "$ENCODER" \
		"$@"
)
