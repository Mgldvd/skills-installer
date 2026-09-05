#!/usr/bin/env bash
set -Eeuo pipefail

# Args: $1 = TEST_DIR, $2 = CLI_BIN (both relative to the project root).
test_dir="$(pwd)/$1"
cli_bin="$(pwd)/$2"

rm -rf "$test_dir"
mkdir -p "$test_dir"
cd "$test_dir"

ids=$("$cli_bin" list --json | jq -r '.[] | select(.enabled) | .id' | shuf -n 5)
echo "Installing: $ids"
# shellcheck disable=SC2086 # intentional word-splitting: $ids is several skill ids, one CLI arg each.
"$cli_bin" install $ids --yes
echo
echo "Installed in $test_dir/:"
"$cli_bin" list --installed
