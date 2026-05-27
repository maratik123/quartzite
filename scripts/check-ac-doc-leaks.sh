#!/usr/bin/env bash
# scripts/check-ac-doc-leaks.sh
#
# Regression gate for the "no AC# acceptance-criteria tokens in published-rustdoc
# doc-comments" rule introduced by issue #559 (PR feat/2026-05-27-ac-doc-leak-guard,
# spec: ai-docs/plans/2026-05-27-ac-doc-leak-guard.spec.md).
#
# Walks the workspace's Rust source tree (excluding tests/, benches/,
# quartzite-test-helpers/src/, and target/) and matches lines that begin with a
# doc-comment marker (///, //!, or #[doc = "..."]) and contain a token of shape
# \bAC[0-9]+[a-z]*\b. The "#[cfg(test)]-region filter" drops hits that are
# inside a #[cfg(test)] sibling-file or an inline #[cfg(test)] mod ... { ... }
# block, because those lines are not part of the published rustdoc surface.
#
# Source-of-truth donor file. scripts/check-rustdoc-internal-refs.sh is the
# structural precedent for this gate; the `sibling_re` regex, the
# `is_cfg_test_enclosed` awk helper, and the `known_sibling_site` fail-loud
# smoke-check block are copied verbatim from that script. Any bug fix to one
# script's #[cfg(test)]-region filter MUST land in the other in the same PR
# (drift risk tracked in the spec's design doc § Risks).
#
# v1 scope. No `--list-skipped` debugging flag (the precedent ships one; this
# script can grow one in a follow-up if developers want a debugging mode).
#
# Exit codes:
#   0  no retained published-surface hits
#   1  at least one retained published-surface hit (gate FAILS)
#   2  internal usage error (missing tool, missing known sibling site)

set -euo pipefail

# Resolve repo root so the script works from any cwd.
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)
cd -- "$REPO_ROOT"

if ! command -v rg >/dev/null 2>&1; then
    echo "error: ripgrep (rg) not found on PATH" >&2
    exit 2
fi

# Single combined regex covering all three published-doc-comment shapes
# (///, //!, #[doc = "..."]) and the AC<digits>[lowercase]* token alphabet.
PATTERN='^\s*(///|//!|#\[doc\s*=).*\bAC[0-9]+[a-z]*\b'

# Raw hit list, scoped to published-surface paths (matches the precedent's
# exclusion globs: out-of-process tests/, benches/, quartzite-test-helpers
# (which sets [lib] doc = false), and the target/ build cache).
hits_raw=$(rg --type rust -n --no-heading \
    -e "$PATTERN" \
    -g '!**/tests/**' -g '!**/benches/**' \
    -g '!quartzite-test-helpers/src/**' -g '!target/**' \
    || true)

if [[ -z "$hits_raw" ]]; then
    exit 0
fi
