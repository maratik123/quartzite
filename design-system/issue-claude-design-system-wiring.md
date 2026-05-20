# Wire Quartzite Design System into Claude Code agent context

## Motivation

The `design-system/` folder (added on the `design-system` branch) is the single source of truth for Quartzite's visual vocabulary — palette roles, type, paint rules, widget mocks. Today none of the Claude Code agents read it. As a result:

- `/task`-spawned `design` and `design-review` agents formulate UI-touching plans against `AGENTS.md` and source only, with no awareness of the documented visual rules (1 px outline, no radius, `pressed > checked > hovered > idle` precedence, `Button.blend(Highlight, 0.25)` hover, ×0.5 α disabled, additive 2 px focus outline, the dark-palette seeds, etc.).
- Top-level sessions don't know the folder exists unless the human mentions it.
- The skill at `design-system/SKILL.md` is marked `user-invocable: true` but Claude Code's loader only discovers skills under `.claude/skills/<slug>/SKILL.md`, so it isn't actually reachable as a slash command.

## Constraint: context budget

`CLAUDE.md`'s `@<file>` syntax loads every referenced file **unconditionally** into every session. `AGENTS.md` is already ~35 KB and is auto-imported. Adding `@design-system/SKILL.md` would (a) load 2 KB on every session whether or not the task is visual, and (b) instruct the agent to cascade-load `design-system/README.md` (19 KB) — pushing the auto-imported total past 55 KB on tasks that don't need any of it.

**Therefore the wiring must be conditional, not blanket-imported.** The agents that need the context (`design`, `design-review`, and any future visual-work sub-agent) read it on demand; other agents never load it.

## Goal

Make the design system discoverable and default-on **for visual work**, with zero impact on the auto-imported context size for unrelated tasks.

## Acceptance Criteria

1. **`AGENTS.md` gains a short "Design system" pointer section** (one paragraph, ≤ 15 lines), describing when to consult `design-system/`:
   - When working on `quartzite-style` (any `Style` impl, including `DefaultStyle`)
   - When working on `quartzite-widgets` paint paths, widget views, or any user-facing rendering
   - When changing `Palette` / `ColorRole` semantics or seeds
   - When adding or modifying snapshot tests under `quartzite-style/tests/snapshots/`

   The section names the two entry points (`design-system/SKILL.md` for the manifest, `design-system/README.md` for the visual rules) and the supporting subfolders (`preview/`, `colors_and_type.css`, `ui_kits/widgets/`), but does **not** quote their contents. Total added bytes must stay under ~1 KB so the section is a pointer, not a duplicate.

   Verifiable: `wc -c AGENTS.md` shows a delta of ≤ 1 024 bytes; `grep -n 'design-system' AGENTS.md` returns the new section.

2. **`CLAUDE.md` is NOT changed.** No new `@<file>` import. The auto-imported context size stays where it is today (`AGENTS.md` only). Verifiable: `git diff CLAUDE.md` is empty.

3. **`.claude/agents/design.md`** lists the design system under **Read before designing**. The new bullet names both `design-system/SKILL.md` and `design-system/README.md` and qualifies *when* it applies (tasks touching `quartzite-widgets` paint logic, `quartzite-style`, or any user-facing surface). The bullet is intentionally a "read this only when the task is visual" hint, not a blanket load. Verifiable: from a fresh `design` agent context, asking *"What is the hover fill for `Button`?"* on a UI-touching task returns `Button.blend(Highlight, 0.25)` / `#BFDFFF` without priming.

4. **`.claude/agents/design-review.md`** extends **Read context** (Step 2 of its Workflow) to require `design-system/README.md` § VISUAL FOUNDATIONS + `design-system/colors_and_type.css` whenever the design under review touches a widget, paint code, or `Palette` / `ColorRole`. Deviations from documented rules (outline width, radius, derivation formulas, focus overlay) are flagged as `major` blockers in the verdict table — same severity bar as the existing handoff-grouping checks.

5. **A pointer skill at `.claude/skills/design/SKILL.md`** makes the design system invocable as a slash command for **explicit** on-demand load. Contents are minimal — frontmatter (`name: design`, short description) plus a body that instructs the agent to read `design-system/SKILL.md`, then `design-system/README.md`, then explore `design-system/preview/` and `design-system/ui_kits/widgets/` as needed. The canonical content stays in `design-system/`; this file is a 10-line pointer, not a duplicate.

6. **No content duplication.** The 19 KB `design-system/README.md` is not inlined into `AGENTS.md`, `CLAUDE.md`, or any agent prompt. Each touchpoint references the design-system folder by path.

