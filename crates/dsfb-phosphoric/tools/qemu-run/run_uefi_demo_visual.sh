#!/usr/bin/env bash
# Visual variant of run_uefi_demo.sh: builds the image, runs QEMU with a
# monitor socket, schedules a `screendump` after the demo's render path
# completes, then converts the PPM framebuffer dump to PNG so a human can see
# what the demo drew.
#
# Output:
#   - build/uefi-demo/qemu-debug.log      (same as run_uefi_demo.sh)
#   - build/uefi-demo/screenshot.ppm      (raw QEMU framebuffer dump)
#   - build/uefi-demo/screenshot.png      (PNG conversion via ImageMagick)
#   - build/uefi-demo/demo-summary.txt    (human-readable run summary)
#
# This script does not replace run_uefi_demo.sh in the verify pipeline;
# `make verify` still uses the headless variant. Use `make demo-visual` to
# invoke this one.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_script="$repo_root/tools/image-builder/build_uefi_demo.sh"
build_dir="$repo_root/build/uefi-demo"
esp_root="$build_dir/esp"
esp_image="$build_dir/esp.img"
ovmf_vars="$build_dir/OVMF_VARS.4m.fd"
debug_log="$build_dir/qemu-debug.log"
link_manifest="$build_dir/linked-artifact.txt"
screenshot_ppm="$build_dir/screenshot.ppm"
screenshot_png="$build_dir/screenshot.png"
summary_file="$build_dir/demo-summary.txt"
monitor_sock="$build_dir/qemu-monitor.sock"

find_ovmf_file() {
  local configured="$1"
  shift

  if [ -n "$configured" ] && [ -f "$configured" ]; then
    printf '%s\n' "$configured"
    return 0
  fi

  local candidate
  for candidate in "$@"; do
    if [ -f "$candidate" ]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  return 1
}

if ! ovmf_code="$(find_ovmf_file "${PHOSPHORIC_OVMF_CODE:-}" \
  /usr/share/edk2/x64/OVMF_CODE.4m.fd \
  /usr/share/OVMF/OVMF_CODE_4M.fd \
  /usr/share/OVMF/OVMF_CODE.fd)"; then
  echo "failed to locate OVMF code image; set PHOSPHORIC_OVMF_CODE" >&2
  exit 1
fi

if ! ovmf_vars_src="$(find_ovmf_file "${PHOSPHORIC_OVMF_VARS:-}" \
  /usr/share/edk2/x64/OVMF_VARS.4m.fd \
  /usr/share/OVMF/OVMF_VARS_4M.fd \
  /usr/share/OVMF/OVMF_VARS.fd)"; then
  echo "failed to locate OVMF vars image; set PHOSPHORIC_OVMF_VARS" >&2
  exit 1
fi

# Build the image (delegates to existing build script).
"$build_script" >/dev/null

# Verify provenance manifest fields (subset of the headless verify, kept here
# so the visual run is self-contained).
grep -q '^archive_executed=false$' "$link_manifest"
grep -q '^clang_used=false$' "$link_manifest"
grep -q '^lld_used=false$' "$link_manifest"
grep -q '^external_linker_used=false$' "$link_manifest"
grep -q '^non_phosphoric_runtime_objects=none$' "$link_manifest"
grep -q '^c_objects=none$' "$link_manifest"

mkdir -p "$build_dir"
cp "$ovmf_vars_src" "$ovmf_vars"
rm -f "$debug_log" "$esp_image" "$screenshot_ppm" "$screenshot_png" \
      "$summary_file" "$monitor_sock"
truncate -s 64M "$esp_image"
mkfs.vfat "$esp_image" >/dev/null
mmd -i "$esp_image" ::/EFI ::/EFI/BOOT
mcopy -i "$esp_image" "$esp_root/EFI/BOOT/BOOTX64.EFI" ::/EFI/BOOT/BOOTX64.EFI

