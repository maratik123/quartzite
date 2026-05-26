#!/usr/bin/env bash
# Heuristic check for #[inline] / _Simple._ axiom violations in a single .rs file.
# Invoked by the PostToolUse Edit|Write hook in .claude/settings.json.
#
# Detects two shapes the recurrence record on `ai-docs/learnings.md` flags
# repeatedly (10+ occurrences with rule already in AGENTS.md + code-style.md +
# self-review.md + review-findings.md):
#   1. `#[inline]` and `_Simple._` within 2 lines of each other  (always wrong)
#   2. `#[inline]` on a fn whose body contains `match` / `if` / `while` / `for`
#      / `loop`  (best-effort regex; false positives acceptable per the same
#      model as commit 1da36b0's ROADMAP hook).
#
# Always exits 0 (non-blocking). Findings go to stderr so the model sees them
# in the next prompt and can self-correct in the same turn.

set -u

f="${1:-}"
if [ -z "$f" ] || [ ! -f "$f" ]; then
  exit 0
fi

case "$f" in
  *.rs) ;;
  *) exit 0 ;;
esac

case "$f" in
  */tests/*|*/benches/*|*/examples/*) exit 0 ;;
esac

cooc=$(awk '
  BEGIN { inl=0; simp=0 }
  /^[[:space:]]*#\[inline\]/ {
    if (simp && NR-simp <= 2) {
      print FILENAME":"simp": _Simple._ (line "simp") co-occurs with #[inline] (line "NR")"
      inl=0; simp=0; next
    }
    inl=NR; next
  }
  /^[[:space:]]*\/{2,3}[[:space:]]*_Simple\._/ {
    if (inl && NR-inl <= 2) {
      print FILENAME":"inl": #[inline] (line "inl") co-occurs with _Simple._ (line "NR")"
      inl=0; simp=0; next
    }
    simp=NR
  }
' "$f")

branch=$(awk '
  BEGIN { inl=0; depth=0; saw_fn=0; seen_open=0 }
  /^[[:space:]]*#\[inline\]/ { inl=NR; depth=0; saw_fn=0; seen_open=0; next }
  inl && /(^|[^A-Za-z0-9_])fn[[:space:]]+[A-Za-z_]/ { saw_fn=1 }
  inl && saw_fn {
    d_open = split($0, tmp, "{") - 1
    d_close = split($0, tmp, "}") - 1
    depth += d_open - d_close
    if (d_open > 0) seen_open=1
    if (depth > 0 && /(^|[^A-Za-z0-9_])(match|if|while|for|loop)([^A-Za-z0-9_]|$)/ && !/^[[:space:]]*\/\//) {
      print FILENAME":"inl": #[inline] fn declared near line "inl" contains branching keyword at line "NR
      inl=0; saw_fn=0; depth=0; seen_open=0
      next
    }
    if (seen_open && depth == 0) { inl=0; saw_fn=0; depth=0; seen_open=0 }
  }
' "$f")

if [ -n "$cooc" ] || [ -n "$branch" ]; then
  printf '\n[inline-gate] #[inline] / _Simple._ axiom violations (AGENTS.md § Code Style):\n' >&2
  [ -n "$cooc" ] && printf '%s\n' "$cooc" >&2
  [ -n "$branch" ] && printf '%s\n' "$branch" >&2
  printf '[inline-gate] strip #[inline] from branching fns; never co-occur #[inline] with _Simple._. See ai-docs/code-style.md § #[inline] and _Simple._.\n' >&2
fi

exit 0
