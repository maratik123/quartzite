# Design: Frontmatter on `learnings.md` (v1, token-positive only)

**Issue:** #575
**Date:** 2026-05-26

## Approach

The spec locks the v1 scope to **`ai-docs/learnings.md` only** and the per-entry field floor to **`escalated:` (always) + `kind:` (always) + `superseded_by:` (when present)**. Round-1 design-review (Spec Amendment trigger) added `kind:` to the floor and corrected the baseline measurement to ~225 KB; this round folds those in. The design phase owns five concrete operational decisions:

1. **Backfill mechanism** — one-shot, byte-deterministic, idempotent (AC16, AC17). Decision below: a self-contained Python script (`scripts/backfill-learnings-frontmatter.py`), versioned in the PR, that streams `ai-docs/learnings.md`, emits a fenced ```` ```yaml … ``` ```` block immediately above every `### YYYY-MM-DD —` heading, and is a no-op on a second run. Rejected alternatives: `awk` (multi-line YAML escaping of double-quotes inside quoted strings — e.g. the existing `Superseded by: 2026-05-08 ("...")` shape — is fragile in pure awk); `sed` (same escaping problem plus no easy idempotence check); hand-edit (136 entries → not idempotent, not byte-deterministic). Python `re` + careful quoting matches the YAML-string contract in the spec (Technical constraints) precisely.

