# Allow `ast-index` in permissions + keep index fresh on commit

**Source:** issue #507
**Date:** 2026-05-20
**Tracked in:** #507

## Scope

1. Add an `ast-index` wildcard allow-list entry to `.claude/settings.json` (project-shared file) so agents stop prompting for permission when invoking the `ast-index` CLI.
2. **Migrate** the two broad search-tool wildcards from `.claude/settings.local.json` to `.claude/settings.json`:
   - `Bash(grep *)` (currently `.claude/settings.local.json:32`) → `.claude/settings.json` `permissions.allow`.
   - `Bash(rg *)` (currently `.claude/settings.local.json:341`) → `.claude/settings.json` `permissions.allow`.
   After the migration all three search-tool wildcards (`grep`, `rg`, `ast-index`) live together in the project-shared `.claude/settings.json`.
3. Consolidate the redundant specific-argument `Bash(rg "...")` / `Bash(grep ...)` entries in `.claude/settings.local.json` — currently 33 specific `rg` entries (lines 56, 59, 61, 69, 70, 76, 77, 79, 80, 81, 86–90, 114, 133, 134, 152–158, 199, 253, 334–336, 339, 340) and 6 specific `grep` entries (lines 25, 27, 384–386). After this PR the file carries 0 specific `rg` / `grep` entries — all are covered by the migrated wildcards in `.claude/settings.json`.
4. Add `scripts/update-ast-index.sh` — a thin wrapper that:
   - No-ops (`exit 0`) on hosts where `ast-index` is not on `$PATH`.
   - Otherwise runs `ast-index update` (incremental); falls back to `ast-index rebuild` if the DB is missing.
   - Prints stderr only on failure; **never blocks the commit** (always `exit 0`).
5. Wire `scripts/update-ast-index.sh` into a `PreToolUse` hook in `.claude/settings.json` (`matcher: "Bash"`, fires on `git commit`), mirroring the shape of the existing `scripts/gen-roadmap.sh` hook (`.claude/settings.json:27`) — but **non-blocking** (hook command always returns 0, regardless of script outcome).

## Out of scope

- Sharing the ast-index DB via git or any other transport — DB is host-local by design.
- A long-lived `ast-index watch` daemon as the primary sync mechanism (issue explicitly rejects this).
- Changes to `ast-index` itself or to the `ast-index:ast-index` skill.
- Touching `.claude/settings.local.json` entries that are NOT specific-argument `rg` / `grep` invocations (other tools like `python3 -c`, `curl`, `xargs`, etc. remain as they are — out of scope for this PR even though they are similarly redundant).

## Deferred

- (none)

## Key decisions

| Question | Decision |
|---|---|
| Where does the new `ast-index` allow-entry live? | `.claude/settings.json` (project-shared, checked-in — visible to every contributor; matches where the hook entry lives). Source: Round 1 Q1. |
| Where do the existing `grep` / `rg` wildcards live after this PR? | Migrated to `.claude/settings.json` alongside `Bash(ast-index *)`. All three search-tool wildcards in one project-shared file → single mental model, single source of truth for "what search tools any contributor can use without a prompt". Source: Round 2 Q1. |
| Where does the hook entry live? | `.claude/settings.json` — host-portable thanks to the `command -v ast-index` no-op guard, and the existing `gen-roadmap.sh` hook precedent is the project-shared file. |
| Hook blocking semantics? | **Non-blocking**: hook command always exits 0; on script failure, prints to stderr only. Contrast with `gen-roadmap.sh` which `exit 2`s because `ROADMAP.md` is checked in. The ast-index DB is host-local + gitignored, so a failed update must not break the commit. |
| Hook matcher / trigger? | `PreToolUse` on `matcher: "Bash"`, with the same `git commit` substring guard used by the `gen-roadmap.sh` hook in `.claude/settings.json:27`. |
| Script POSIX-portability constraint? | Same baseline as `scripts/gen-roadmap.sh` (POSIX `sh` / `set -eu`, no bash-isms). The script body is short enough that the constraint is trivial to honour. |
| Consolidation scope? | **In this PR** (Round 1 Q2). Remove the 33 redundant specific-argument `Bash(rg "...")` entries + 6 redundant `Bash(grep ...)` entries from `.claude/settings.local.json`. The migrated wildcards `Bash(grep *)` and `Bash(rg *)` (now in `.claude/settings.json`) cover them all; removing the specifics is a no-op for permission semantics, a positive change for readability. |
| Allow-list syntax form for the new `ast-index` entry? | **Space-form `Bash(ast-index *)`** to match the existing `Bash(grep *)` / `Bash(rg *)` / `Bash(cargo *)` / `Bash(git *)` / `Bash(gh *)` precedent across both settings files. The lone `Bash(echo:*)` colon-form at `.claude/settings.local.json:567` is an outlier; the design phase confirms the canonical schema-supported form before commit, with colon-form as documented fallback. |

