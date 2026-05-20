# Design: Allow `ast-index` in permissions + keep index fresh on commit

**Issue:** #507
**Date:** 2026-05-20

## Approach

This task is an instruction-file + dev-tooling change with **zero Rust code**. Three artefacts mutate:

1. `.claude/settings.json` — append three wildcard `permissions.allow` entries (`Bash(ast-index *)`, `Bash(grep *)`, `Bash(rg *)`) and append a new `PreToolUse` Bash hook for `git commit` that drives `scripts/update-ast-index.sh`.
2. `.claude/settings.local.json` — delete the two migrated wildcards (`Bash(grep *)`, `Bash(rg *)`) and the **32 specific-`rg` + 5 specific-`grep` entries** they cover.
3. `scripts/update-ast-index.sh` — a new POSIX `sh` wrapper.

### Chosen solution

**Permissions form: space-form (`Bash(<bin> *)`).** Confirmed by direct inspection of `.claude/settings.json` (lines 85–87 already use `Bash(cargo *)`, `Bash(git *)`, `Bash(gh *)`) and 90%+ of `.claude/settings.local.json` (`mv *`, `mkdir *`, `rustfmt *`, `grep *`, `rg *`, `xargs rg *`, `awk *`, etc.). The only colon-form outlier is `Bash(echo:*)` at line 567 — and the same project session today has been using both `Bash(ast-index --help)` (line 627) and `Bash(command -v ast-index)` (line 628) without trouble, so the space-form clearly works for this binary too. Decision: **adopt space-form for all three search-tool wildcards**, no fallback needed. (Open question Q1 resolved.)

**Rebuild-fallback detection: stdout sentinel string.** Live probe of `/home/syt/.cargo/bin/ast-index v3.42.0`:
- Running `ast-index update` or `ast-index stats` in a directory with no DB writes `Index not found. Run 'ast-index rebuild' first.` to **stdout** and exits **0**.
- Exit code alone cannot distinguish "no DB" from "successful incremental update".
- Therefore the script captures `ast-index update`'s combined output, exits 0 immediately on success (no sentinel in output), and on detecting the sentinel string `Index not found` it invokes `ast-index rebuild` once and surrenders.

**Hook shape: mirror `gen-roadmap.sh` precedent at `.claude/settings.json:27` but invert the blocking semantics.** The existing `gen-roadmap.sh` hook uses a `jq -r .tool_input.command` extractor + `grep -qE '(^|[ ;&|`])git[[:space:]]+commit\b'` substring guard, and emits `exit 2` on failure (blocking) because `ROADMAP.md` is checked-in. For ast-index, the DB is host-local and gitignored, so **the hook command must always `exit 0`** — even a future bug in the script must not block a commit. Implementation pattern (literal, for the design — code goes in the implementation phase):

```
cmd=$(jq -r '.tool_input.command // empty'); \
if echo "$cmd" | grep -qE '(^|[ ;&|`])git[[:space:]]+commit\b'; then \
  bash "$CLAUDE_PROJECT_DIR/scripts/update-ast-index.sh" >/dev/null 2>&1 || true; \
fi; \
exit 0
```

Note the **trailing `exit 0`**: it makes the always-success contract explicit + grep-able + auditable. The `|| true` on the `bash` invocation is redundant belt-and-suspenders alongside the script's own internal `exit 0` guarantees.

**Script shape: `bash` with `set -eu`, four effective lines of logic. Shebang `#!/usr/bin/env bash` matching the `gen-roadmap.sh` precedent** (`scripts/gen-roadmap.sh` is also `#!/usr/bin/env bash`, despite its in-file comment claiming "POSIX bash + awk/sed"). The hook literal snippet above invokes the script via `bash`, so the runtime interpreter and the shebang agree — no risk of POSIX-only-via-shellcheck drift. (Option A — keep POSIX `sh` shebang and change the hook to `sh` — was rejected: it forfeits the precedent match with `gen-roadmap.sh`, and the script body uses only constructs that work identically in `sh` and `bash`, so the bash choice is essentially free.)
- Line 1: `command -v ast-index >/dev/null 2>&1 || exit 0` — no-op when binary absent (host without ast-index installed).
- Line 2: `out=$(ast-index update 2>&1) || true` — capture combined output, swallow non-zero exits.
- Line 3: `case "$out" in *"Index not found"*) ast-index rebuild >/dev/null 2>&1 || true ;; esac` — fallback to full rebuild when DB missing.
- Line 4: `exit 0` (explicit — `set -e` does not save us here because every line is guarded).

