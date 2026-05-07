#!/usr/bin/env bash
# gen-roadmap.sh — POSIX bash + awk/sed generator for ROADMAP.md.
#
# Reads  : ai-docs/plans/INDEX.md
# Writes : ROADMAP.md (at repo root)
# Determinism: no timestamps, no random output; same input bytes -> same output bytes.
#
# Load-bearing INDEX.md headings (renaming any of these silently breaks the gate):
#   Extraction triggers:
#     ## Active plans
#     ## Completed plans
#     ## Deferred plans
#     ## Dependency order        (output is renamed to "## Dependency tree")
#   Output terminator:
#     ## Suggested next steps    (everything from this line down is dropped)
#
# Banned non-portable constructs (POSIX.1-2008 baseline; CI gate runs only on
# ubuntu-latest / GNU coreutils, so portability bugs do NOT surface in CI —
# they break only for macOS / BSD-awk contributors running this locally):
#   sed -E / sed -r        — extended regex flags are GNU/macOS-only, not POSIX.
#                            Use POSIX BRE; for character classes prefer
#                            [[:alpha:]] / [[:digit:]] / [[:space:]].
#   awk gensub(...)        — GNU awk extension. Use chained gsub(...) calls.
#   awk --re-interval      — already enabled by default in POSIX awk; the flag
#                            itself is GNU-only.
#   sed -i (in-place edit) — flag semantics differ between GNU (-i) and BSD
#                            (-i ''). Pipe through tee to a tempfile or rewrite
#                            the whole file via printf/awk redirection instead.
#   bash [[ ... ]] tests   — bash-specific; use POSIX [ ... ] or case.
#   bash (( ... )) arith   — bash-specific; use POSIX $(( ... )) or expr.
#   mapfile / readarray    — bash 4+ only; use a
#                            "while IFS= read -r line; do ...; done < file" loop.
#
# Reference: POSIX.1-2008 utilities (awk, sed, sh).

set -eu
LC_ALL=C
export LC_ALL

# Resolve repo root from the location of this script (scripts/ lives at repo root).
script_dir=$(cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)
index_md="$repo_root/ai-docs/plans/INDEX.md"
out_md="$repo_root/ROADMAP.md"

if [ ! -f "$index_md" ]; then
    printf 'gen-roadmap.sh: cannot find %s\n' "$index_md" >&2
    exit 1
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT HUP TERM

active_raw="$tmpdir/active.raw"
completed_raw="$tmpdir/completed.raw"
deferred_raw="$tmpdir/deferred.raw"
deptree_raw="$tmpdir/deptree.raw"

