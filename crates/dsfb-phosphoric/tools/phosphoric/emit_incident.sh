#!/usr/bin/env bash
# emit_incident.sh — driver for phosphoric_incident host tool.
#
# Packs a `.pfi` incident container from the four required inputs:
#   --manifest=<path>     task.manifest.toml
#   --image-section=<p>   the boot image's .pmanifest section bytes (300 B)
#   --residuals=<path>    runtime residual ring dump
#   --gray=<path>         optional dsfb-gray report (omit to write zero-length)
#   --out=<path>          output .pfi file
#
# Closed format taxonomy: only .pfi and .pfa exist. Do not extend this
# script to emit any other format.

set -euo pipefail

if [ ! -x build/host-tools/phosphoric_incident ]; then
    echo "[scaffold] tools/phosphoric-host/phosphoric_incident.phos not yet built"
    exit 2
fi

manifest=""
image_section=""
residuals=""
gray="/dev/null"
out=""

for arg in "$@"; do
    case "$arg" in
        --manifest=*)      manifest="${arg#--manifest=}"           ;;
        --image-section=*) image_section="${arg#--image-section=}" ;;
        --residuals=*)     residuals="${arg#--residuals=}"         ;;
        --gray=*)          gray="${arg#--gray=}"                   ;;
        --out=*)           out="${arg#--out=}"                     ;;
        *) echo "unknown arg: $arg" >&2; exit 1                    ;;
    esac
done

for required in manifest image_section residuals out; do
    if [ -z "${!required}" ]; then
        echo "missing --$required" >&2
        exit 1
    fi
done

build/host-tools/phosphoric_incident \
    --manifest="$manifest" \
    --image-section="$image_section" \
    --residuals="$residuals" \
    --gray="$gray" \
    --out="$out"