### Rejected alternatives

| Alternative | Rejected because |
|---|---|
| Per-subcommand allow-list entries (`Bash(ast-index search *)`, `Bash(ast-index symbol *)`, …) | YAGNI; the issue's stated goal is "stop prompting for permission" — a single wildcard achieves that. Per-subcommand grain would re-create the 30+ specific-entries clutter we are deleting from `.claude/settings.local.json`. |
| Putting the new ast-index allow-entries in `.claude/settings.local.json` (host-local) | Spec Round 1 Q1 + Round 2 Q1 settled this: project-shared `.claude/settings.json` so every contributor's agent benefits; matches where the hook entry has to live anyway. |
| Detecting "missing DB" via exit code rather than stdout sentinel | Empirically falsified — `ast-index update` exits 0 with the missing-DB message. Re-checking via a separate `ast-index stats` probe before `update` doubles the cold-cache overhead on the happy path. |
| Using `--format json` and `jq` to detect the missing-DB case structurally | Adds `jq` to the script's hot path even when the binary is installed and the DB is fresh — extra fork on **every** commit. Sentinel-grep is cheap, robust against minor wording changes (the substring `Index not found` is stable across `update` + `stats` + `search` outputs), and keeps the script free of non-POSIX deps. |
| Long-lived `ast-index watch` daemon as the primary sync mechanism | Explicitly rejected in the issue body. The pre-commit hook is the chosen mechanic. |
| Sharing the DB via git | Out of scope per spec — DB is host-local + gitignored. |
| Removing `Bash(xargs rg *)` (line 29) as part of the consolidation | Out of scope. It is a compound entry (`xargs rg`), not a standalone `Bash(rg ...)` — the migrated `Bash(rg *)` wildcard alone does not cover an `xargs rg` invocation under Claude Code's permission semantics. Leave it. |
| Removing the 5 redundant `Bash(ast-index ...)` specific entries (`.claude/settings.local.json` lines 627, 629–632 — `ast-index --help`, `ast-index update *`, `ast-index stats *`, `ast-index rebuild *`, `ast-index --version`) | **Follow-up, out of scope per spec §4.** Once the wildcard `Bash(ast-index *)` lands in `.claude/settings.json`, these 5 entries become redundant on the same grounds as the 32 + 5 `rg`/`grep` entries being removed in subtask 4. The asymmetry is intentional: spec §4 explicitly carves the scope to the `rg` / `grep` consolidation only. Surface as a follow-up housekeeping pass (Step 12 of `/task` propagates this row to `ai-docs/deferred/_inbox.md` automatically — no hand-edit). |

### Spec count reconciliation

The spec headline `33 + 6` counts the wildcards plus specifics; the enumerated line list (which is what subtask 4 executes against) is `32 + 5`. Both are mutually consistent; AC3's grep gate (`= 0`) is the authoritative final check. The implementation follows the enumerated line list verbatim; the headline numerics are not consulted at edit time.

### Gitignore deviation (recorded post-Group-B)