# Extract the four blocks via an awk state machine.
#
# State variables:
#   state -- one of: "" | "active" | "completed" | "deferred" | "deptree"
#   done  -- 1 once "## Suggested next steps" is reached; from then on, no
#            extraction occurs.
#
# Blocks ("active" / "completed" / "deferred") capture lines from the heading's
# table (starting with the row beginning "| Plan |") until the next "## " heading
# or the terminator. The "deptree" block captures the contents of the first
# triple-backtick fenced block following "## Dependency order".
awk '
    BEGIN {
        state = ""
        done_flag = 0
        in_fence = 0
    }
    {
        if (done_flag) { next }

        # Terminator: stop extracting at "## Suggested next steps".
        if ($0 ~ /^## Suggested next steps[[:space:]]*$/) {
            done_flag = 1
            state = ""
            next
        }

        # Section openers. We only switch state on H2 lines that exactly match
        # the four triggers. Any other H2 (or deeper) closes the current state.
        if ($0 ~ /^## Active plans[[:space:]]*$/) {
            state = "active"
            next
        }
        if ($0 ~ /^## Completed plans[[:space:]]*$/) {
            state = "completed"
            next
        }
        if ($0 ~ /^## Deferred plans[[:space:]]*$/) {
            state = "deferred"
            next
        }
        if ($0 ~ /^## Dependency order[[:space:]]*$/) {
            state = "deptree"
            in_fence = 0
            next
        }
        # Any other "## " heading closes whatever block we were in.
        if ($0 ~ /^## /) {
            state = ""
            next
        }

        # For the three plan-table blocks, capture only table lines
        # (lines starting with "|"). The first non-blank, non-pipe line
        # after the table closes the block — that is how we drop the
        # trailing "> Tracking issues..." blockquote that lives between
        # "## Deferred plans" and "## Dependency order" in INDEX.md.
        if (state == "active" || state == "completed" || state == "deferred") {
            if (substr($0, 1, 1) == "|") {
                if (state == "active") {
                    print > "'"$active_raw"'"
                } else if (state == "completed") {
                    print > "'"$completed_raw"'"
                } else {
                    print > "'"$deferred_raw"'"
                }
            } else if ($0 ~ /^[[:space:]]*$/) {
                # Blank line: keep waiting; do not emit, do not close.
                # (No state change — a blank between two table chunks
                # would be unusual but not fatal.)
                next
            } else {
                # Non-blank, non-pipe line ends the table block.
                state = ""
            }
            next
        }
        if (state == "deptree") {
            # Capture the contents of the first fenced code block only.
            if ($0 ~ /^```/) {
                if (in_fence == 0) {
                    in_fence = 1
                } else {
                    # Closing fence -> stop deptree extraction.
                    in_fence = 0
                    state = ""
                }
                next
            }
            if (in_fence == 1) {
                print > "'"$deptree_raw"'"
            }
        }
    }
' "$index_md"

# Each raw file is created by awk only if the block emitted at least one line.
# Touch them so later cat/sed always have something to read.
for f in "$active_raw" "$completed_raw" "$deferred_raw" "$deptree_raw"; do
    if [ ! -f "$f" ]; then
        : > "$f"
    fi
done

# Trim leading and trailing blank lines from a captured block. POSIX awk only.
trim_blanks() {
    src=$1
    dst=$2
    awk '
        /^[[:space:]]*$/ {
            if (started) { blanks = blanks $0 "\n" }
            next
        }
        {
            if (started) { printf "%s", blanks }
            blanks = ""
            started = 1
            print
        }
    ' "$src" > "$dst"
}

active_trim="$tmpdir/active.trim"
completed_trim="$tmpdir/completed.trim"
deferred_trim="$tmpdir/deferred.trim"
deptree_trim="$tmpdir/deptree.trim"

trim_blanks "$active_raw" "$active_trim"
trim_blanks "$completed_raw" "$completed_trim"
trim_blanks "$deferred_raw" "$deferred_trim"
trim_blanks "$deptree_raw" "$deptree_trim"

# Sed link-rewrite for the three known prefixes only. POSIX BRE; use alternate
# delimiter | so the / inside the replacement does not need escaping. Multiple
# -e clauses instead of -E alternation.
rewrite_links() {
    src=$1
    dst=$2
    sed \
        -e 's|](done/|](ai-docs/plans/done/|g' \
        -e 's|](deferred/|](ai-docs/plans/deferred/|g' \
        -e 's|](2026-|](ai-docs/plans/2026-|g' \
        "$src" > "$dst"
}

active_out="$tmpdir/active.out"
completed_out="$tmpdir/completed.out"
deferred_out="$tmpdir/deferred.out"
deptree_out="$tmpdir/deptree.out"

rewrite_links "$active_trim" "$active_out"
rewrite_links "$completed_trim" "$completed_out"
rewrite_links "$deferred_trim" "$deferred_out"
# The dependency-tree code block in INDEX.md contains no plan-link prefixes
# we rewrite, but pipe it through the same pass for consistency (no-op).
rewrite_links "$deptree_trim" "$deptree_out"

# Emit the final document. Use printf for literals (deterministic, no shell
# escape surprises) and cat for the awk-extracted blocks. The single-quoted
# header strings contain markdown backtick code-spans on purpose; shellcheck
# SC2016 ("expressions don't expand in single quotes") is a false positive
# here — we deliberately want zero shell expansion of the literal text.
# shellcheck disable=SC2016
{
    printf '%s\n' \
        '# quartzite — Roadmap' \
        '' \
        '> **Auto-generated** from [`ai-docs/plans/INDEX.md`](ai-docs/plans/INDEX.md) by' \
        '> [`scripts/gen-roadmap.sh`](scripts/gen-roadmap.sh). Do not edit by hand —' \
        '> changes here will be reverted by the CI sync-gate. Edit `INDEX.md` instead' \
        '> and re-run the generator.' \
        '' \
        'Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked' \
        '' \
        '## Dependency tree' \
        '' \
        '```'
    cat "$deptree_out"
    printf '%s\n' \
        '```' \
        '' \
        '## Active plans' \
        ''
    cat "$active_out"
    printf '\n%s\n\n' '## Completed plans'
    cat "$completed_out"
    printf '\n%s\n\n' '## Deferred plans'
    cat "$deferred_out"
} > "$out_md"

printf 'gen-roadmap.sh: wrote %s\n' "$out_md"
