#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
target_dir="${PHOSPHORIC_TARGET_DIR:-/tmp/phosphoric-target}"

cd "$repo_root"

printf '%s\n' "== phosphoric: repo hygiene =="
tools/verify/check_repo_hygiene.sh

printf '%s\n' "== phosphoric: archive inertness =="
tools/verify/check_archive_inert.sh

printf '%s\n' "== phosphoric: docs checks =="
tools/verify/check_docs.sh

printf '%s\n' "== phosphoric: no non-Phosphoric golden boot link gate =="
tools/verify/check_boot_phosphoric_only.sh

printf '%s\n' "== phosphoric: direct PE negative tests =="
tools/verify/check_direct_pe_negative_tests.sh

printf '%s\n' "== phosphoric: uefi demo smoke test =="
tools/qemu-run/run_uefi_demo.sh

printf '%s\n' "== phosphoric: verification complete =="