# Take screenshots at multiple checkpoints. The visual runner deliberately
# does NOT include the isa-debug-exit device (unlike the headless runner),
# so QEMU runs for the full timeout instead of exiting immediately when the
# demo finishes. This keeps the framebuffer alive long enough to capture.
mkdir -p "$build_dir/screenshots"
(
  # Wait for the monitor socket to be ready.
  for _ in $(seq 1 50); do
    [ -S "$monitor_sock" ] && break
    sleep 0.1
  done

  # Race against the demo's exit. The painted frame exists for only a few
  # milliseconds between `redraw complete` and `demo complete`. Strategy:
  # poll the debug log every 20 ms; the moment `redraw complete` appears,
  # screendump as fast as we can. Take additional frames at fixed delays
  # so we also capture firmware screens for context.
  saw_redraw=false
  redraw_dump_count=0
  for tick in $(seq 1 600); do
    sleep 0.02
    [ -S "$monitor_sock" ] || break
    if ! $saw_redraw && [ -f "$debug_log" ] && grep -qF 'phosphoric: redraw complete' "$debug_log"; then
      saw_redraw=true
    fi
    if $saw_redraw && [ "$redraw_dump_count" -lt 5 ]; then
      idx=$(printf '%02d' "$redraw_dump_count")
      printf 'screendump %s\n' "$build_dir/screenshots/redraw-$idx.ppm" | \
        socat - UNIX-CONNECT:"$monitor_sock" >/dev/null 2>&1 || break
      redraw_dump_count=$((redraw_dump_count + 1))
    fi
  done

  # Take fixed-delay reference frames too (firmware screens for context).
  for delay in 1 3 6 10; do
    [ -S "$monitor_sock" ] || break
    printf 'screendump %s\n' "$build_dir/screenshots/frame-${delay}s.ppm" | \
      socat - UNIX-CONNECT:"$monitor_sock" >/dev/null 2>&1 || true
  done

  # Final screendump labeled as canonical.
  if [ -S "$monitor_sock" ]; then
    printf 'screendump %s\n' "$screenshot_ppm" | \
      socat - UNIX-CONNECT:"$monitor_sock" >/dev/null 2>&1 || true
  fi
) &
screendump_pid=$!

qemu_status=0
# NOTE: -device isa-debug-exit is intentionally absent here so the demo's
# "demo complete" log line does not end the QEMU process. The 15s timeout
# bounds the run.
if ! TMPDIR=/tmp timeout 15s qemu-system-x86_64 \
  -machine q35,accel=tcg \
  -cpu max \
  -m 256M \
  -display none \
  -monitor "unix:$monitor_sock,server,nowait" \
  -serial none \
  -debugcon "file:$debug_log" \
  -global isa-debugcon.iobase=0x402 \
  -drive if=pflash,format=raw,readonly=on,file="$ovmf_code" \
  -drive if=pflash,format=raw,file="$ovmf_vars" \
  -drive format=raw,file="$esp_image" \
  -vga std \
  -boot menu=off; then
  qemu_status=$?
fi

wait "$screendump_pid" 2>/dev/null || true