`.claude/settings.local.json` is gitignored (`.gitignore:3:/.claude/settings.local.json`); `git ls-files` confirms it has never been tracked. Subtask 4's cleanup therefore mutates the file **on disk only** and produces **no commit** for the PR. AC2 and AC3 are satisfied locally on this host, but they do **not** propagate cross-host via git — other contributors' `.claude/settings.local.json` files remain untouched until each runs the same cleanup. The original "Stage `.claude/settings.local.json`" instruction in subtask 5 was impossible and was dropped at implementation time. The cross-host deliverable lives entirely in `.claude/settings.json` (the migrated wildcards + the new ast-index entry + the PreToolUse hook), which IS committed and therefore propagates. This deviation is intentional and is reflected in the spec's AC2 / AC3 wording (the host-local qualifier added during Step 12 finalisation).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create feature branch `feat/2026-05-20-ast-index-allowlist-and-hook`, then create `scripts/update-ast-index.sh` with: `#!/usr/bin/env bash` shebang (matches `gen-roadmap.sh` precedent and the hook's `bash` invocation) + `set -eu`, the `command -v ast-index` no-op guard, the `ast-index update` capture, the `Index not found` sentinel-grep with `ast-index rebuild` fallback, and explicit final `exit 0`. `chmod +x` the file. Verify with `shellcheck scripts/update-ast-index.sh` (shellcheck auto-detects the bash shebang; same shellcheck-clean baseline as `gen-roadmap.sh`) and three local smoke runs: (a) on this host (binary present, DB present) — exits 0, ≤1s; (b) inside `/tmp/ast_no_db_test` (binary present, DB absent) — exits 0, triggers rebuild path; (c) `PATH=/nonexistent bash scripts/update-ast-index.sh` (binary absent) — exits 0, no output. | `scripts/update-ast-index.sh` (NEW) | — |
| 2 | Migrate the two wildcards to `.claude/settings.json` and add the ast-index entry: append `"Bash(ast-index *)"`, `"Bash(grep *)"`, `"Bash(rg *)"` to `permissions.allow` after `"Bash(gh *)"` (line 87). Order them alphabetically among themselves for diff stability: `ast-index`, `grep`, `rg`. Validate with `python3 -m json.tool < .claude/settings.json` (JSON well-formedness). Verify `wc -c .claude/settings.json` stays well below 35 000 / 40 000 (current 6 665 → ~6 800 after delta). | `.claude/settings.json` | 1 |
| 3 | Add the `PreToolUse` hook entry to `.claude/settings.json` `hooks.PreToolUse[0].hooks` (the `matcher: "Bash"` block currently containing the branch-guard + gen-roadmap hooks). Append a third hook object after the gen-roadmap entry, mirroring its `jq -r '.tool_input.command // empty'` + `grep -qE '(^|[ ;&|`])git[[:space:]]+commit\b'` extractor shape, calling `bash "$CLAUDE_PROJECT_DIR/scripts/update-ast-index.sh" >/dev/null 2>&1 \|\| true; exit 0`. Set `timeout: 15`, `statusMessage: "Refreshing ast-index..."`. Validate JSON; do a wet-run by `touch`ing a `.rs` file then `git commit --allow-empty -m "design: hook smoke"` on a throwaway branch (do NOT push) and confirm: hook fires once, no permission prompt, `ast-index stats` reflects the new mtime. Discard the throwaway commit with `git reset --soft HEAD~1 && git restore --staged .` before moving on. | `.claude/settings.json` | 2 |
| 4 | Delete from `.claude/settings.local.json`: (a) the standalone `Bash(grep *)` entry (line 32 in pre-edit numbering); (b) the standalone `Bash(rg *)` entry (line 341 in pre-edit numbering); (c) the 32 specific-`rg` entries enumerated by line in the spec's Scope §3 (live lines 56, 59, 61, 69, 70, 76, 77, 79, 80, 81, 86, 87, 88, 89, 90, 114, 133, 134, 152, 153, 154, 155, 156, 157, 158, 199, 253, 334, 335, 336, 339, 340); (d) the 5 specific-`grep` entries (live lines 25, 27, 384, 385, 386). **Do NOT touch** `Bash(xargs rg *)` (line 29) — it is a compound `xargs rg` entry, not a standalone `Bash(rg ...)`, and is out of scope. Validate JSON: `python3 -m json.tool < .claude/settings.local.json`. Verify the AC3 grep: `grep -cE 'Bash\((rg\|grep) ' .claude/settings.local.json` returns `0`. (Edit pattern: in a single Edit invocation per entry, or one multi-line Edit. Editing in descending line order avoids shifting line numbers mid-edit.) | `.claude/settings.local.json` | 3 |
| 5 | End-to-end verification on the developer host (AC8 + AC9): (a) edit a real `.rs` file (e.g. add a doc-comment to any existing pub fn), `git add` it, run `git commit -m "test: hook smoke"`, then immediately `git reset --soft HEAD~1 && git restore --staged .` to discard. Confirm the hook ran (`ast-index stats` symbol count includes the new doc-comment lines). (b) Run any `ast-index <subcommand>` invocation that has never been allow-listed individually (e.g. `ast-index symbol Renderer`) and confirm no permission prompt. (c) Re-run `python3 -m json.tool` on both settings files for the last time. Stage `.claude/settings.json`, `.claude/settings.local.json`, `scripts/update-ast-index.sh`, plus `ai-docs/plans/2026-05-20-ast-index-allowlist-and-hook.{spec,design}.md`, plus the `ai-docs/learnings.md` entries accumulated in-flight, and commit on the feature branch. Self-review per `/task` Step 10 before push. | (verification only — no file mutations beyond commit) | 4 |

## Handoff plan

`M = 5` subtasks → two groups (3 + 2). The first group does the file mutations that depend on each other linearly (script, settings.json wildcards, settings.json hook); the second group does the bulk JSON edit + end-to-end verification, where context-reset payoff is highest because the bulk edit benefits from fresh attention.

- **Group A:** subtasks 1–3 — initial implementation chunk (script creation, settings.json wildcards, settings.json hook entry). Parent `/task` Step 8 enters this group via a `/context-reset` subagent per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — terminal group (2 subtasks; within the `1..=3` range). The 39-entry deletion in subtask 4 is the largest single edit in the task and benefits from the post-handoff fresh context; subtask 5 is the verification + commit step.

## Risks

- **Permissions-schema rejection of `Bash(ast-index *)` space-form.** *Mitigation:* the same session has already used `Bash(ast-index --help)` and `Bash(command -v ast-index)` without prompts, proving the binary is allow-listable; space-form is the dominant pattern in both settings files. If a schema rejection nonetheless surfaces during Subtask 3's wet-run (manifesting as a permission prompt for an `ast-index` invocation), drop the entry, re-add it as `Bash(ast-index:*)`, and apply the colon form to all three search-tool wildcards in `.claude/settings.json` for consistency.
- **Hook fires on `git commit -m '... including the word `git commit` in the message body'`.** *Mitigation:* the substring guard is shared with the `gen-roadmap.sh` hook, which has the identical false-positive surface; if it has not been a problem there, it will not be a problem here. The script is no-op when invoked redundantly, so a false-positive is harmless beyond ~1 s of extra commit latency.
- **`ast-index rebuild` is slow on cold-cache hosts and the fallback runs synchronously in the hook.** *Mitigation:* (a) the fallback only fires once — after the first commit on a host without a pre-existing index — and (b) the hook `timeout: 15` from `gen-roadmap.sh` is too tight for a cold `ast-index rebuild` on a large project (this workspace took ~770 ms for an incremental update; a full rebuild on a slow disk could exceed 15 s). Set the hook `timeout: 30` for headroom while still being substantially shorter than a reasonable commit budget. Confirmed acceptable per spec ("never blocks the commit" — even if the hook times out, the script's own `exit 0` path means no `exit 2` is propagated; worst case the hook is killed mid-rebuild and the next commit retries from a partial DB).
- **JSON edit accidentally creates trailing-comma syntax error in `.claude/settings.local.json`.** *Mitigation:* subtask 4 ends with `python3 -m json.tool < .claude/settings.local.json` as a forced gate; Subtask 5 re-runs it. Order edits in **descending** line number so the line numbers in the deletion list stay valid throughout the pass. Last entry on the array is line 628; if a removal happens to be last, ensure no trailing comma.
- **`scripts/update-ast-index.sh` runs even when the developer has explicitly chosen not to install `ast-index`.** *Mitigation:* `command -v ast-index || exit 0` as the first effective line; verified in Subtask 1's smoke (c).
- **Hook double-fires due to `git commit --amend` or `git commit -p` substring match.** *Mitigation:* both still match `git[[:space:]]+commit\b` — but they also actually commit, so the index needs refreshing in both cases. No mitigation needed; it is correct behaviour.

## Test Design

This task has no Rust code, so there is no `#[cfg(test)] mod tests` to write. The test plan is the per-subtask shell-level verification:

### Subtask 1 — `scripts/update-ast-index.sh`

- **Tool:** `shellcheck scripts/update-ast-index.sh` (already available, verified in Subtask 1 prep). Same baseline as `gen-roadmap.sh` — must pass clean.
- **Smoke (a) — binary present, DB present** (this host): `bash scripts/update-ast-index.sh && echo "exit=$?"` → `exit=0`, completes in <1 s, no stderr output. Then `ast-index stats` shows updated symbol count if any `.rs` file was edited since the last update.
- **Smoke (b) — binary present, DB absent**: `cd /tmp && mkdir -p ast_smoke && cd ast_smoke && bash /home/syt/RustroverProjects/quartzite/scripts/update-ast-index.sh; echo exit=$?` → `exit=0`. (Subsequent `ast-index stats` in `/tmp/ast_smoke` shows a freshly-rebuilt empty index or "Index not found" again if rebuild also failed because the dir has no project files — both acceptable, both `exit 0`.)
- **Smoke (c) — binary absent**: `env -i HOME=/tmp PATH=/usr/local/bin:/usr/bin:/bin bash -c 'command -v ast-index || echo "binary not found"; bash /home/syt/RustroverProjects/quartzite/scripts/update-ast-index.sh; echo exit=$?'`. With `/home/syt/.cargo/bin` purged from `PATH`, expect `binary not found` line followed by `exit=0`, with **no other output** between them.

### Subtask 2 — `.claude/settings.json` permissions delta

- **Tool:** `python3 -m json.tool < .claude/settings.json` must succeed.
- **Spot-check:** `jq -r '.permissions.allow[]' .claude/settings.json | grep -E '^Bash\((ast-index|grep|rg) \*\)$'` returns exactly three lines.
- **Negative spot-check:** `jq '.permissions.allow | length' .claude/settings.json` returns `12` (previously 9 — `Bash(cargo *)`, `Bash(git *)`, `Bash(gh *)` plus 6 Edit/Write entries; now +3).

### Subtask 3 — `.claude/settings.json` hook delta

- **Tool:** `python3 -m json.tool < .claude/settings.json` must succeed.
- **Spot-check:** `jq '.hooks.PreToolUse[0].hooks | length' .claude/settings.json` returns `3` (previously 2 — branch-guard + gen-roadmap; now +1).
- **Wet-run:** on a throwaway empty commit on a throwaway branch (`git checkout -b throwaway-hook-smoke; git commit --allow-empty -m 'test'`), confirm the hook output line includes `Refreshing ast-index...` and the commit succeeds. Discard branch + commit. (DO NOT push the throwaway commit.)

### Subtask 4 — `.claude/settings.local.json` cleanup

- **Tool:** `python3 -m json.tool < .claude/settings.local.json` must succeed.
- **AC3 gate:** `grep -cE 'Bash\((rg|grep) ' .claude/settings.local.json` returns `0`. (Note: this command intentionally **does not** match `Bash(xargs rg *)` which is preserved.)
- **Negative spot-check:** `jq '.permissions.allow | length' .claude/settings.local.json` returns `(previous - 39)` — i.e., 626 - 39 = 587, give or take depending on exact pre-edit count. Use this as a sanity check, not a strict gate.

### Subtask 5 — end-to-end

- **AC8 wet-run:** real `.rs` file edit + `git commit` + verify `ast-index stats` reflects change. Then `git reset --soft HEAD~1 && git restore --staged .` to discard.
- **AC9 wet-run:** `ast-index symbol Renderer` (or any subcommand) in an agent context with no per-invocation permission prompt.
- **Final JSON sanity:** both settings files re-validated with `python3 -m json.tool`.

### Fixtures / helpers

None new. All verification uses tools already available in the environment (`python3`, `jq`, `shellcheck`, `git`, `ast-index`).

## Open questions

Both spec-flagged "non-blocking, design-only" questions are now resolved:

1. **Permissions schema accepts `Bash(ast-index *)` space-form** — resolved by inspection of `.claude/settings.json:85–87` (`Bash(cargo *)`, `Bash(git *)`, `Bash(gh *)`) plus the current session's frictionless use of `Bash(ast-index --help)` (line 627 of `.claude/settings.local.json`). No colon-form fallback needed; the **risk** section retains a one-paragraph contingency in case schema behaviour drifts during implementation.
2. **`ast-index update` already prunes stale entries for removed files** — confirmed by the verbose probe `ast-index update -v` (output: `Found 2 new/changed files, 0 deleted files` shows the deleted-files counter is part of normal `update` semantics). No separate prune step needed; `scripts/update-ast-index.sh` calls `ast-index update` once, full stop.

No outstanding architect/product-owner blockers.
