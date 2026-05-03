#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
release_root="$repo_root/build/release"
staging_root="$release_root/staging"
build_script="$repo_root/tools/image-builder/build_uefi_demo.sh"

cd "$repo_root"

efi_path="$("$build_script")"

if [ ! -f "$efi_path" ]; then
  printf 'missing built EFI artifact: %s\n' "$efi_path" >&2
  exit 1
fi

version="$(git rev-parse --short HEAD 2>/dev/null || printf '%s' uncommitted)"
if ! git diff --quiet --ignore-submodules -- 2>/dev/null || ! git diff --cached --quiet --ignore-submodules -- 2>/dev/null; then
  version="${version}-dirty"
fi

package_name="phosphoric-demo-${version}"
package_dir="$staging_root/$package_name"
archive_path="$release_root/${package_name}.tar.gz"
checksum_path="$release_root/SHA256SUMS"

rm -rf "$package_dir" "$archive_path"
mkdir -p "$package_dir/docs" "$package_dir/ember/docs" "$package_dir/efi"

cp README.md STATUS.md CLAIMS.md LANGUAGE_NON_GOALS.md "$package_dir/"
cp \
  docs/repro_build.md \
  docs/DETERMINISTIC_BUILD_VERIFICATION.md \
  docs/ATTACK_SURFACE_REVIEW.md \
  docs/FAKE_CLAIM_PREVENTION.md \
  docs/SECURITY_ASSUMPTIONS.md \
  docs/LEGENDARY_REVIEW.md \
  docs/invariant_trace.md \
  docs/quality_bar.md \
  "$package_dir/docs/"
cp ember/docs/EMBER_TRUST_AUDIT.md ember/docs/EMBER_MINIMALITY.md "$package_dir/ember/docs/"
cp "$efi_path" "$package_dir/efi/BOOTX64.EFI"

tar -C "$staging_root" -czf "$archive_path" "$package_name"

(
  cd "$release_root"
  sha256sum "${package_name}.tar.gz" > "$checksum_path"
  sha256sum "staging/${package_name}/efi/BOOTX64.EFI" >> "$checksum_path"
)

printf '%s\n' "$archive_path"