2. **Python script preamble + posture** (new in round 2). Existing `scripts/*.sh` use `#!/usr/bin/env bash` + `set -euo pipefail`. The Python equivalent: `#!/usr/bin/env python3` shebang + fail-loud posture (every emitted fenced block re-fed through `yaml.safe_load`; any parse failure raises a `SystemExit(1)` with a diagnostic naming the offending entry's heading; any schema-floor deviation — missing `escalated:` or `kind:` key after emit — also exits 1). Stdlib only (`re`, `sys`, `pathlib`, `yaml`). PyYAML is the one non-stdlib import; the script declares it required and exits 1 with a `pip install PyYAML` hint if `import yaml` fails. CLI: `python3 scripts/backfill-learnings-frontmatter.py [--check]`. With `--check` it runs in dry-run mode and exits 0 iff the file would receive zero changes (used as the AC17 idempotence verifier).

3. **Tuple-extraction invocation cited in consumer agents** (AC10, AC11). Two equally-valid forms; the spec authorises either. Decision below: cite both in the consumer-agent diffs — a **PyYAML form** (canonical, parses every field including the new `kind:`) and an **awk-extract form** (zero-dep fallback that yields the raw fenced-block text for a per-entry PyYAML pass). The PyYAML form is the documented primary; the awk form documents the manual-grep escape hatch when PyYAML isn't on PATH. Both are self-tested by the AC12 self-test on the post-migration file. With `kind:` now in the FM floor, the Carrot pass (`/improve` Step 1b, `/ai-audit` Phase 1 Carrot sweep) can filter entirely in script — no markdown-body regex sidecar needed.

4. **AGENTS.md edit shape + char-cap budget** (AC7, AC8, AC15). Two distinct insertion sites: (a) Boundary rule 1 gains a new `> **Exception — additive YAML preambles.** …` blockquote inserted **between** the existing `Escalated?` / `Superseded by:` subagent-driven exception (line 301) and the named 2026-05-19 carve-out (line 303). (b) the fenced template block under `### Entry format` (line 328 ff.) gains a ```` ```yaml ```` preamble above the `###` heading line + a one-paragraph note. Measured sizes: verbatim Boundary-rule-1 exception block (spec Scope item 5) = **964 chars**; Entry-format note budget ≈ 600 chars; combined net add ≈ **1,575 chars**. Current `wc -c AGENTS.md` = **37,069**; projected post-edit ≈ **38,644 chars** — comfortably below the 40 KB hard cap and below the 39 KB proactive-extraction trigger (no extraction needed).

5. **corrections-log.md sibling sub-section** (AC9). Add a NEW blockquote between the existing 2026-05-19 carve-out (line 19 of `corrections-log.md`) and the `## Boundary rule 2 Exception` heading (line 21). Format mirrors the 2026-05-19 carve-out's `> **<label>.** …` shape, cites the PR opened by this task (tracking issue #575; the PR number is substituted at /task Step 12 when the PR is opened) as the authorising migration, and links back to `AGENTS.md § Learning Log → Boundary rule 1` for the verbatim exception text. Single source of truth for the verbatim exception is AGENTS.md; `corrections-log.md` is the reference doc that documents the precedent.

### Per-entry YAML-block shape (decided here, sourced from spec)

For every `### YYYY-MM-DD —` heading, emit immediately above (no blank line between the closing ```` ``` ```` and the `###`):

```yaml
escalated: "<verbatim-rhs-of-Escalated?-line>"
kind: "<verbatim-rhs-of-Kind:-line>"
superseded_by: "<verbatim-rhs-of-Superseded-by-line>"   # only when entry has the bold-key line
```

- `escalated:` value = the text after `**Escalated?**` (and after the space following the asterisks), whitespace-trimmed at both ends. Confirmed by `grep -c '^\*\*Escalated\?\*\*' learnings.md` = 136 = entry count (every entry already has the line); the default-when-omitted branch (`escalated: "no"`) is wired in for forward-safety.
- `kind:` value = the text after `**Kind:**`, whitespace-trimmed. Confirmed by `grep -c '^\*\*Kind:\*\*' learnings.md` = **22** (114 entries lack the bold-key line and default to `kind: "correction"` per AGENTS.md "defaults to `correction` when omitted"). The default-when-omitted branch IS exercised — every entry currently lacking `**Kind:**` gets `kind: "correction"` in its FM block. This is the **per-entry** Kind (correction | validation), distinct from the file-level `kind: learnings` (file-class discriminator); parsers address them by source location, not key name (spec Technical constraints).
- `superseded_by:` value = the text after `**Superseded by:**`, whitespace-trimmed. Confirmed by `grep -c '^\*\*Superseded by:\*\*' learnings.md` = **9**. Embedded `"` characters are YAML-escaped as `\"`. Verified inventory of the 9 lines (line numbers in pre-migration HEAD):
  - Line 473 — backticks only, no `"`
  - Line 834 — contains `"` (mutually exclusive markers)
  - Line 911 — contains `"` (all-features = true for doc quality gates)
  - Line 1142 — contains `"` (spec-writer tools regression)
  - Line 1151 — contains `"` (spec amendment during `/pr-commented`)
  - Line 1226 — backticks only, no `"`
  - Line 1235 — backticks only, no `"`
  - Line 1249 — backticks only, no `"`
  - Line 1309 — backticks only, no `"`

  **4** of 9 contain embedded `"` (834, 911, 1142, 1151); **all 9** contain backticks. The script's escape handler MUST cover the `"` shapes (the 4 cases above) and pass backticks through verbatim (YAML double-quoted scalars treat `` ` `` literally).
- Quoting policy: ALWAYS double-quoted YAML scalar (preserves whitespace, backticks, parens, commas, and the `(date 1da36b0)` / backtick-bracketed shapes that appear in real `Escalated?` values without per-character re-escape decisions).
- **No** `id:` / `date:` / `category:` keys — recoverable from heading at zero per-entry cost (per spec Scope item 3).

### File-level FM shape (decided here, sourced from spec)

Lines 1–4 of `ai-docs/learnings.md`:

```yaml
---
schema_version: 1
kind: learnings
---
```

Line 5 = blank; line 6 = existing `# Learnings` H1 title (currently line 1). Pre-existing line shift: every existing line moves down by 5. The backfill script handles the file-level FM insertion as the same single-pass operation as per-entry preambles.

### Token-math verification (AC18)

Round-2 design owns the design-phase calculation; spec's *Token-math sketch* is reproduced and verified against measured baselines.

- **(a) Measured baseline.** `wc -c ai-docs/learnings.md` = **225,288 bytes** (≈ 225 KB) at PR-branch HEAD, 1528 lines, **136** `### YYYY-MM-DD —` entries. (Round-1 design echoed the spec's stale "~60 KB" figure; spec amendment corrected to ~225 KB; this design phase verifies the measurement.)
- **(b) Projected post-migration size.** File-level FM: 4 lines × ~20 B avg ≈ **40 B**. Per-entry FM cost: with the 3-field floor (`escalated:` always; `kind:` always; `superseded_by:` only when present, 9/136 entries), per-entry block averages ~5 lines × ~40 B avg ≈ ~200 B; 136 entries × ~200 B ≈ **27 KB total growth**. Projected post-migration size: 225,288 + ~40 + ~27,200 ≈ **~252 KB** (≈ 252,528 B). Cost is paid once per full-file Read by an LLM consumer.
- **(c) Per-`/improve` Correction pass saving.** Today, `self-improve` Step 1 Reads the entire `learnings.md` body (~225 KB) to walk every entry's `**Escalated?**` field. Post-migration: tuple-extraction prelude scans the fenced YAML blocks via PyYAML / awk on the byte stream — emitting `(date, escalated, kind, superseded_by)` tuples (no LLM token cost for the bash scan) — then Reads entry bodies only for flagged candidates. Typical `/improve` Step 1 flags **5–15 unescalated-recurrence candidates** × ~1.6 KB avg entry size ≈ **~8–25 KB body Read** (down from ~225 KB — **>89% reduction** in LLM-visible token cost).
- **(d) Per-`/improve` Carrot pass saving (now enabled by `kind:` in FM).** Today, `self-improve` Step 1b Reads the full body again to walk `**Kind:**`. Post-migration: the same tuple list carries `kind:` — Carrot pass filters in script, Reads only the `kind: validation` entries' bodies (currently 22/136 entries with explicit `**Kind:**`; the script-extracted `kind: validation` subset is typically 5–15). Body Read **~8–25 KB** (down from ~225 KB — **~90% reduction**). Round 1 design could not claim this saving because `kind:` was not in the FM floor; the round-2 amendment unlocks it.
- **(e) `/ai-audit` Phase 1 same shape.** `learnings-escalation-audit` Step 2 today walks every entry; post-migration walks only entries flagged ⚠️ / ❌ / 🌱 by the tuple scan. Comparable reduction (~85–90%).
- **(f) Net per-pass.** The ~27 KB file-growth cost is paid once per LLM full-file Read (mostly during interactive review, not skill execution). Consumer savings: **~200 KB per Correction pass + ~200 KB per Carrot pass + ~200 KB per `/ai-audit` Phase 1 run** per round. **Migration pays back within the first invocation** (one Correction-or-Carrot pass saves ~200 KB; one full-file Read pays ~27 KB — ratio ≈ 7:1). Cumulative across 3 passes per round, the migration is unambiguously net-positive. Round-1 design's lack of Carrot-pass savings (because `kind:` was not in the FM floor) is now restored — Correction AND Carrot both filter via script.

The argument above is the AC18 verifier; a runtime CI metric is in *Deferred*.

### Rejected alternatives

| Alternative | Why rejected |
|---|---|
| Single per-entry `^---$` YAML block (no fence) | Spec Scope item 2 cites round-2 finding: GitHub renderer treats only the first top-of-file `---` block as frontmatter; subsequent `---` blocks fall through to CommonMark setext-heading / HR rules → renders poorly. |
| Inline TOML-frontmatter | Inconsistent with file-level YAML; PyYAML wouldn't parse it; adds parser complexity. |
| `awk`-only backfill | Cannot mechanically YAML-escape `"` inside `Superseded by:` values without an external parser; idempotence detection (skip-if-fenced-block-already-above-heading) is straightforward in awk but the escaping makes the script brittle. 4 of 9 existing `Superseded by:` lines carry `"`; pure awk handling is rejected. |
| Manual hand-edit of 136 entries | Not byte-deterministic; not idempotent; AC16 + AC17 would require a verifier anyway. |
| Adding `category:` / `id:` / `date:` to per-entry YAML | Spec Scope item 3 + Key Decisions row "Per-entry field floor" forbids — duplicate of heading-derivable data; pure cost. (`kind:` is IN per the round-2 amendment because it gates the Carrot pass.) |
| Omitting `kind:` from the per-entry floor (round 1's shape) | Round-1 design-review Spec Amendment trigger: without `kind:` in FM, Carrot pass still requires full-body Read, breaking the bilateral net-positive claim. Spec amendment fixed this; this design honours it. |
| Hand-rolled supersession-field rewrite to a YAML list | Spec Technical constraints explicitly forbids ("No re-parsing into a YAML list — preserve the existing comma-separated format so the round-trip is mechanical"). |
| Cargo `xtask` for the backfill | Workspace has no `xtask` crate; introducing one for a single one-shot script is YAGNI. A standalone `scripts/*.py` matches the existing `scripts/gen-roadmap.sh` convention. |
| Bundling consumer-side updates into a follow-up PR | Spec Key Decisions row "Consumer-side update timing" + token-math sketch require bundled-PR scope; cost-only path (file-level FM with no consumer adoption) is not net-positive. |

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Author `scripts/backfill-learnings-frontmatter.py` (Python 3, stdlib + `PyYAML`; preamble `#!/usr/bin/env python3` + fail-loud posture: re-feeds every emitted fenced block through `yaml.safe_load` and exits 1 on any parse failure with a diagnostic naming the offending heading). Streams `ai-docs/learnings.md`; emits file-level FM block at line 1–4; for every `### YYYY-MM-DD —` heading, emits a fenced ```` ```yaml … ``` ```` block immediately above (no blank line); per-entry block contains `escalated: "<…>"` always + `kind: "<…>"` always (default `"correction"` when entry lacks `**Kind:**` line — 114 of 136 entries today) + `superseded_by: "<…>"` when the entry has a `**Superseded by:**` line (9 entries today, 4 carrying embedded `"` requiring `\"` escape). Idempotent at two granularities: (i) file-level FM is emitted only if not already present at lines 1–4; (ii) per-entry FM is emitted only above headings that don't already have a fenced ```` ```yaml ```` block immediately above containing both `escalated:` and `kind:` keys. The two granularities are checked **independently**, so the script tolerates partial-migration recovery (e.g., file-level FM already present but per-entry blocks missing for a freshly-appended entry; or vice versa) — it completes the missing layer without rewriting the present one. Full idempotence: re-running on a fully-migrated file exits 0 with no diff. CLI: `python3 scripts/backfill-learnings-frontmatter.py [--check]`; `--check` runs dry-run and exits 0 iff zero changes would be emitted. | `scripts/backfill-learnings-frontmatter.py` (new) | — |
| 2 | Run the backfill script against `ai-docs/learnings.md`; commit the resulting diff. Spot-verify: `head -6 learnings.md` shows file-level FM; AC3 verification recipe (awk row-count parity — fenced ```yaml block-close followed by `### YYYY-MM-DD —` heading line — count = 136); `python3 -c "import yaml,re; raw=open('ai-docs/learnings.md').read(); blocks=re.findall(r'\`\`\`yaml\n(.*?)\n\`\`\`\n### \d{4}-\d{2}-\d{2}', raw, re.S); assert all('kind' in yaml.safe_load(b) and 'escalated' in yaml.safe_load(b) for b in blocks)"` passes. Note: naive `grep -c '^\`\`\`yaml$'` is unreliable for the count check — see Risks (forward-safety against future entries embedding ```` ```yaml ```` body blocks). | `ai-docs/learnings.md` (modified, additive only) | 1 |
| 3 | Author `AGENTS.md § Learning Log → Boundary rule 1` revision. Insert the verbatim *additive-YAML-preamble* general-form exception blockquote from spec Scope item 5 (964 chars verbatim) between the existing line-301 `Escalated?` / `Superseded by:` subagent-driven exception and the line-303 named 2026-05-19 carve-out. Both prior exceptions remain byte-identical. | `AGENTS.md` (Boundary rule 1 section) | — |
| 4 | Author `AGENTS.md § Learning Log → Entry format` template update. The fenced template block under `### Entry format` (line 328 ff.) gains a ```` ```yaml escalated: "<value>" kind: "<value>" superseded_by: "<value>"   # only when present ``` ```` preamble shown above the `###` heading line. Add a one-paragraph note (~600 chars) below the closing ```` ``` ```` of the template block explaining: (a) the field floor (`escalated:` always; `kind:` always; `superseded_by:` when present); (b) why NOT `id:` / `date:` / `category:` (recoverable from heading); (c) per-entry-vs-file-level `kind:` disambiguation (the file-level `kind: learnings` lives in the top-of-file `---` block; the per-entry `kind: correction \| validation` lives in the fenced block above each `###` heading — parsers MUST address them by source location, not key name); (d) `**Kind:**` markdown defaulting rule unchanged (`correction` when omitted; the per-entry FM mirrors this default). | `AGENTS.md` (Entry format section) | — |
| 5 | Add a new sub-section to `ai-docs/corrections-log.md § Boundary rule 1 Exception` documenting the *additive-YAML-preamble* general-form exception. Inserted between the existing 2026-05-19 carve-out (line 19) and the `## Boundary rule 2 Exception` heading (line 21). Format: a single `> **Schema migration — additive YAML preambles (PR #<N>).** …` blockquote (`<N>` substituted with the PR number at /task Step 12 when the PR is opened — the design implementer drafts the blockquote with `#<N>` and the Step-12 commit substitutes the actual number against the freshly-opened PR for tracking issue #575) that (a) names the PR as the authorising migration, (b) cites the verbatim exception text in AGENTS.md by anchor link (single source of truth — no body duplication), (c) notes the precedent: the 2026-05-19 carve-out anticipated this ("Any future schema migration touching `learnings.md` requires its own named carve-out here"). | `ai-docs/corrections-log.md` (Boundary rule 1 Exception section) | 3 |
| 6 | Update `.claude/agents/self-improve.md § Inputs` + Step 1 + Step 1b. Insert a tuple-extraction prelude before any per-entry-body Read. Concrete invocation cited verbatim in the diff: **PyYAML form** (`python3 -c "import yaml,re,sys; raw=open('ai-docs/learnings.md').read(); blocks=re.findall(r'\`\`\`yaml\n(.*?)\n\`\`\`\n### (\d{4}-\d{2}-\d{2})', raw, re.S); for body, date in blocks: meta=yaml.safe_load(body); print(date, meta.get('escalated'), meta.get('kind'), meta.get('superseded_by'))"`) + **awk form** (`awk '/^\`\`\`yaml$/{f=1; next} /^\`\`\`$/{f=0; next} f{print}' ai-docs/learnings.md` to dump the raw fenced bodies for piping into `yaml.safe_load`). The Correction pass scans tuples for `escalated == "no"` + recurrence; the Carrot pass scans for `kind == "validation"` — both filter entirely in script now that `kind:` is in the FM floor (the round-1 markdown-body regex sidecar is gone). Entry-body Reads only on entries flagged by the tuple scan. | `.claude/agents/self-improve.md` | 2, 3, 4 |
| 7 | Update `.claude/agents/learnings-escalation-audit.md § Inputs` + Step 1 + Step 2. Step 1 (Parse entries) gains the same tuple-extraction prelude as task 6: extract `(date, escalated, kind, superseded_by)` tuples from the per-entry YAML blocks via PyYAML / awk. The `(date, category, description, rule, escalated)` extraction in current Step 1 stays — date / category / description come from the `### …` heading line via the same regex as the tuple extractor; rule comes from the `**Rule:**` body line (full-entry-body Read for that one line is unavoidable). Step 2 unchanged in verification logic — only the entry-body-Read trigger changes (Reads only for entries the tuple scan flags as ⚠️ Mismatch / ❌ Broken / 🌱 Stale-validation candidates, where Stale-validation uses `kind == "validation"` from the FM tuple). | `.claude/agents/learnings-escalation-audit.md` | 2, 3, 4 |
| 8 | Verify char-cap AXIOM post-edit (AC15). Run `wc -c AGENTS.md .claude/agents/self-improve.md .claude/agents/learnings-escalation-audit.md ai-docs/corrections-log.md` AFTER tasks 3–7 land. Acceptance: `AGENTS.md < 40000` (projected ≈ 38,644 chars), all `.claude/agents/*.md` < 40 KB (self-improve.md ~22.8 KB pre-edit; learnings-escalation-audit.md ~12.3 KB pre-edit; both stay well under), `ai-docs/corrections-log.md` < 40 KB (~13.9 KB pre-edit). If any file ≥ 39 KB, file an extraction follow-up before merge (per the AGENTS.md char-cap AXIOM table); the design phase projects AGENTS.md to 38,644 chars, below the 39 KB trigger — no extraction expected. | All five files (read-only verification) | 3, 4, 5, 6, 7 |
| 9 | Self-test the post-migration tuple-extraction commands documented in task 6 (AC12 + AC5b). Run BOTH the PyYAML invocation AND the awk fallback against the post-migration `ai-docs/learnings.md`. Verify: (a) tuple count equals `grep -c '^### 20' ai-docs/learnings.md` (should be 136); (b) tuple `escalated:` distribution matches `grep '^\*\*Escalated\?\*\*' ai-docs/learnings.md \| sort \| uniq -c` byte-for-byte (after rhs-extract); (c) tuple `kind:` distribution matches `grep '^\*\*Kind:\*\*' ai-docs/learnings.md \| sort \| uniq -c` byte-for-byte (the 22 explicit `**Kind:**` lines + 114 defaulted `kind: "correction"` blocks); (d) tuple `superseded_by:` count equals `grep -c '^\*\*Superseded by:\*\*' ai-docs/learnings.md` (should be 9). This satisfies AC5b (kind round-trip) AND AC12 (both fields' distributions). | `ai-docs/learnings.md` (read-only) | 2 |

## Handoff plan

Total subtasks M = 9. Groups follow the every-group handoff contract (a) M ≥ 1 — required, (b) maximum 3 consecutive subtasks per non-terminal group, (c) destination `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry), (d) terminal-group sizing 1..=3.

Three groups of 3 + 3 + 3 (last group is terminal at exactly 3, within `1..=3`).

- **Handoff into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). The parent /task workflow enters Group A in a fresh subagent context.
- **Group A:** subtasks 1–3 — backfill script + backfill execution + AGENTS.md Boundary-rule-1 revision. Group A produces the migrated `ai-docs/learnings.md` (data substrate) AND the verbatim AGENTS.md exception clause (authorisation) — these are the two foundation artefacts every subsequent group depends on.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — AGENTS.md Entry-format template update + corrections-log.md sibling sub-section + self-improve.md consumer update. Group B handles the AGENTS.md / corrections-log.md instruction-side authorisation completion AND the first of the two consumer-agent updates.
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group C with fresh context.
- **Group C:** subtasks 7–9 — terminal group (3 subtasks; within the 1..=3 range). learnings-escalation-audit.md consumer update + post-edit char-cap verification + AC12 + AC5b self-test of the tuple-extraction commands.

## Risks

- **Backfill script fragility on existing entry edge cases (parens, backticks, embedded double-quotes in `Superseded by:` values)** — mitigation: the script uses double-quoted YAML scalars throughout with `\"` escaping; the 9 existing `Superseded by:` lines (catalogued in the *Per-entry YAML-block shape* section above) provide the full known shape inventory — **4 of 9** contain embedded `"` (lines 834, 911, 1142, 1151) and **all 9** contain backticks (which pass through verbatim inside YAML double-quoted scalars). The Python `re`-driven escape handler covers the `"` cases; the script's fail-loud posture re-feeds every emitted block through `yaml.safe_load` and aborts with exit 1 + diagnostic naming the offending heading if any block fails to parse. AC4 + AC17 are the verifier.
- **`grep -c '^```yaml$'` mis-count from future entries that embed fenced YAML in entry bodies** — mitigation: the current `ai-docs/learnings.md` HEAD has **0** ```` ```yaml ```` fences in entry bodies (verified by `grep -c '^\`\`\`yaml$' ai-docs/learnings.md` = 0 pre-migration). This is forward-safety only: AC3 mandates `awk` row-count parity (fence-line immediately above each heading), not naive `grep -c`. The verifier MUST anchor on the pattern `\`\`\`yaml\n…\n\`\`\`\n(?=### \d{4}-\d{2}-\d{2})` — counting only fence-blocks whose closing ```` ``` ```` is immediately followed by a `### YYYY-MM-DD —` heading line. Documented in the AC3 verification recipe and in the backfill-script idempotence check, so future entries that introduce body-level ```` ```yaml ```` blocks (e.g. a hypothetical entry documenting YAML schema design) do not break the count.
- **AGENTS.md char-cap drift if reviewer requests longer prose** — mitigation: spec Technical constraints mandates `wc -c AGENTS.md` measurement after both edits land; task 8 of the decomposition operationalises this gate. Measured pre-edit 37,069 chars + verbatim Boundary-rule-1 exception 964 chars + Entry-format note ~600 chars = projected **38,644 chars**. Hard cap is 40 KB; the 39 KB threshold for proactive extraction is the design-phase escalation trigger. If a reviewer asks for >350 additional chars in either edit (post-edit ≥ 39,000), the design must propose extraction of a section into `ai-docs/corrections-log.md` before merge.
- **Consumer-agent tuple-extraction adoption gap** — risk: if the diffs to `self-improve.md` / `learnings-escalation-audit.md` cite the PyYAML invocation without the awk fallback (or vice versa), the agent may not have the cited primitive available at runtime. Mitigation: both forms are cited in the diff (task 6 + task 7); each agent picks the form that runs in its execution environment. AC10 + AC11 require the cited pattern to appear in the diff; AC12 + AC5b self-test against the post-migration file before merge.
- **`kind:` per-entry vs file-level key collision** — risk: a YAML parser configured to load the whole file (rather than per-block) could conflate the file-level `kind: learnings` with the per-entry `kind: "correction"` values. Mitigation: spec Technical constraints mandates parsers address them by source location, not by key name. The cited PyYAML invocation in tasks 6/7 uses `re.findall` on `\`\`\`yaml\n(.*?)\n\`\`\`` patterns to isolate per-entry blocks before `yaml.safe_load`, so the file-level block (delimited by `^---$`) is never co-loaded with per-entry blocks. The AGENTS.md Entry-format note (task 4 sub-bullet c) documents this disambiguation explicitly for future maintainers.
- **Idempotence regression after a future hand-edit to `learnings.md`** — risk: a future contributor appends a new entry WITHOUT running the backfill script, omitting the per-entry YAML preamble; on the next backfill run the heading WITHOUT a preamble will be detected and the preamble emitted. This is the *intended* idempotence behaviour: missing preambles get added; existing ones are left untouched. Mitigation: the new AGENTS.md Entry format template (task 4) shows the YAML preamble as part of the template, so the next manual append will follow the new convention. The 2026-05-13 Boundary-Rule-2 `/task` Steps-8–12 exception remains the path for in-flow appends; those appends will follow the updated template.
- **Propagation Rule compliance** — risk: forgetting one of the three Learning-Log-group files. The PR inherently touches AGENTS.md + `self-improve.md` + `learnings-escalation-audit.md` together (tasks 3 + 6 + 7); the Propagation Rule sync is satisfied by construction. `corrections-log.md` is touched as the Boundary-rule-1 Exception reference doc (task 5).

## Test Design

Instruction-files + scripts-only PR — no Rust diff expected. AC13 + AC14 are formality gates. The substantive tests live in the verification recipes for the file-format ACs.

- **Backfill script idempotence + fail-loud posture** (task 1)
  - Location: `scripts/backfill-learnings-frontmatter.py` (self-test mode invoked via `python3 scripts/backfill-learnings-frontmatter.py --check`)
  - Entry point: `main()` invoked twice; second invocation's `git diff --quiet ai-docs/learnings.md` returns 0; `--check` mode returns 0 iff no changes would be made
  - Scenarios:
    - Happy path: first run on a pre-migration `learnings.md` emits 1 file-level FM block + 136 per-entry blocks (each containing both `escalated:` and `kind:` keys)
    - Idempotence: second run on the post-migration file produces no diff
    - Edge case 1: entry whose `Escalated?` value contains `"` (none today; forward-safety) → emitted as `\"` escape
    - Edge case 2: entry whose `Superseded by:` value contains `("…")` (4 existing entries — lines 834, 911, 1142, 1151) → emitted as `\"...\"` escape
    - Edge case 3: entry without a `**Escalated?**` body line (0 existing today; forward-safety) → emit `escalated: "no"`
    - Edge case 4: entry without a `**Kind:**` body line (114 existing today — confirmed by `grep -c '^\*\*Kind:\*\*' = 22`, so 136 - 22 = 114) → emit `kind: "correction"`
    - Fail-loud edge case: artificially corrupt one emitted block (test fixture) → script exits 1 with diagnostic naming the offending heading
  - Fixtures: the actual `ai-docs/learnings.md` AT the pre-migration state (use `git stash` to roll back for re-test)

- **AGENTS.md Boundary rule 1 + Entry format edits** (tasks 3, 4)
  - Location: verification recipe in the PR description
  - Entry point: `diff <(awk '/^### Boundary rule 1/,/^### Boundary rule 2/' AGENTS.md.pre) <(awk '/^### Boundary rule 1/,/^### Boundary rule 2/' AGENTS.md.post)` shows ONLY an additive blockquote between the existing `Escalated?` / `Superseded by:` exception and the 2026-05-19 carve-out
  - Scenarios:
    - AC7: both prior exceptions byte-identical except for line-number shift
    - AC8: Entry format template shows ```` ```yaml ```` preamble (including the new `kind:` line) + one-paragraph note documenting field floor + per-entry-vs-file-level `kind:` disambiguation
  - Fixtures: pre-edit AGENTS.md via `git show HEAD:AGENTS.md`

- **corrections-log.md sibling sub-section** (task 5)
  - Location: verification recipe — `grep -c '^> \*\*' ai-docs/corrections-log.md` increases by 1 (new exception blockquote)
  - Entry point: `awk '/^## Boundary rule 1 Exception/,/^## Boundary rule 2 Exception/' ai-docs/corrections-log.md` ends with the new sibling sub-section
  - Scenarios: AC9 verifier
  - Fixtures: pre-edit `ai-docs/corrections-log.md` via `git show HEAD:`

- **Consumer-agent tuple-extraction self-test** (task 9 — AC5b + AC12)
  - Location: shell one-liner runnable from repo root
  - Entry point:
    ```bash
    # tuple count
    python3 -c "import yaml,re; raw=open('ai-docs/learnings.md').read(); blocks=re.findall(r'\`\`\`yaml\n(.*?)\n\`\`\`\n### (\d{4}-\d{2}-\d{2})', raw, re.S); print(len(blocks))"
    # → 136
    # escalated distribution parity
    python3 -c "import yaml,re,collections; raw=open('ai-docs/learnings.md').read(); blocks=re.findall(r'\`\`\`yaml\n(.*?)\n\`\`\`\n### \d{4}-\d{2}-\d{2}', raw, re.S); c=collections.Counter(yaml.safe_load(b)['escalated'] for b in blocks); print(sorted(c.items()))"
    # kind distribution parity (AC5b)
    python3 -c "import yaml,re,collections; raw=open('ai-docs/learnings.md').read(); blocks=re.findall(r'\`\`\`yaml\n(.*?)\n\`\`\`\n### \d{4}-\d{2}-\d{2}', raw, re.S); c=collections.Counter(yaml.safe_load(b)['kind'] for b in blocks); print(sorted(c.items()))"
    # → [('correction', 114 + N_existing_correction), ('validation', N_validation)] — must match grep distribution
    # superseded_by count
    python3 -c "import yaml,re; raw=open('ai-docs/learnings.md').read(); blocks=re.findall(r'\`\`\`yaml\n(.*?)\n\`\`\`\n### \d{4}-\d{2}-\d{2}', raw, re.S); print(sum(1 for b in blocks if 'superseded_by' in yaml.safe_load(b)))"
    # → 9
    ```
  - Scenarios: AC5b + AC12 — outputs must match `grep -c '^### 20' learnings.md` (136), the markdown-side `**Escalated?**` and `**Kind:**` distributions (from `grep | sort | uniq -c`), and `grep -c '^\*\*Superseded by:\*\*' learnings.md` (9)
  - Fixtures: the post-migration `ai-docs/learnings.md`

- **AC16 diff-purity verifier** (task 2)
  - Location: shell one-liner runnable post-migration
  - Entry point:
    ```bash
    # Strip every fenced ```yaml … ``` block immediately above a ### heading; compare to pre-migration file
    python3 -c "import re,sys; raw=sys.stdin.read(); print(re.sub(r'\`\`\`yaml\n.*?\n\`\`\`\n(?=### \d{4}-\d{2}-\d{2})', '', raw, flags=re.S))" < ai-docs/learnings.md | tail -n +6 > /tmp/stripped.md
    git show HEAD~1:ai-docs/learnings.md > /tmp/original.md
    diff /tmp/original.md /tmp/stripped.md
    # → zero diff
    ```
  - Scenarios: AC16 — backfill diff is purely additive
  - Fixtures: pre-migration `learnings.md` via `git show HEAD~1:` (or `git show <merge-base>:`)

- **AC17 idempotence verifier** (task 2)
  - Location: `python3 scripts/backfill-learnings-frontmatter.py && git diff --quiet ai-docs/learnings.md` (or `python3 scripts/backfill-learnings-frontmatter.py --check; echo $?` returning 0)
  - Entry point: exit code 0 (no diff) on the second invocation
  - Scenarios: AC17 — running the backfill script twice produces no diff
  - Fixtures: the post-migration `ai-docs/learnings.md`

- **Char-cap verification** (task 8 — AC15)
  - Location: `wc -c AGENTS.md .claude/agents/self-improve.md .claude/agents/learnings-escalation-audit.md ai-docs/corrections-log.md`
  - Entry point: each file < 40000
  - Scenarios: AC15
  - Fixtures: post-edit files

- **Token-math verification** (Approach § Token-math verification (AC18))
  - Location: design-document calculation only (no runtime probe — runtime CI metric is in *Deferred*)
  - Entry point: the six-step argument (a)–(f) in the *Approach* section
  - Scenarios: AC18 — the calculation must show net-positive within first consumer invocation
  - Fixtures: measured `wc -c ai-docs/learnings.md` = 225,288 (verified at design time); measured `grep -c '^### 20' learnings.md` = 136; measured `grep -c '^\*\*Kind:\*\*' learnings.md` = 22; measured `grep -c '^\*\*Superseded by:\*\*' learnings.md` = 9

## Open questions

None. The spec round-5 closed all open questions; round-3 reframed the optimisation target; round-4 codebase investigation locked the v1 file set; round-1 design-review (Spec Amendment) added `kind:` to the per-entry FM floor and corrected the baseline measurement. The design phase committed to (1) backfill mechanism (Python over awk) + preamble + fail-loud posture, (2) tuple-extraction invocation form (cite both PyYAML + awk; `kind:` now fully script-filterable), (3) AGENTS.md insertion-point geometry (line 301–303 + line 330–337; verified projection 38,644 chars), (4) corrections-log.md sibling-section format, (5) AC18 design-phase calculation against measured baselines — all decided above against verified codebase state (`wc -c`, `grep -c`, line-number anchors).
