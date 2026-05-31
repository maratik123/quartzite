# Triage-runner — Bridge action semantics

Extracted from .claude/agents/triage-runner.md § Phase 4.5 — Bridge sweep. Documents the verbatim action semantics for each bridge-detected conflict type.

**Action semantics:**

- **`update md`** (rewrite the JSON row via read-modify-write `Write`, no `>` redirect, no gh mutation):
  - **Type 1 (stale tracked):** rewrite the `tracked` value to keep `#N` and append ` (closed)` after it.
    - Thematic-file `.tracked`: `#60` → `#60 (closed)`.
    - `_inbox.jsonl` `.tracked`: same — `#60` → `#60 (closed)`.
    - `widget-backlog.jsonl` widget `.notes`: `tracked: #60 — needs button group` → `tracked: #60 (closed) — needs button group`.
  - **Type 2 (status mismatch, widget-backlog `.emoji_status` = `✅` vs OPEN gh issue):** follow-up prompt picks one of `🟡 v2` / `🤔 undecided` / `❌ dropped` / `📭 future` to replace `✅`. Defensible default: `🟡 v2` (OPEN means still planned but not done). `.notes` unchanged.
  - **Concurrent-edit guard** (B's Phase 6 / Phase 7.5 rule, verbatim): re-read the row's JSON line immediately before the write; abort with diff on mismatch; mtime not part of the check.

- **`update issue`** (write to gh, no md mutation):
  - **Type 1:** user asserts md row is right (work still open) — `gh issue reopen <N>`. Diff preview: `CLOSED` → `OPEN`. User confirms via yes/no prompt before the call runs.
  - **Type 2:** user asserts md row is right (work done) — `gh issue close <N>`. Diff preview: `OPEN` → `CLOSED`. User confirms.
  - **No `gh issue edit` calls in v1** — body drift is out of scope for the bridge.
  - **Failure handling:** if `gh issue close/reopen` fails, surface the error, leave the conflict unresolved in the run summary, continue with next conflict. No retry, no md mutation.

- **`keep both`**: no mutation. Capture user-supplied free-text reason in the bridge sub-section. Conflict re-surfaces on next `/triage` run (no marker is written that short-circuits it).