## Technical constraints

- **Mirror the `gen-roadmap.sh` hook shape** for the PreToolUse entry, with the blocking-semantics inversion. Reference: `.claude/settings.json:27`.
- **Host-local DB**: never `git add` / commit the ast-index database, never reference it from CI.
- **No-op on absent binary**: `command -v ast-index >/dev/null 2>&1 || exit 0` as the first effective line of `scripts/update-ast-index.sh`.
- **Hook command exit code**: regardless of script exit, the hook command itself returns 0 — so even a future bug in `scripts/update-ast-index.sh` cannot block a commit. (Belt-and-suspenders with the script's own `exit 0` guarantee.)
- **Script lives at `scripts/update-ast-index.sh`** (path fixed by issue body) and is executable (`chmod +x`).
- **Permissions-schema verification before commit**: design phase confirms `Bash(ast-index *)` (space-form) is accepted by Claude Code's current permissions schema. Issue Notes explicitly flags this as a design-time check; if the schema rejects space-form, fall back to colon-form (`Bash(ast-index:*)`) and apply the same form consistently to all three search-tool wildcards in `.claude/settings.json`.
- **JSON well-formedness**: both `.claude/settings.json` and `.claude/settings.local.json` must remain valid JSON after every change in this PR (no trailing commas after the last array element when a removal happens to be the last entry).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/settings.json` `permissions.allow` contains all three search-tool wildcards: `Bash(ast-index *)`, `Bash(grep *)`, `Bash(rg *)` (or the colon-form variants of all three, applied consistently). |
| AC2 | (host-local — `.claude/settings.local.json` is gitignored per `.gitignore:3:/.claude/settings.local.json`; cleanup is per-host hygiene, not propagated via git. Project-shared wildcards in `.claude/settings.json` are what propagate cross-host.) `.claude/settings.local.json` no longer contains the standalone `Bash(grep *)` (was line 32) and `Bash(rg *)` (was line 341) entries — they have been moved, not duplicated. |
| AC3 | (host-local — gitignored as above.) `.claude/settings.local.json` no longer contains the 33 specific-argument `Bash(rg "...")` entries nor the 6 specific-argument `Bash(grep ...)` entries enumerated in Scope §3. After the cleanup, `grep -cE 'Bash\((rg\|grep) ' .claude/settings.local.json` returns `0`. |
| AC4 | `scripts/update-ast-index.sh` exists, is executable (`chmod +x`), is POSIX-portable (same shellcheck-clean baseline as `gen-roadmap.sh`), and is a no-op (`exit 0`, no other output) on hosts where `command -v ast-index` is empty. |
| AC5 | On hosts where `ast-index` is installed, `scripts/update-ast-index.sh` runs `ast-index update` (incremental), and falls back to `ast-index rebuild` when the database is missing (detected by `ast-index update` exit code or `ast-index stats` probe — design phase picks the mechanic). |
| AC6 | `scripts/update-ast-index.sh` exits 0 on every code path, including on `ast-index` failure; failure messages go to stderr only. |
| AC7 | A `PreToolUse` hook entry in `.claude/settings.json` invokes `scripts/update-ast-index.sh` on `git commit` commands, using the same matcher shape as the existing `gen-roadmap.sh` hook (`.claude/settings.json:27`). The hook command itself returns 0 on every code path — i.e., the hook does **not** propagate a non-zero script exit to Claude Code. |
| AC8 | Spot-check on the developer host (issue #507 author): after editing a `.rs` file and running `git commit`, `ast-index stats` reflects the change without a separate manual `ast-index update`. |
| AC9 | Spot-check: invoking `ast-index symbol foo` (or any `ast-index <subcommand> ...`) in an agent turn no longer triggers a permission prompt. |
| AC10 | After all changes, `python3 -m json.tool < .claude/settings.json` and `python3 -m json.tool < .claude/settings.local.json` both succeed (JSON well-formed). |

## Open questions

- (Non-blocking, design-only) Confirm `Bash(ast-index *)` space-form is accepted by the current Claude Code permissions schema before commit. The issue Notes flags this as a verification step, not a spec question. Fallback to the colon-form `Bash(ast-index:*)` if the schema rejects space-form; if the fallback fires, apply the same form to `grep` and `rg` wildcards too for consistency.
- (Non-blocking, design-only) Whether `scripts/update-ast-index.sh` should additionally prune stale entries for files removed since the last `update` — current `ast-index update` semantics already handle this; flagged here only to confirm during design.
