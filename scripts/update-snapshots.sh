#!/usr/bin/env bash
# update-snapshots.sh — regenerate committed GPU snapshot goldens.
#
# Usage:
#   scripts/update-snapshots.sh                          # all crates, detect backend
#   scripts/update-snapshots.sh --backend vulkan         # force vulkan (all crates)
#   scripts/update-snapshots.sh --crate widgets          # widgets only
#   scripts/update-snapshots.sh --crate style            # quartzite-style only
#   scripts/update-snapshots.sh --crate all              # both crates (default)
#   scripts/update-snapshots.sh --crate style --backend metal
#
# Sets `QUARTZITE_REGENERATE_SNAPSHOTS=1` and runs the snapshot integration
# test(s). The snapshot helper (`tests/support/mod.rs`) writes the rendered
# images into `<crate>/tests/snapshots/<backend>/` instead of comparing.
#
# Bootstrapping `shared/` goldens is a manual step: after regen, move the
# per-backend PNGs to `shared/`:
#   mv quartzite-widgets/tests/snapshots/<backend>/*.png \
#      quartzite-widgets/tests/snapshots/shared/
#   mv quartzite-style/tests/snapshots/<backend>/*.png \
#      quartzite-style/tests/snapshots/shared/
#
# Does NOT run the `xvfb_smoke` integration test: that test asserts only on
# clean startup + clean exit (no pixels), so there is nothing to regenerate.
#
# Notes for *intentional* visual changes:
#   1. Run this script for each backend you have access to locally.
#   2. Inspect the changed PNGs under tests/snapshots/<backend>/ — diff
#      tools struggle with binary PNGs, so a visual review is required.
#   3. Commit the regenerated goldens together with the code change that
#      caused the diff. Reviewers see the new pixels in the PR.
#
# POSIX bash, no GNU-only constructs (matches gen-roadmap.sh precedent).

set -euo pipefail

usage() {
    sed -n '/^# Usage:/,/^$/p' "$0" | sed 's/^# \{0,1\}//'
    exit "${1:-0}"
}

backend=""
crate="all"

while [ $# -gt 0 ]; do
    case "$1" in
    --backend)
        if [ $# -lt 2 ]; then
            echo "error: --backend requires a value (vulkan|dx12|metal)" >&2
            exit 2
        fi
        backend="$2"
        shift 2
        ;;
    --backend=*)
        backend="${1#--backend=}"
        shift
        ;;
    --crate)
        if [ $# -lt 2 ]; then
            echo "error: --crate requires a value (all|widgets|style)" >&2
            exit 2
        fi
        crate="$2"
        shift 2
        ;;
    --crate=*)
        crate="${1#--crate=}"
        shift
        ;;
    -h | --help)
        usage 0
        ;;
    *)
        echo "error: unknown argument: $1" >&2
        usage 2
        ;;
    esac
done

case "$crate" in
all | widgets | style) ;;
*)
    echo "error: --crate must be one of all|widgets|style (got: $crate)" >&2
    exit 2
    ;;
esac

# Auto-detect backend when --backend was not supplied. Honours
# pre-existing `WGPU_BACKEND` first (caller already chose) and falls
# back to `uname` (Linux → vulkan, Darwin → metal, anything else →
# vulkan as the most common cross-platform fallback).
if [ -z "$backend" ]; then
    if [ -n "${WGPU_BACKEND:-}" ]; then
        backend="$WGPU_BACKEND"
    else
        case "$(uname -s)" in
        Linux) backend="vulkan" ;;
        Darwin) backend="metal" ;;
        MINGW* | MSYS* | CYGWIN*) backend="dx12" ;;
        *) backend="vulkan" ;;
        esac
    fi
fi

case "$backend" in
vulkan | dx12 | metal) ;;
*)
    echo "error: --backend must be one of vulkan|dx12|metal (got: $backend)" >&2
    exit 2
    ;;
esac

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

echo "regenerating snapshots: crate=$crate backend=$backend"

if [ "$crate" = "all" ] || [ "$crate" = "widgets" ]; then
    WGPU_BACKEND="$backend" \
        QUARTZITE_REGENERATE_SNAPSHOTS=1 \
        cargo test -p quartzite-widgets --test snapshots
fi

if [ "$crate" = "all" ] || [ "$crate" = "style" ]; then
    WGPU_BACKEND="$backend" \
        QUARTZITE_REGENERATE_SNAPSHOTS=1 \
        cargo test -p quartzite-style --test snapshots
fi

echo
echo "done. regenerated goldens in:"
if [ "$crate" = "all" ] || [ "$crate" = "widgets" ]; then
    echo "  quartzite-widgets/tests/snapshots/$backend"
fi
if [ "$crate" = "all" ] || [ "$crate" = "style" ]; then
    echo "  quartzite-style/tests/snapshots/$backend"
fi
echo "review the diff and commit the *.png files together with the change that caused them."
echo "to bootstrap shared/ goldens: mv <backend>/*.png shared/"
