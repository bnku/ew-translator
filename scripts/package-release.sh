#!/usr/bin/env bash

set -euo pipefail

project_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
release_dir="${project_root}/release"
raw_binary="${project_root}/target/release/ew-translator"
packed_binary="${release_dir}/ew-translator"

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
  echo "This packaging script currently supports Linux x86_64 only." >&2
  exit 1
fi

for command_name in cargo npm strip upx sha256sum; do
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    echo "Required command not found: ${command_name}" >&2
    exit 1
  fi
done

npm --prefix "${project_root}/gui" ci
npm --prefix "${project_root}/gui" run check
npm --prefix "${project_root}/gui" run build
cargo build \
  --manifest-path "${project_root}/Cargo.toml" \
  --release \
  --locked \
  --features custom-protocol

mkdir -p "${release_dir}"
install -m 755 "${raw_binary}" "${packed_binary}"
strip --strip-all "${packed_binary}"
upx --best --lzma "${packed_binary}"
upx -t "${packed_binary}"

(
  cd "${release_dir}"
  sha256sum ew-translator > ew-translator.sha256
)

stat --printf '%n: %s bytes\n' "${packed_binary}"
cat "${release_dir}/ew-translator.sha256"
