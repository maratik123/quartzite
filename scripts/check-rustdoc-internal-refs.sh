#!/usr/bin/env bash
# scripts/check-rustdoc-internal-refs.sh
#
# Regression gate for the "no repo-internal references in published-rustdoc
# doc-comments" rule documented in ai-docs/doc-convention.md § Self-sufficiency.
# Authored 2026-05-21 for issue #336 (PR feat/2026-05-21-rustdoc-strip-internal-refs).
#
# Walks the workspace's Rust source tree (excluding tests/, benches/,
# quartzite-test-helpers/src/, and target/), runs the two audit patterns from
# the spec's ## Audit patterns section against every line beginning with a
# ///, //!, or #[doc = "..."] doc-comment, and exits non-zero on any retained
# match. The "#[cfg(test)]-region filter" (option b — backward-scan heuristic)
# drops hits that are inside a #[cfg(test)] sibling-file or an inline
# #[cfg(test)] mod ... { ... } block, because those lines are not part of the
# published rustdoc surface.
#
# Known #[cfg(test)]-enclosed false-positive sites in-tree at script-authoring
# time (worked examples — these MUST be filtered out and the gate MUST exit 0):
#
#   1. quartzite-style/src/default_style_tests.rs:2
#        Sibling-file shape. The file is attached via
#          quartzite-style/src/default_style/mod.rs:392-394
#            #[cfg(test)]
#            #[path = "../default_style_tests.rs"]
#            mod tests;
#        Every doc-comment in this file is consequently #[cfg(test)]-enclosed.
#
#   2. quartzite-runtime/src/timer_drivers.rs:450
#        Inline-shape. The file has #[cfg(test)] at line 425 followed
#        immediately by `mod tests {`, and line 450 is a doc-comment inside
#        that block.
#
#   3. quartzite-renderer/src/render_harness.rs:543
#        Inline-shape. The file has #[cfg(test)] at line 440 followed by
#        `mod tests {`, and line 543 is a doc-comment inside that block.
#
# Implementation-limitation note. The #[cfg(test)]-region filter is a
# heuristic, not a full Rust parser. It handles exactly the two shapes above:
#   - inline `#[cfg(test)] mod NAME { ... }` (with brace-depth tracking)
#   - sibling-file `#[cfg(test)] #[path = "NAME.rs"] mod IDENT;`
# More exotic shapes — `#[cfg(any(test, feature = "bar"))]`, nested
# #[cfg(test)] inside another `cfg`, doc-comments inside `cfg(test)`
# expression blocks rather than `mod` blocks — would need follow-up if they
# appear. The intent is to catch real published-surface leaks pre-merge, not
# to verify Rust semantics.
#
# Optional flags:
#   --list-skipped  print the file:line of each hit that was filtered out as
#                   `#[cfg(test)]`-enclosed (debugging / verification mode).
#
# Exit codes:
#   0  no retained published-surface hits
#   1  at least one retained published-surface hit (gate FAILS)
#   2  internal usage error (missing tool, bad flag)

set -euo pipefail

LIST_SKIPPED=0
case "${1:-}" in
    --list-skipped)
        LIST_SKIPPED=1
        shift
        ;;
    "")
        ;;
    *)
        echo "Usage: $0 [--list-skipped]" >&2
        exit 2
        ;;
esac

# Resolve repo root so the script works from any cwd.
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)
cd -- "$REPO_ROOT"

if ! command -v rg >/dev/null 2>&1; then
    echo "error: ripgrep (rg) not found on PATH" >&2
    exit 2
fi

# Patterns from ai-docs/doc-convention.md § Self-sufficiency, identical to the
# spec's ## Audit patterns section (Pattern A includes the round-3 design-system/,
# CONTRIBUTING.md, .claude/ tokens AND the round-5 bare \b#[0-9]{1,4}\b token;
# Pattern B includes the 2026-05-14 contributor-tooling tokens).
PATTERN_A='^\s*(///|//!).*(\bissue #[0-9]|\bPR #[0-9]|github\.com/.+/(issues|pull|tree|blob|commit|raw)/|ai-docs/|AGENTS\.md|CLAUDE\.md|CONTRIBUTING\.md|design-system/|\.claude/|\bspec AC[0-9]|\bplan #[0-9]|tracked in|deferred to a future (plan|spec)|\b#[0-9]{1,4}\b)'
PATTERN_B='^\s*(///|//!).*(\bVerify locally|\bcargo build -p|\bcargo test\b|\bcargo clippy\b|\bcargo fmt\b|RUSTDOCFLAGS|cargo doc --|scripts/[a-z]|\bthis PR\b|\bthis commit\b|\bthis implementation\b)'

# Raw hit list from both patterns, scoped to published-surface paths.
# Note: the test-surface globs match the spec's ## Out of scope §1 and Scope §4
# (quartzite-test-helpers now has [lib] doc = false, so its src/ is also out).
hits_raw=$(rg --type rust -n --no-heading \
    -e "$PATTERN_A" -e "$PATTERN_B" \
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
# if rg --multiline returns empty against default_style/mod.rs (where we know the
# shape exists in-tree), error out loudly so a future regex regression is
# caught at script-startup time, not silently masked as "no false-positives".
known_sibling_site=quartzite-style/src/default_style/mod.rs
if [[ ! -f $known_sibling_site ]]; then
    echo "error: known sibling-attached site $known_sibling_site not found" >&2
    echo "       update known_sibling_site in scripts/check-rustdoc-internal-refs.sh" >&2
    exit 2
fi
if ! rg --type rust --multiline -o "$sibling_re" "$known_sibling_site" >/dev/null 2>&1; then
    echo "error: shape-2 multiline regex no longer matches the known sibling-attached site $known_sibling_site:392-394" >&2
    echo "       update sibling_re in scripts/check-rustdoc-internal-refs.sh" >&2
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

if (( LIST_SKIPPED )); then
    if (( ${#skipped[@]} > 0 )); then
        echo "# Skipped (cfg(test)-enclosed):"
        printf '%s\n' "${skipped[@]}"
    else
        echo "# Skipped (cfg(test)-enclosed): (none)"
    fi
fi

if (( ${#retained[@]} > 0 )); then
    echo "error: repo-internal references found in published-surface doc-comments:" >&2
    printf '  %s\n' "${retained[@]}" >&2
    echo "" >&2
    echo "See ai-docs/doc-convention.md § Self-sufficiency: no repo-internal references" >&2
    echo "for the rule and Pattern A / Pattern B definitions." >&2
    exit 1
fi

exit 0
