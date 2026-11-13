#!/usr/bin/env bash
# Install a released sj (Linux x64) into ~/.local/bin, verifying its SHA256 checksum.
# Usage: install-sj.sh <version>     (needs GH_TOKEN for gh)
set -euo pipefail
version="${1:?usage: install-sj.sh <version>}"
target=x86_64-unknown-linux-gnu
dir=$(mktemp -d)
gh release download "v$version" --repo nabour-labs/specjutsu --dir "$dir" \
  --pattern "sj-$target.tar.gz" --pattern SHA256SUMS
(cd "$dir" && grep " sj-$target.tar.gz\$" SHA256SUMS | sha256sum -c -)
tar -xzf "$dir/sj-$target.tar.gz" -C "$dir"
mkdir -p "$HOME/.local/bin"
install "$dir/sj-$target/sj" "$HOME/.local/bin/sj"
echo "$HOME/.local/bin" >> "${GITHUB_PATH:-/dev/null}"
"$HOME/.local/bin/sj" --version
