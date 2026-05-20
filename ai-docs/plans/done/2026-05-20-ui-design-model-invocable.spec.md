# Enable model-side auto-invocation for /ui-design skill

**Source:** issue #510
**Date:** 2026-05-20
**Tracked in:** #510

Make `.claude/skills/ui-design/SKILL.md` model-invocable so Claude can auto-load the design-system context whenever it begins visual work, instead of relying on the user to type `/ui-design`. Today the skill is gated behind `disable-model-invocation: true` — it loads only on explicit slash invocation. After this change, both invocation paths (user slash + model auto-pick) must coexist.

## Scope

1. Remove the `disable-model-invocation: true` line from `.claude/skills/ui-design/SKILL.md` YAML frontmatter (or set it to `false` — equivalent effect; pick whichever matches existing tree convention for explicitly-model-invocable skills).
2. Rewrite the SKILL.md `description:` field so the model can identify the trigger conditions without further prompting. The new description embeds the explicit trigger keywords called out in the issue body: `paint`, `painter`, `Style impl`, `DefaultStyle`, `Palette`, `ColorRole`, `snapshot`, `widget view`, `quartzite-style`, `quartzite-widgets`, `quartzite-paint-api`.
3. Add the 5th trigger condition (`quartzite-paint-api`: painter primitives, brush, pen, path, font, color) to the body of `.claude/skills/ui-design/SKILL.md` so the skill body and the issue's "trigger conditions" list match. (Currently the body enumerates only 4 triggers; the issue body lists 5.)
4. Propagate the same 5th trigger to AGENTS.md § *Design system* — that section is the canonical workspace-level trigger list and must not diverge from the SKILL.md body.
5. Verify that, after the change, the slash-invocable path (`/ui-design`) still resolves to the same skill and loads the same body. This is purely behavioural — no code-side wiring exists to break.

## Out of scope

- **No change** to `design-system/SKILL.md` (`name: quartzite-design`) at the repo root. It remains the canonical pointer-only manifest, non-slash-discoverable.
- **No change** to the body of design-system content (`design-system/README.md`, `design-system/colors_and_type.css`, `design-system/preview/`, `design-system/ui_kits/widgets/`).
- **No change** to other skills' `disable-model-invocation` values.
- **No change** to `.claude/agents/design.md` / `.claude/agents/design-review.md` Read-before-designing bullets (they already point at `design-system/SKILL.md` + `README.md` directly; they do not depend on `/ui-design` being model-invocable).
- **No change** to `CLAUDE.md` § *Design system* — its trigger list is the same content as AGENTS.md (CLAUDE.md `@AGENTS.md`-imports AGENTS.md), so the AGENTS.md edit propagates implicitly.

## Deferred

- (none)

## Key decisions

| Question | Decision |
|---|---|
| Drop `disable-model-invocation: true` entirely or set to `false`? | Defer the literal choice to design — both are equivalent semantically; pick the form matching closest in-tree precedent. `pr-ci-failed/SKILL.md` and `master-ci-failed/SKILL.md` use the explicit `disable-model-invocation: false`; design may follow that precedent for parity, or drop the line entirely (Claude Code default is `false` when omitted). |
| Should the new `quartzite-paint-api` trigger condition also be added to AGENTS.md § Design system and the SKILL.md body, or only encoded in the SKILL.md `description:` field? | **All three surfaces** (round-1 answer): add to AGENTS.md § *Design system* bullet list, to the SKILL.md body trigger list, AND embed the keyword in the SKILL.md `description:` field. The three surfaces stay in sync — workspace-level trigger list (AGENTS.md) ↔ skill-level trigger list (SKILL.md body) ↔ model-facing keyword surface (SKILL.md `description:`). |
| Sync-group surface: do any other instruction files need to update? | AGENTS.md § *Agent Docs* row for `.claude/skills/ui-design/SKILL.md` carries a one-line label (`/ui-design skill — pointer to design-system/ …`). It does not name the invocation mode, so no edit needed there. `ai-docs/agent-docs-index.md` similarly carries a row without invocation-mode wording. Confirmed by grep at spec time. |
| `.claude/skills/ai-audit/reference.md` Checklist D rule (`disable-model-invocation: true ↔ skill is user-only — verify intent matches`) | Unchanged. The rule still applies — after this change the skill's intent is "model-invocable", and the rule simply now expects either `false` or an absent key. No edit needed. |

## Technical constraints

- The SKILL.md `description:` field is the primary signal Claude uses to decide whether to auto-load the skill. Per Claude Code docs, the description should make trigger conditions clear. Embed the explicit keywords listed in the issue body so the trigger surface is unambiguous to the model.
- The skill remains pointer-only — it must continue to point at `design-system/SKILL.md` + `design-system/README.md` rather than inlining visual rules. Auto-invocation must not turn it into a context-budget consumer on non-visual sessions; the body's Read instructions stay on-demand.
- AGENTS.md is on a soft 35,000-char early warning before the 40,000-char hard harness cap. Any AGENTS.md edit must measure size delta. Adding one bullet to § *Design system* is well below the threshold but the size check still applies.
- The Propagation Rule (AGENTS.md § *Propagation Rule*) does NOT list `.claude/skills/ui-design/SKILL.md` in any sync group — confirmed by grep. The sync surface for this change is informal (AGENTS.md § *Design system* trigger list ↔ SKILL.md body trigger list).
- No code, no tests, no CI changes. This is a Claude Code instruction-file edit.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/skills/ui-design/SKILL.md` YAML frontmatter does NOT contain `disable-model-invocation: true`. (Either the line is removed or set to `false`.) Verifiable via `grep -E '^disable-model-invocation:\s*true$' .claude/skills/ui-design/SKILL.md` returning zero hits. |
| AC2 | `.claude/skills/ui-design/SKILL.md` `description:` field contains all of these substrings (case-insensitive, single field): `paint`, `Style impl` (or equivalent phrasing naming Style implementations), `DefaultStyle`, `Palette`, `ColorRole`, `snapshot`, `quartzite-style`, `quartzite-widgets`, `quartzite-paint-api`. Verifiable via per-keyword `grep -i` over the `description:` line. |
| AC3 | The SKILL.md body (the prose after the frontmatter) lists 5 trigger conditions including the new `quartzite-paint-api` trigger (painter primitives, brush, pen, path, font, color). |
| AC4 | AGENTS.md § *Design system* lists the same 5 trigger conditions as the SKILL.md body. Verifiable by visual diff against AC3's list. |
| AC5 | Running `/ui-design` still loads the same skill body as before the change (slash invocation path unbroken). Verifiable by user-side smoke test or by inspection: the skill directory `.claude/skills/ui-design/` still exists with `SKILL.md` inside and the frontmatter `name:` field still equals `ui-design`. |
| AC6 | No other instruction file gains or loses a `disable-model-invocation` line. Verifiable via `grep -rn 'disable-model-invocation' .claude/skills/ | diff` against the pre-change baseline (only the single `ui-design/SKILL.md` line should change). |
| AC7 | AGENTS.md total size (`wc -c AGENTS.md`) remains below 40,000 chars (hard cap) and the size delta from this edit is reported in the implementation commit message. |
| AC8 | `.claude/skills/ai-audit/reference.md` Checklist D wording is left untouched — its rule still describes the contract correctly after the change. |

## Open questions

- The exact wording of the new `description:` field (which is design-authored, not spec-authored). The spec fixes only the set of keywords that must appear (AC2); design chooses the prose.
- The exact wording of the new `quartzite-paint-api` trigger bullet for AGENTS.md and the SKILL.md body. Design-authored, constrained by AC3 + AC4 to match each other.
