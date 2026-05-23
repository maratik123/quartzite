# Triage-runner — Design-link outcomes (Phase 8 sub-section)

Extracted from .claude/agents/triage-runner.md § Phase 8 — Run-output summary. Documents the Phase 8 run-summary shape for the UI-design classification gate.

**Sub-section header.** `## Design-link outcomes` — placed in the Phase 8 run summary AFTER the Bridge sub-section and BEFORE the "Issues created" list.

**Per-row outcomes.** One bullet per row that entered the gate this run, in collection order (Phase 6 sweep approvals first, then Phase 7 drain promotes). Exactly one of:

- `#<C> → #<N> (existing)` — numbered-pick branch resolved to an open ui-design umbrella that already existed at Phase 4's bulk-list snapshot.
- `#<C> → #<N> (new)` — `new`-branch inline-created umbrella; `#<N>` is the umbrella's number returned by Phase 7.5's umbrella-first sub-pass.
- `#<C> (skip-link)` — `none`-branch row: child created without `blocked` / `ui-design` labels and without a `**Blocked by:**` body line. The "design-work issue without umbrella link" status is captured here.
- `<row> deferred` — `defer`-branch row: no `gh issue create` ran; the row remained in `_inbox.md` (or its Phase 6 sweep approval was downgraded to "deferred (gate)" with source cell untouched). `<row>` identifies the row by `file + cell` (e.g. `ai-docs/deferred/widgets.md cell 4`) so the maintainer can re-promote on a future run.

**Per-umbrella body-edit summary.** One bullet per umbrella the run touched (mutated OR widened), in `#N` ascending order. Shape:

```
#<N>: +<count> child(ren) appended
```

where `<count>` is the number of children appended under the umbrella's `## Child issues (blocked on this epic)` anchor THIS run (idempotency no-ops do NOT count). Umbrellas with `<count> == 0` (because every child was a no-op via the `#<C> ` sentinel) are still listed for resume-visibility.

**`Body-edit skipped — anchor absent` sub-list.** Per-umbrella **structural** state — fires when sub-step 4a of the numbered-pick / `new` branch finds no `## Child issues (blocked on this epic)` substring in the umbrella body. Same shape on every run until the umbrella body is hand-edited. One bullet per umbrella:

```
#<N> — <umbrella-title> — manual update needed: add child checklist anchor
```

**`Body-edit failed — gh API error` sub-list.** Per-run **transient** state — fires when sub-step 4e (`gh issue edit <N> --body-file <tmpfile>`) returns non-zero (network error, rate limit, auth expiry, etc.); parse + idempotency + compose succeeded, only push-back failed. **Separate** from anchor-absent because the recovery path differs (re-run heals it via the unsatisfied `#<C> ` sentinel; the child already carries `**Blocked by:** #N`). One bullet per (umbrella, child) pair that failed:

```
#<N> ← #<C> — body edit failed: <gh stderr first-line>
```

**`/next` SKILL.md propagation-grep result line per mutated umbrella.** AC10 contract: every umbrella that was created (`new` branch) OR widened (numbered-pick branch) this run gets one line recording the result of `grep -n "#<N>" .claude/skills/next/SKILL.md` against that umbrella's `#N`. Shape, one bullet per mutated umbrella:

```
#<N>: grep -n "#<N>" .claude/skills/next/SKILL.md → <hit-count> match(es) <file:line list OR "(no matches)">
```

The grep is mandatory (executed by `triage-runner` in the same `/triage` run as the umbrella mutation); the optional follow-up edit to the *UI-designer label* mini-section OR the `## Candidates for UI-designer handoff` output bullet of `next/SKILL.md` is at the maintainer's discretion and, if applied, recorded as a separate "next/SKILL.md edited: <description>" bullet immediately below the grep line for the same umbrella.
