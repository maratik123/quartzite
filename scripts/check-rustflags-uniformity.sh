#!/usr/bin/env bash
# scripts/check-rustflags-uniformity.sh
#
# Regression gate for the "RUSTFLAGS / RUSTDOCFLAGS strictness uniformity"
# rule documented in ai-docs/plans/2026-05-23-rustflags-strictness-audit.spec.md
# Authored 2026-05-23 for issue #385 (PR feat/2026-05-23-rustflags-strictness-audit).
#
# Walks .github/workflows/*.yml and asserts three orthogonal invariants:
#
#   A. No `rustflags:` input override on `actions-rust-lang/setup-rust-toolchain@vN`
#      (the action's default `-D warnings` MUST be inherited unchanged).
#
#   B. No `RUSTFLAGS:` env-var at any scope (workflow / job / step) — setting it
#      would override the action default.
#
#   C. Every cargo invocation that exercises rustdoc (`cargo doc`, `cargo rustdoc`,
#      `cargo llvm-cov ... --doctests`) carries an `env:` block with
#      `RUSTDOCFLAGS: "-D warnings -D missing-docs"` (literal canonical value).
#
# Allow-list shape mirrors scripts/check-rustdoc-internal-refs.sh: header comment
# names the approved overrides (workflow file × job, with one-line reason); a
# matching token in the corresponding bash array is required. At authoring time
# ALL THREE ALLOW-LISTS ARE EMPTY — the empty state is the asserted invariant.
#
# Step-shape assumption (Invariant C). The Invariant C scanner assumes single-
# line `run:` with the step's `env:` block placed AFTER the `run:` key within
# the step indent — true for every current rustdoc invocation. Future steps
# using `run: |`-multiline shapes or `env:` placed before `run:` would not be
# detected by this pattern alone. A future contributor introducing such a shape
# MUST either widen the pattern OR add an `ALLOWLIST_RUSTDOCFLAGS_EXEMPT` entry
# with a one-line reason.
#
# Known intentional overrides (workflow_file:job, one per line, with reason).
# Authoring time: ALL EMPTY. Adding an entry requires inline justification
# comment AND a matching token in the corresponding bash array.
#
#   Invariant A — rustflags-input override on setup-rust-toolchain@v1:
#     (none at authoring time)
#
#   Invariant B — RUSTFLAGS env-var override:
#     (none at authoring time)
#
#   Invariant C — cargo rustdoc invocation without RUSTDOCFLAGS env:
#     (none at authoring time)
#
# Exit codes:
#   0  all invariants hold
#   1  at least one invariant violated (gate FAILS)
#   2  internal usage error (missing tool, bad flag, PCRE2 unsupported)

set -euo pipefail

case "${1:-}" in
    "")
        ;;
    -h|--help)
        echo "Usage: $0" >&2
        echo "Asserts RUSTFLAGS / RUSTDOCFLAGS strictness uniformity across" >&2
        echo ".github/workflows/*.yml. Exits 0 if clean, 1 if any invariant" >&2
        echo "fails, 2 on usage / tooling error." >&2
        exit 2
        ;;
    *)
        echo "Usage: $0" >&2
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

# PCRE2 capability check — Invariant C uses `--pcre2 -U` for multi-line
# lookahead. A positive-match probe distinguishes "no match" (exit 1) from
# "feature missing / build error" (exit 2 with message).
if ! printf 'x\n' | rg --pcre2 -e 'x' >/dev/null 2>&1; then
    echo "error: ripgrep was built without PCRE2 support (--pcre2 unavailable)" >&2
    echo "       Invariant C requires PCRE2; install a ripgrep build with +pcre2." >&2
    exit 2
fi

# Allow-list arrays — see header comment for entry format / rationale.
# Adding an entry requires the inline justification AND a matching token here.
ALLOWLIST_RUSTFLAGS_INPUT=()      # rustflags: input on setup-rust-toolchain@v1
ALLOWLIST_RUSTFLAGS_ENV=()        # RUSTFLAGS: env-var override
ALLOWLIST_RUSTDOCFLAGS_EXEMPT=()  # rustdoc cargo step without RUSTDOCFLAGS env

# Token helper: a finding's token is "<workflow-basename>:<job-id>". Used to
# look the finding up in the corresponding allow-list array.
allowlist_contains() {
    local -n arr=$1
    local token=$2
    local entry
    for entry in "${arr[@]}"; do
        if [[ "$entry" == "$token" ]]; then
            return 0
        fi
    done
    return 1
}

# Job-id heuristic: given a workflow file + 1-based target line, walk lines
# 1..target and return the most-recent `^  <id>:` job header (two-space indent,
# YAML identifier, trailing colon, blank/newline). Falls back to "unknown" if
# no header is found above the target line.
job_id_for_line() {
    local file=$1
    local target_line=$2
    awk -v target="$target_line" '
        NR > target { exit }
        # Two-space indent + identifier + colon + EOL — job header shape.
        match($0, /^  [A-Za-z_][A-Za-z0-9_-]*:[[:space:]]*$/) {
            jid = $0
            sub(/^  /, "", jid)
            sub(/:.*$/, "", jid)
        }
        END {
            print (jid == "" ? "unknown" : jid)
        }
    ' "$file"
}