7. **Sanity check from a fresh session — non-visual task.** Asking the top-level agent *"What's the next milestone in the roadmap?"* does not cause it to load `design-system/`. Verifiable behaviourally: the agent's Read tool calls do not include any path under `design-system/`.

8. **Sanity check from a fresh session — visual task.** Asking the top-level agent *"I want to add a new `Slider` widget — what's the visual contract I need to match?"* causes it to (a) reach for `AGENTS.md`'s Design system pointer, then (b) Read `design-system/SKILL.md` / `README.md`. The eventual answer cites the 1 px outline + no-radius + `Highlight` for pressed/checked + ×0.5 α disabled + additive 2 px focus rules.

## Files touched (expected)

- `AGENTS.md` — one added section (≤ 1 KB)
- `.claude/agents/design.md` — one added bullet in **Read before designing**
- `.claude/agents/design-review.md` — one added bullet in **Workflow** step 2
- `.claude/skills/design/SKILL.md` — new file (~10 lines)

## Out of scope

- Editing `CLAUDE.md` (rejected — auto-import is the wrong tool for conditional context).
- Editing `design-system/` contents themselves (the source of truth is already correct).
- Modifying other `.claude/agents/*` (e.g. `spec-writer`, `triage-runner`) — those don't currently make visual decisions; if a future task surfaces a gap there, file a follow-up.
- Per-PR review automation that checks visual-rule deviations programmatically. Could be a follow-up issue once point 4 has been exercised in practice.

## Non-goals / FORBIDDEN

- Do NOT add `@design-system/SKILL.md` or `@design-system/README.md` to `CLAUDE.md`. The `@<file>` syntax loads unconditionally — that defeats the purpose of conditional design context.
- Do NOT inline the visual rules into `AGENTS.md`. The pointer section names paths; agents Read the paths when relevant.
- Do NOT change the YAML frontmatter of `design-system/SKILL.md` (the `user-invocable: true` flag stays; the new `.claude/skills/design/SKILL.md` is a separate, complementary file).

## Verification plan

| # | Criterion | Verification |
|---|---|---|
| 1 | `AGENTS.md` pointer section added | `grep -n 'design-system' AGENTS.md` returns the new section; `git diff --stat AGENTS.md` shows ≤ 1 024 bytes added |
| 2 | `CLAUDE.md` unchanged | `git diff CLAUDE.md` is empty |
| 3 | `design` agent reads design-system on UI tasks | Fresh agent prompt on a visual task: *"hover fill for Button?"* → cites `Button.blend(Highlight, 0.25)` |
| 4 | `design-review` flags deviations | Synthetic design doc proposing 4 px radius on Button → review verdict includes a `major` row citing § VISUAL FOUNDATIONS |
| 5 | `/skill design` invocable | Slash-command dispatcher loads the new skill file and returns its body |
| 6 | No duplication | `wc -c AGENTS.md CLAUDE.md` — only `AGENTS.md` grows, and by ≤ 1 KB; no `design-system/README.md` text appears inside `.claude/` |
| 7 | Non-visual task doesn't load design-system | Behavioural: a roadmap question doesn't trigger any Read under `design-system/` |
| 8 | Visual task does load it | Behavioural: a new-widget question triggers Read of `design-system/SKILL.md` and `design-system/README.md` |

## Notes for `/interview` / `/task`

This issue is intentionally framed as small, scoped wiring work — no Rust code, no test additions. Expected design decomposition is 4 subtasks (one per file: `AGENTS.md`, `design.md`, `design-review.md`, new `.claude/skills/design/SKILL.md`). 4 in one terminal group violates the `1..=3` handoff rule per the existing design-review rubric (`major` severity per `.claude/agents/design.md` § Rules → handoff-grouping). Easiest fix in the design phase is `3 + 1`: Group A = `AGENTS.md`, `design.md`, `design-review.md`; Group B (terminal) = the new `.claude/skills/design/SKILL.md`. Alternatives the `design` agent should consider and document the rationale for:

- **3 + 1 split** (recommended) — clean Group A around editing existing agent surfaces, Group B introduces the new file.
- **Merge the two agent edits into one subtask**, yielding 3 terminal subtasks (`AGENTS.md`, agents-as-one, `.claude/skills/design/SKILL.md`). Slightly less atomic but stays under the cap with no handoff.
- **2 + 2** — not allowed under the current rules; flagged here so the design agent doesn't accidentally propose it.
