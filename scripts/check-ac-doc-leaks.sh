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

# Build the set of sibling-attached-test-file basenames once (shape 2 detection).
# Looks for `#[cfg(test)] [other attrs] #[path = "NAME.rs"] mod IDENT;` across
# the workspace. Per the design's shape-2 correction note, the POSIX [:space:]
# class already matches newlines, so we use [[:space:]]* (not [[:space:]\n]*)
# in the multiline form below — the literal `\n` inside a POSIX bracket class
# would be the two characters `\` and `n`, not a newline.
#
# We deliberately use a permissive between-attributes pattern so the rg
# --multiline match still works if a future contributor interposes other
# attributes (e.g. #[cfg(test)] #[allow(unused)] #[path = "..."] mod tests;).
sibling_re='#\[cfg\(test\)\][[:space:]]*(#\[[^]]*\][[:space:]]*)*#\[path[[:space:]]*=[[:space:]]*"([^"]+\.rs)"\][[:space:]]*(#\[[^]]*\][[:space:]]*)*mod[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*;'

# Verify the multiline form actually captures the known site as a smoke check —
# if rg --multiline returns empty against default_style/mod.rs (where we know
# the shape exists in-tree at lines 392-394), error out loudly so a future
# regex regression is caught at script-startup time, not silently masked as
# "no false-positives".
#
# Fail-loud shape (round-2 design directive vs precedent round-1 silent-skip):
# the missing-site branch MUST `exit 2`, NOT a silent `if [[ -f ... ]]` skip.
# The precedent's twin scripts/check-rustdoc-internal-refs.sh is being patched
# to the same shape in subtask 4 of this PR.
known_sibling_site=quartzite-style/src/default_style/mod.rs
if [[ ! -f $known_sibling_site ]]; then
    echo "error: known sibling-attached site $known_sibling_site not found" >&2
    echo "       update known_sibling_site in scripts/check-ac-doc-leaks.sh" >&2
    exit 2
fi
if ! rg --type rust --multiline -o "$sibling_re" "$known_sibling_site" >/dev/null 2>&1; then
    echo "error: shape-2 multiline regex no longer matches the known sibling-attached site $known_sibling_site:392-394" >&2
    echo "       update sibling_re in scripts/check-ac-doc-leaks.sh" >&2
    exit 2
fi
sibling_attached_files=""
# Capture every "path = NAME.rs" basename from all matches across the
# workspace. rg's --replace gives us just the capture group; we then collect
# unique basenames.
sibling_attached_files=$(rg --type rust --multiline --no-filename -o --replace '$2' "$sibling_re" \
    -g '!**/tests/**' -g '!**/benches/**' -g '!target/**' 2>/dev/null \
    | awk -F/ '{print $NF}' | sort -u || true)

# awk helper: given a file and a target 1-based line number, walks lines
# 1..target tracking brace depth and the most-recent #[cfg(test)] attribute.
# Prints "yes" if the target line is inside a #[cfg(test)] mod ... { ... } block,
# "no" otherwise.
#
# Per the design's shape-1 correction note, the saw_cfg_test reset block fires
# only on non-attribute, non-blank lines (lines matching ^\s*[^#\s]) — this
# lets attribute chains like `#[cfg(test)] #[other_attr] mod tests { ... }`
# still detect as cfg(test)-enclosed even though no such shape exists in-tree
# today.
is_cfg_test_enclosed() {
    local file=$1
    local target_line=$2
    awk -v target="$target_line" '
        BEGIN {
            depth = 0
            cfg_test_mod_depth = -1
            saw_cfg_test = 0
        }
        NR > target { exit }
        /^[[:space:]]*#\[cfg\(test\)\]/ {
            saw_cfg_test = 1
            next
        }
        /^[[:space:]]*mod[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*\{/ {
            if (saw_cfg_test && depth == 0) {
                cfg_test_mod_depth = depth
            }
            saw_cfg_test = 0
        }
        # Shape-1 correction: only reset saw_cfg_test on lines that are
        # neither attributes (^#) nor blank — attribute chains preserve the
        # pending cfg(test) marker.
        /^[[:space:]]*[^#[:space:]]/ { saw_cfg_test = 0 }
        {
            for (i = 1; i <= length($0); i++) {
                c = substr($0, i, 1)
                if (c == "{") depth++
                else if (c == "}") {
                    depth--
                    if (cfg_test_mod_depth >= 0 && depth <= cfg_test_mod_depth) {
                        cfg_test_mod_depth = -1
                    }
                }
            }
        }
        END {
            print (cfg_test_mod_depth >= 0 ? "yes" : "no")
        }
    ' "$file"
}

retained=()
skipped=()
while IFS= read -r hit; do
    [[ -z "$hit" ]] && continue
    # Split on the first two colons only; the matched text may itself contain colons.
    file=${hit%%:*}
    rest=${hit#*:}
    line=${rest%%:*}
    text=${rest#*:}

    # Shape-2 check: whole-file skip if this file's basename appears in the
    # sibling-attached-test-file set.
    base=${file##*/}
    if [[ -n "$sibling_attached_files" ]] && grep -Fxq "$base" <<<"$sibling_attached_files"; then
        skipped+=("$file:$line:$text")
        continue
    fi

    # Shape-1 check: backward-scan from line 1 up to line N tracking brace depth.
    enclosed=$(is_cfg_test_enclosed "$file" "$line")
    if [[ "$enclosed" == "yes" ]]; then
        skipped+=("$file:$line:$text")
        continue
    fi
    retained+=("$file:$line:$text")
done <<<"$hits_raw"

if (( ${#retained[@]} > 0 )); then
    echo "error: AC# acceptance-criteria tokens found in published-surface doc-comments:" >&2
    printf '  %s\n' "${retained[@]}" >&2
    echo "" >&2
    echo "See ai-docs/plans/2026-05-27-ac-doc-leak-guard.spec.md (issue #559)" >&2
    echo "for the rule. AC# tokens belong in test traceability (// line comments)," >&2
    echo "not in published rustdoc surfaces (///, //!, #[doc = \"...\"])." >&2
    exit 1
fi

exit 0