# Findings accumulators.
findings=()

# ---------------------------------------------------------------------------
# Invariant A — no `rustflags:` input override on setup-rust-toolchain@vN.
# ---------------------------------------------------------------------------
# Strategy: rg every `^\s*rustflags:` line in workflow files; for each hit,
# walk up at most 20 lines (awk) to find an enclosing
# `uses: actions-rust-lang/setup-rust-toolchain@`. If found, the hit is a
# rustflags-input override on the action — finding unless allow-listed.
while IFS= read -r hit; do
    [[ -z "$hit" ]] && continue
    file=${hit%%:*}
    rest=${hit#*:}
    line=${rest%%:*}
    # Walk back up to 20 lines looking for the setup-rust-toolchain `uses:`.
    enclosing=$(awk -v target="$line" '
        NR > target { exit }
        /actions-rust-lang\/setup-rust-toolchain@/ {
            uses_line = NR
        }
        END {
            print (uses_line == "" ? 0 : uses_line)
        }
    ' "$file")
    if [[ "$enclosing" -gt 0 ]] && (( line - enclosing <= 20 )); then
        job=$(job_id_for_line "$file" "$line")
        base=${file##*/}
        token="$base:$job"
        if ! allowlist_contains ALLOWLIST_RUSTFLAGS_INPUT "$token"; then
            findings+=("$file:$line: invariant-A: rustflags-input override on setup-rust-toolchain@v1 (job: $job)")
        fi
    fi
done < <(rg -n --no-heading '^\s*rustflags:' .github/workflows/ || true)

# ---------------------------------------------------------------------------
# Invariant B — no `RUSTFLAGS:` env-var anywhere in workflow files.
# ---------------------------------------------------------------------------
while IFS= read -r hit; do
    [[ -z "$hit" ]] && continue
    file=${hit%%:*}
    rest=${hit#*:}
    line=${rest%%:*}
    job=$(job_id_for_line "$file" "$line")
    base=${file##*/}
    token="$base:$job"
    if ! allowlist_contains ALLOWLIST_RUSTFLAGS_ENV "$token"; then
        findings+=("$file:$line: invariant-B: RUSTFLAGS env-var override (job: $job)")
    fi
done < <(rg -n --no-heading '^\s*RUSTFLAGS:\s' .github/workflows/ || true)

# ---------------------------------------------------------------------------
# Invariant C — every cargo rustdoc invocation carries
# `RUSTDOCFLAGS: "-D warnings -D missing-docs"`.
# ---------------------------------------------------------------------------
# Strategy: line-number capture of every `run:` line containing a rustdoc
# token. For each hit, capture the next N lines (bounded by either ~20 lines
# or the next `- name:` step header, whichever comes first) and assert that
# the literal substring `RUSTDOCFLAGS: "-D warnings -D missing-docs"` appears.
#
# This avoids the multiline-rg pattern's brittleness on captured-block boundaries
# while staying within the step-shape assumption documented in the header.
CANONICAL='RUSTDOCFLAGS: "-D warnings -D missing-docs"'
while IFS= read -r hit; do
    [[ -z "$hit" ]] && continue
    file=${hit%%:*}
    rest=${hit#*:}
    line=${rest%%:*}
    # Capture up to 20 lines starting at $line, stopping at the next step header.
    block=$(awk -v start="$line" -v max=20 '
        NR < start { next }
        NR == start { collected = $0; n = 1; next }
        NR > start {
            # Stop at the next step header (a `- name:` line at any indent).
            if ($0 ~ /^[[:space:]]*-[[:space:]]+name:/) { exit }
            collected = collected "\n" $0
            n++
            if (n >= max) { exit }
        }
        END { print collected }
    ' "$file")
    if [[ "$block" != *"$CANONICAL"* ]]; then
        job=$(job_id_for_line "$file" "$line")
        base=${file##*/}
        token="$base:$job"
        if ! allowlist_contains ALLOWLIST_RUSTDOCFLAGS_EXEMPT "$token"; then
            findings+=("$file:$line: invariant-C: cargo rustdoc invocation missing RUSTDOCFLAGS (job: $job)")
        fi
    fi
done < <(rg -n --no-heading --pcre2 \
    -e '^\s*run:.*\bcargo\s+doc(\s|$)' \
    -e '^\s*run:.*\bcargo\s+rustdoc(\s|$)' \
    -e '^\s*run:.*\bcargo\s+llvm-cov\b.*--doctests' \
    .github/workflows/ || true)

# ---------------------------------------------------------------------------
# Output / exit.
# ---------------------------------------------------------------------------
if (( ${#findings[@]} > 0 )); then
    echo "error: RUSTFLAGS / RUSTDOCFLAGS strictness uniformity violated:" >&2
    printf '  %s\n' "${findings[@]}" >&2
    echo "" >&2
    echo "See ai-docs/plans/2026-05-23-rustflags-strictness-audit.spec.md" >&2
    echo "for invariant definitions and allow-list shape." >&2
    exit 1
fi

exit 0
