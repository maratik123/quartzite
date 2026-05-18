# Rename `design` skill to `ui-design`

**Source:** issue #470
**Date:** 2026-05-18
**Tracked in:** #470

## Scope

1. Rename the slash-discoverable skill directory `.claude/skills/design/` → `.claude/skills/ui-design/`.
2. Update the SKILL.md frontmatter `name: design` → `name: ui-design` inside the renamed file (skill loader keys off both the directory slug and the `name:` field — they must match per the convention established by every other `.claude/skills/<slug>/SKILL.md` in-tree).
3. Update the SKILL.md description and body text where it refers to itself or to its purpose, so the slash-command surface (`/ui-design`) reads naturally and distinguishes itself from the unrelated `design` agent at `.claude/agents/design.md`.
4. Update **all** active operational references to the old path:
   - `AGENTS.md` § Agent Docs table row — change `.claude/skills/design/SKILL.md` and the `/design` skill prose label to `/ui-design`.
   - `ai-docs/deferred/_inbox.md` row mentioning `.claude/skills/design/SKILL.md` — update path string in the affected cell.
5. Update **historical** references to the old path so the project corpus is internally consistent after rename (user-confirmed in Round 1 Q1: sweep all). The set of files to update:
   - `ai-docs/plans/INDEX.md` — historical row for `wire-design-system-context` (path `.claude/skills/design/SKILL.md` inside the table cell).
   - `ai-docs/context.md` § Plans list — `wire-design-system-context` bullet mentioning `.claude/skills/design/SKILL.md`.
   - `ROADMAP.md` — `wire-design-system-context` row that names `.claude/skills/design/SKILL.md` in its files-touched column.
   - `ai-docs/plans/done/2026-05-18-wire-design-system-context.spec.md` — every textual occurrence of `.claude/skills/design/` (path) and `design` (the skill-`name` value, in §§ where the value is quoted).
   - `ai-docs/plans/done/2026-05-18-wire-design-system-context.design.md` — same.
   - Any other file the AC4 grep flags after the edits above land.