# Convert every captured PPM to PNG via ImageMagick.
for ppm in "$screenshot_ppm" "$build_dir"/screenshots/*.ppm; do
  [ -f "$ppm" ] || continue
  png="${ppm%.ppm}.png"
  convert "$ppm" "$png" 2>/dev/null || true
done

# Build the human-readable summary.
{
  printf 'Phosphoric demo — visual run summary\n'
  printf '====================================\n\n'
  printf 'Run timestamp:       %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'Boot artifact:       %s\n' "$esp_root/EFI/BOOT/BOOTX64.EFI"
  if [ -f "$esp_root/EFI/BOOT/BOOTX64.EFI" ]; then
    printf 'Boot artifact size:  %s bytes\n' "$(stat -c %s "$esp_root/EFI/BOOT/BOOTX64.EFI")"
    printf 'Boot artifact SHA-256: %s\n' "$(sha256sum "$esp_root/EFI/BOOT/BOOTX64.EFI" | awk '{print $1}')"
  fi
  printf 'QEMU exit code:      %s\n' "$qemu_status"
  printf '\n'
  printf 'Provenance manifest:\n'
  sed 's/^/  /' "$link_manifest"
  printf '\n'
  printf 'Debug-port log (every line emitted by the demo runtime):\n'
  if [ -f "$debug_log" ]; then
    sed 's/^/  /' "$debug_log"
  else
    printf '  (no debug log produced)\n'
  fi
  printf '\n'
  printf 'Required QEMU markers:\n'
  for marker in \
    'phosphoric: entering generated boot-asm demo' \
    'phosphoric: generated boot-asm demo runtime active' \
    'phosphoric: event routed' \
    'phosphoric: redraw complete' \
    'phosphoric: demo complete'
  do
    if grep -qF "$marker" "$debug_log" 2>/dev/null; then
      printf '  [OK]   %s\n' "$marker"
    else
      printf '  [MISS] %s\n' "$marker"
    fi
  done
  printf '\n'
  printf 'Framebuffer screenshots:\n'
  if [ -f "$screenshot_ppm" ]; then
    printf '  Final PPM: %s (%s bytes)\n' "$screenshot_ppm" "$(stat -c %s "$screenshot_ppm")"
  else
    printf '  Final PPM: (not captured)\n'
  fi
  if [ -f "$screenshot_png" ]; then
    printf '  Final PNG: %s (%s bytes)\n' "$screenshot_png" "$(stat -c %s "$screenshot_png")"
  else
    printf '  Final PNG: (not converted)\n'
  fi
  if [ -d "$build_dir/screenshots" ]; then
    printf '  Per-frame captures:\n'
    for f in "$build_dir"/screenshots/*.png; do
      [ -f "$f" ] || continue
      printf '    %s (%s bytes)\n' "$f" "$(stat -c %s "$f")"
    done
  fi
  printf '\n'
  printf 'All artifacts under: %s\n' "$build_dir"
  printf '  - qemu-debug.log         debug-port output captured during the run\n'
  printf '  - demo-summary.txt       this summary\n'
  printf '  - linked-artifact.txt    provenance manifest\n'
  printf '  - screenshot.ppm/.png    canonical framebuffer state (final monitor screendump)\n'
  printf '  - screenshots/redraw-NN  framebuffer captured immediately after `redraw complete` log\n'
  printf '  - screenshots/frame-Ns   framebuffer at fixed wall-clock checkpoints (firmware context)\n'
  printf '\n'
  printf 'NOTE on visual content: the current boot-asm-v1 demo runtime is a\n'
  printf '*logical* render path. Its `phosphoric_demo_render` function builds a\n'
  printf 'bounded render-command list and emits the `redraw complete` debug-log\n'
  printf 'marker, proving the entire generated boot-asm pipeline executed. It does\n'
  printf 'NOT yet emit framebuffer MMIO writes, so the screendumps show OVMF\n'
  printf 'firmware screens — not painted Phosphoric pixels.\n'
  printf '\n'
  printf 'Pixel-visible rendering lands when the Ember UEFI firmware bridge\n'
  printf 'gains GOP framebuffer access and the boot-profile codegen lowers\n'
  printf 'render commands to MMIO writes.\n'
} > "$summary_file"

# Print the summary to stdout so the run is self-explanatory.
cat "$summary_file"

# Verify required markers present (non-fatal — visual run prioritizes
# producing artifacts over strict gating; the headless run_uefi_demo.sh
# remains the verification gate).
all_present=true
for marker in \
  'phosphoric: entering generated boot-asm demo' \
  'phosphoric: generated boot-asm demo runtime active' \
  'phosphoric: event routed' \
  'phosphoric: redraw complete' \
  'phosphoric: demo complete'
do
  if ! grep -qF "$marker" "$debug_log" 2>/dev/null; then
    all_present=false
  fi
done

if ! $all_present; then
  printf 'one or more required QEMU markers were missing; visual artifacts still produced\n' >&2
fi

if [ "$qemu_status" -ne 0 ] && [ "$qemu_status" -ne 1 ]; then
  exit "$qemu_status"
fi