6. Verify the sweep is complete via the AC4 grep — exactly zero hits across the entire project tree except for the deliberate carve-outs (this rename's own spec + design, and `ai-docs/learnings.md` per § Out of scope below).

## Out of scope

- Renaming the `design` **agent** at `.claude/agents/design.md` (`name: design`). The issue explicitly contrasts the skill against the existing agent and asks only the skill to move.
- Renaming the canonical design-system manifest skill at `design-system/SKILL.md` (`name: quartzite-design`, `user-invocable: true`). That skill is the Read-target of the renamed pointer; it keeps its name.
- Editing `ai-docs/learnings.md`. AGENTS.md § *Corrections Log* Boundary rule 1 is **APPEND-ONLY** — existing entries are never edited, even to correct stale path references. If any past learning entry quotes `.claude/skills/design/`, it stays as-is (faithful historical record). The AC4 grep explicitly excludes `ai-docs/learnings.md` from the zero-hit set.
- Rewriting the path-form occurrences of `.claude/skills/design/` inside **this rename's own spec and design files** (`ai-docs/plans/2026-05-18-rename-design-skill-to-ui-design.spec.md` and `ai-docs/plans/2026-05-18-rename-design-skill-to-ui-design.design.md`). These artefacts are tautologically the canonical place the old path is described — they record the from→to of the rename itself (e.g. Scope §1 reads `.claude/skills/design/` → `.claude/skills/ui-design/`), include AC1's verification command quoting the old path, and document risk reasoning that depends on the literal old-path string. Mechanical substitution would corrupt their narrative integrity (`ui-design/ → ui-design/` becomes nonsense; AC1 verification becomes self-contradictory). Carved out of the AC4 zero-hit set on the same principle that carves out `learnings.md` — the file documents a historical state and must continue to.

## Deferred

(none)

## Key decisions

| Question | Decision |
|---|---|
| Rename the skill directory, the `name:` field, or both? | Both — Claude Code's skill loader uses the directory slug AND the frontmatter `name:` field; every in-tree `.claude/skills/<slug>/SKILL.md` matches them and the new file must follow that convention. |
| New slug? | `ui-design` (matches the issue title verbatim). |
| Update completed-task artefacts (`done/`, `INDEX.md` history row, `context.md` § Plans list, `ROADMAP.md`)? | **Yes — sweep all** (user, Round 1 Q1). Rewrite every textual occurrence of `.claude/skills/design/` (and the `name: design` value where it appears in body prose) across active and historical surfaces. AC4 grep returns zero hits afterwards (excepting the explicit carve-outs). |
| `ai-docs/learnings.md` exclusion | AGENTS.md § Corrections Log Boundary rule 1 makes `learnings.md` append-only; existing entries are never edited even when they reference a now-renamed path. Carved out of the AC4 zero-hit set. |
| This rename's own spec + design exclusion | The rename's own `.spec.md` and `.design.md` describe the from→to of the rename itself (Scope §1: `.claude/skills/design/` → `.claude/skills/ui-design/`), and AC1's verification command quotes the old path literally. Mechanical substitution destroys narrative integrity. Carved out of AC4 on the same "file documents a historical state" principle as `learnings.md`. |
| Rename the `design` agent at `.claude/agents/design.md`? | **No.** Issue body explicitly notes the agent exists and works on another task; the rename is one-sided. |
| Aliasing the old slug or keeping a stub `.claude/skills/design/SKILL.md` that points to the new path? | Not done. AGENTS.md § API Stability axiom (clean rename, no aliases) applies; the old slug disappears entirely. |
| `_inbox.md` cell-text update vs. AGENTS.md "written ONLY by `/task` Step 12 and `/triage`" AXIOM? | Treated as a **path-string fix inside an existing row** (no row added, removed, moved, or re-classified); the AXIOM targets row-level structural edits. Implementation guidance: edit only the cell text containing the stale path; do not touch row count / column count / row ordering. |

## Technical constraints

- The renamed file MUST keep the same pointer-only contract documented in the existing SKILL.md: frontmatter `disable-model-invocation: true`, `allowed-tools: Read`, body Reads `design-system/SKILL.md` then `design-system/README.md` then explores `design-system/preview/` and `design-system/ui_kits/widgets/` as needed. No inlined visual rules.
- The rename is mechanical: one directory + one frontmatter field + active-surface text references + historical-surface text references. Test surface: zero new Rust tests; verification is a single `grep` invocation (see AC4).
- AGENTS.md is currently ~36.5 kB (past the 35 000-char proactive-extraction warning). The rename swaps `design` → `ui-design` (+3 bytes per occurrence × ~2 occurrences in AGENTS.md) — well under any threshold concern. No extraction required.
- The Propagation Rule (AGENTS.md) does not list `.claude/skills/design/SKILL.md` as having sync-group siblings, so no group-wide co-edit is mandated. The verification grep in AC4 covers stragglers.
- Historical-surface edits to `ai-docs/plans/done/*.{spec,design}.md` are deliberate per Round 1 Q1; this differs from the general `done/` immutability convention. The user-confirmed sweep is the sole authority for these edits — design phase should NOT auto-extend the sweep to other history (e.g. arbitrary `done/*` files that don't mention the path).
- `ai-docs/learnings.md` and the rename's own spec + design files are excluded from the sweep per § Out of scope; AC4 grep filters all three.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Directory `.claude/skills/ui-design/` exists with file `SKILL.md` inside; directory `.claude/skills/design/` no longer exists. Verify: `test -d .claude/skills/ui-design && test ! -e .claude/skills/design`. |
| AC2 | `.claude/skills/ui-design/SKILL.md` frontmatter contains `name: ui-design` (not `name: design`). Verify: `grep -E '^name: ui-design$' .claude/skills/ui-design/SKILL.md`. |
| AC3 | AGENTS.md § Agent Docs table contains a row for `.claude/skills/ui-design/SKILL.md` (not the old path) and references the skill as `/ui-design`. Verify: `grep -n '/ui-design' AGENTS.md` returns the Agent Docs row; `grep -n '\.claude/skills/design/SKILL\.md' AGENTS.md` returns zero hits. |
| AC4 | Mechanical sweep over the project returns no stale **path-form** references anywhere except the explicit carve-outs: `learnings.md` (append-only) and the rename's own spec + design (tautological from→to record). Precise verification command: `grep -rn '\.claude/skills/design[/ ]' --include='*.md' . \| grep -v -e '^ai-docs/learnings\.md:' -e '^ai-docs/plans/2026-05-18-rename-design-skill-to-ui-design\.spec\.md:' -e '^ai-docs/plans/2026-05-18-rename-design-skill-to-ui-design\.design\.md:'` returns empty. Anchors omit a `./` prefix because GNU `grep -rn` with a bare `.` search-root emits filenames without one. The carve-outs are intentional per § Out of scope. |
| AC4b | Mechanical sweep over **every swept artefact** (active + historical) returns no stale **backtick-quoted name-literal** references. Precise verification command: ``grep -nE '`name: design`' ai-docs/plans/INDEX.md ai-docs/context.md ROADMAP.md ai-docs/plans/done/2026-05-18-wire-design-system-context.spec.md ai-docs/plans/done/2026-05-18-wire-design-system-context.design.md`` returns empty. The five files listed are exactly the active + historical sweep targets per § Scope §§4–5 (excluding AGENTS.md and `_inbox.md`, neither of which contains the `` `name: design` `` literal). The carved-out files — `learnings.md`, the rename's own spec and design — are exempt from this AC for the same reason as AC4: they tautologically document the from→to of the rename and live elsewhere in the file tree. |
| AC5 | Slash-command surface: invoking the skill by its new slug loads the pointer body (smoke check at review time). The body remains pointer-only — no `design-system/` visual rules inlined. |
| AC6 | The retired `ai-docs/plans/2026-05-18-rename-design-skill-to-ui-design.spec.md.state.md` file is removed by `/interview` terminal cleanup at end of round (housekeeping; not part of the rename diff). |

## Open questions

(none)
