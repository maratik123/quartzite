# Frontmatter on `learnings.md` (v1, token-positive only)

**Source:** issue #575
**Date:** 2026-05-26
**Tracked in:** #575

## Scope

Add YAML frontmatter to `ai-docs/learnings.md` only. The addition must be **net token-positive** end-to-end: the bytes paid on every LLM read of the file are recovered by enabling its downstream LLM-side consumers (`self-improve` via `/improve`, `learnings-escalation-audit` via `/ai-audit` Phase 1) to script-extract a filtered tuple list and Read only the entry bodies relevant to the current pass. Per the round-3 reframing, the optimisation target is **minimise token wasting without reducing efficiency**; human-reader benefit is second priority.

**Authorship context** (round 3): `ai-docs/learnings.md` is LLM-written but user-triggered ("add to learnings X"). All other candidate files in the originally-considered umbrella (`INDEX.md`, `_inbox.md`, 8 deferred-thematic files, `widget-backlog.md`) are LLM-authored by `/improve`, `/task` Step 12, `/triage`, `/ai-audit` and skills not yet enumerated.

### Final v1 file set: `learnings.md` only

Round-4 codebase investigation traced every current LLM-side consumer of each candidate file. Only `learnings.md` passes the token-positive test:

| File | Current LLM-side consumers | Read pattern | Filterable via FM? | Verdict |
|---|---|---|---|---|
| `ai-docs/learnings.md` | `self-improve` (`/improve`), `learnings-escalation-audit` (`/ai-audit` Phase 1) | Walks every entry to check `**Escalated?** …` / `**Superseded by:** …` / `**Kind:** …` | **Yes** — both subagents can script-extract a `(date, escalated, superseded_by)` tuple list via PyYAML and Read only mismatched entries | **IN v1** |
| 11 table-shaped files (`INDEX.md`, `_inbox.md`, 8 thematic deferred, `widget-backlog.md`) | `/next` (full-file `cat` of each), `/triage` drain (full body of `_inbox.md`), `triage-runner` Phase 3 sweep (every row of the thematic + widget-backlog files), `scripts/gen-roadmap.sh` (POSIX awk on `INDEX.md`) | Full body read | **No** — every consumer needs the full body today | Deferred (see *Deferred*) |

### What v1 ships

1. **File-level YAML frontmatter on `ai-docs/learnings.md`.** Single block at file head:
   ```yaml
   ---
   schema_version: 1
   kind: learnings
   ---
   ```
   `kind:` is a one-line schema discriminator; `schema_version:` is per-file independent (round-1 Q3), starting at 1 and bumped on schema-floor changes.

2. **Per-entry fenced YAML block on every `### YYYY-MM-DD — …` entry.** Fenced ```` ```yaml … ``` ```` placed immediately *above* each heading (no blank line between the closing ```` ``` ```` and the `###`). Fenced over `^---$` per round-2 finding: GitHub's renderer recognises only the first top-of-file `---` block as frontmatter; subsequent `---` blocks fall through to CommonMark setext-heading / HR rules and render poorly. Fenced YAML parses cleanly via PyYAML's fenced-code-block detection and renders as a muted code block on GitHub.

3. **Minimal per-entry field floor.** ONLY fields that (i) are NOT recoverable from existing markdown without a per-line LLM parse AND (ii) downstream consumers filter on:
   - `escalated:` — always present; quoted string mirroring the value to the right of `**Escalated?**` in the markdown body, whitespace-trimmed. For entries lacking the bold-key line, `escalated: "no"` (mirrors AGENTS.md default-when-omitted).
   - `kind:` — always present; quoted string mirroring the value to the right of `**Kind:**` in the markdown body, whitespace-trimmed. For entries lacking the bold-key line, `kind: "correction"` (mirrors AGENTS.md default-when-omitted). This is the **per-entry** Kind (correction | validation), NOT the file-level `kind: learnings` (file-class discriminator); the two are independent fields with different semantics.
   - `superseded_by:` — present only when the entry has a `**Superseded by:**` markdown line; quoted string mirroring the value, whitespace-trimmed. Embedded double quotes YAML-escaped as `\"`.
   - **NOT** `id:` / `date:` / `category:` — recoverable from the heading line at zero per-entry cost (heading shape: `### YYYY-MM-DD — [category] — [short description]`). Adding them is pure duplication.

4. **Backfill of all 136 existing entries** in the same PR — one-shot file-format migration commit, purely additive (no character of any existing heading / body / blank-line separator changes). Authorised by the Boundary rule 1 general-form revision below.

5. **AGENTS.md `§ Learning Log → Boundary rule 1` general-form revision.** Concrete proposed text, added as a new exception clause **after** the existing `Escalated?` / `Superseded by:` subagent-driven exception and **before** the named 2026-05-19 carve-out (both preserved verbatim):

   > **Exception — additive YAML preambles.** A fenced ```` ```yaml … ``` ```` block placed **above** an entry's `### YYYY-MM-DD — …` heading that preserves every byte of the entry's existing heading, body lines (`**What happened:**`, `**Rule:**`, `**Kind:**`, `**Escalated?**`, `**Superseded by:**`), and blank-line separator does NOT count as "editing, rewriting, reordering, summarising, or deleting" under this rule. Such preambles MAY be added in bulk by a one-shot file-format migration commit; the migration commit must be byte-deterministic (re-running it produces no diff). Subsequent updates to fields inside the preamble follow the same Subagent-driven contract as the existing `Escalated?` / `Superseded by:` exception (only `self-improve` / `learnings-escalation-audit` may rewrite preamble field values). See [`ai-docs/corrections-log.md` → Boundary rule 1 Exception](ai-docs/corrections-log.md#boundary-rule-1-exception) for the per-Subagent contract.

   Recurrence record: this revision is the named precedent the 2026-05-19 carve-out anticipated ("Any future schema migration touching `learnings.md` requires its own named carve-out here.").

6. **AGENTS.md `§ Learning Log → Entry format` template update.** The current template block gains a ```` ```yaml ```` preamble shown above the `###` heading line, plus a one-paragraph note explaining the field floor + the no-`id`/no-`date`/no-`category`/no-`kind` rationale. Single source of truth — no separate reference doc.

7. **`ai-docs/corrections-log.md § Boundary rule 1 Exception` update.** A new sub-section (sibling to the existing 2026-05-19 carve-out) documents the *additive-YAML-preamble* general-form exception with this PR cited as the authorising migration.

8. **Consumer-side updates bundled in the same PR.** Net token-positive from day 1:
   - **`.claude/agents/self-improve.md § Inputs` + Step 1 / Step 1b.** Before any per-entry-body Read, extract `(date, escalated, kind, superseded_by)` tuples from the per-entry YAML preambles via PyYAML or `awk`. Read only the entry bodies whose tuples flag a candidate for the current pass (Correction pass: `escalated: "no"` + recurrence; Carrot pass: `kind: validation` + topic group). Concrete invocation cited in the diff (e.g. `python3 -c "import yaml; ..."` or `awk '/^\`\`\`yaml/,/^\`\`\`$/{print}'` followed by `yaml.safe_load`).
   - **`.claude/agents/learnings-escalation-audit.md § Inputs` + Step 1 / Step 2.** Same tuple-extraction pattern; full-entry Reads only on entries flagged ⚠️ Mismatch / ❌ Broken / 🌱 Stale-validation by the tuple scan.
   - The existing `**Escalated?**` / `**Superseded by:**` markdown lines remain immutable per Boundary rule 1; the per-entry YAML preamble is a parallel projection of the same data, optimised for script-extraction. Subagents may use either source-of-truth; the YAML preamble is the cheap one to scan.
   - **`AGENTS.md § Learning Log → Entry format`** updated per item 6 above.
   - **`.claude/agents/spec-writer.md`** is unaffected — does not read `learnings.md`.

9. **No new reference doc.** The earlier draft proposed `ai-docs/frontmatter-convention.md`; under the token-min framing a new instruction file is itself token cost on every Read and adds char-cap-AXIOM surface. The per-entry FM contract is documented inline in `AGENTS.md § Learning Log → Entry format` — single source of truth, no fan-out. Re-open the convention-doc question only when a second file enters the FM umbrella.

### Token-math sketch (net-positive case)

Worked example for one representative `/improve` Step 1 run on the post-migration `learnings.md`:

- **Baseline (measured):** `wc -c ai-docs/learnings.md` = **~225 KB** today (1528 lines, 136 entries). Note: an earlier draft of this sketch cited "~60 KB" — that figure was wrong; the actual baseline is ~225 KB, which strengthens the net-positive case (savings are larger in absolute terms).
- **File-level FM cost:** 4 lines (`---` / `schema_version: 1` / `kind: learnings` / `---`) ≈ 40 bytes paid on every full Read. Amortised across multi-Read invocations, near-zero.
- **Per-entry FM cost:** with the 3-field per-entry floor (`escalated:` always; `kind:` always; `superseded_by:` only when present), ~5–6 lines per entry × 136 entries ≈ 770 lines ≈ **~27 KB total file growth** (post-migration projected size ≈ 252 KB). Paid only by consumers that Read past the head.
- **Per-`/improve` Correction-pass saving:** today, `self-improve` Step 1 Reads the entire `learnings.md` body (~225 KB) to walk every entry's `**Escalated?**` field. After migration, the tuple-extraction prelude scans only the fenced YAML blocks via PyYAML / awk on the byte stream — no LLM token cost for that scan — then Reads entry bodies only for flagged candidates. Typical `/improve` Step 1 flags ~5–15 unescalated-recurrence candidates × ~1.6 KB avg entry size ≈ **~8–25 KB body Read** (down from ~225 KB — **>89% reduction**).
- **Per-`/improve` Carrot-pass saving (enabled by `kind:` in FM):** today, `self-improve` Step 1b Reads the full body again to walk `**Kind:**`. After migration, the same tuple list carries `kind:` — Carrot pass filters in script, Reads only the `kind: validation` entries' bodies. Comparable reduction (~90%).
- **`/ai-audit` Phase 1 same shape:** `learnings-escalation-audit` Step 2 today walks every entry; post-migration walks only entries flagged ⚠️ / ❌ / 🌱 by the tuple scan. Comparable reduction.
- **Net per-pass:** the ~27 KB file-growth cost is paid once per LLM Read of the full file (mostly during interactive review, not skill execution). Consumer savings are **~200 KB per `/improve` Correction pass + ~200 KB per Carrot pass + ~200 KB per `/ai-audit` Phase 1 run** (each round). The migration pays back within the **first** consumer invocation. The cost-only path (file-level FM with no consumer adoption) would NOT be net-positive — hence the bundled-PR scope.

## Out of scope

- **File-level frontmatter on the 11 table-shaped files** (`INDEX.md`, `_inbox.md`, 8 thematic deferred files, `widget-backlog.md`). The codebase investigation (round 4) confirms zero LLM-side consumer that would filter via FM today. Re-evaluate per file when a concrete script-extraction consumer is proposed (see *Deferred*).
- **Per-row / per-entry frontmatter on any file other than `learnings.md`.** Table columns already encode the filterable data structurally; per-row YAML duplicates it (pure cost under the round-3 reframing).
- **Python consumer scripts** (`scripts/learnings-stats.py`, etc.). The frontmatter is the enabling substrate; specific scripts are independent tasks the user opens as needed.
- **Mechanical CI gate for frontmatter schema validity** (`scripts/check-frontmatter.sh`). v1 ships one file's schema; a validator is meaningful once a multi-file schema set stabilises.
- **Conversion of any markdown table to a pure YAML array.** Issue body explicitly preserves the tables ("Same tables.").
- **Rewriting `gen-roadmap.sh` in Python.** Bash + awk stays; `INDEX.md` is untouched in v1.
- **Touching short / static markdown reference files** beyond the Boundary-rule-1 + Entry-format revisions in `AGENTS.md` + the matching exception sub-section in `ai-docs/corrections-log.md`. `ai-docs/context.md`, `ai-docs/code-style.md`, `ai-docs/doc-convention.md`, `ai-docs/workflow.md` etc. are unaffected.
- **`ai-docs/plans/*.spec.md` / `*.design.md` / `*.progress.md`** — short-lived per-task artefacts with their own `**Source:** / **Date:** / **Tracked in:**` preamble convention; not "long-running".
- **Touching the existing `**Escalated?**` / `**Superseded by:**` markdown lines inside each entry** during backfill. The new per-entry YAML block sits *above* the heading; the bold-key markdown lines remain immutable per Boundary rule 1 (the revision authorises *additive* YAML above the heading, not edits within the entry body). Heterogeneous coexistence is intentional.
- **A shared `ai-docs/frontmatter-convention.md` reference doc.** Documenting the per-entry contract inline in `AGENTS.md § Learning Log → Entry format` is sufficient for a single-file umbrella; a fan-out doc adds char-cap surface for zero benefit until a second file joins.

## Deferred

| Item | Why | Re-entry criterion | Separate issue? |
|---|---|---|---|
| File-level FM on the 11 table-shaped files (`INDEX.md`, `_inbox.md`, 8 thematic deferred, `widget-backlog.md`) | Round-4 codebase investigation: every current LLM-side consumer (`/next`, `/triage` drain, `triage-runner` sweep, `gen-roadmap.sh`) Reads the full file. No filterable read pattern exists today, so file-level FM is pure cost under the round-3 reframing. | **A concrete LLM-side script-extraction consumer is proposed for the target file** (e.g. a `/next` filter-aware pass for `INDEX.md` keyed on status emoji; a per-thematic `/triage` partial-load; a `widget-backlog.md` Status-column filter). Open the issue when the consumer redesign is on the table. | Yes — per file, opened only when its consumer surfaces. |
| Per-row frontmatter on the 11 table files | Table columns already encode the filterable data; per-row YAML is duplicate token cost. | Defer indefinitely; re-open only on concrete consumer demand that table-column parsing cannot satisfy. | No (defer indefinitely). |
| Python consumer scripts (`scripts/learnings-stats.py`, etc.) | v1 ships the substrate; specific tooling scripts are independent tasks. | User opens an issue requesting a specific script. | Yes — per script. |
| Mechanical CI gate (`scripts/check-frontmatter.sh`) | Value emerges once a multi-file schema set stabilises. | A second file enters the FM umbrella. | Yes (open after the second file lands). |
| Future schema growth on the `learnings.md` per-entry block (`category:`, `id:`, `linked_pr:`, etc.) | v1 floor is the audited minimum; growth bumps `schema_version` and lands per-need. | A downstream consumer surfaces that filters on the proposed field. | Yes — per field. |
| `ai-docs/frontmatter-convention.md` reference doc | v1 has only one file under the umbrella; an extra instruction file is itself token cost on every Read and adds char-cap-AXIOM surface. | A second file enters the FM umbrella. | Yes — opened together with the second-file PR. |
| CI metric quantifying the token-positive claim (e.g. `scripts/bench-frontmatter-load.sh`) | v1 verifies the claim by design-phase calculation in the *Token-math sketch* above; a runtime metric is a separate concern. | A regression or claim-divergence surfaces in practice. | Yes. |

## Key decisions

| Question | Decision |
|---|---|
| Primary optimisation target | Minimise token wasting without reducing efficiency. Human-reader benefit second priority (round-3 reframing). |
| Per-file inclusion test | A file enters v1 only if a current LLM-side consumer can use the frontmatter to script-extract a filtered subset, recovering more tokens than the FM bytes cost across the expected read frequency. |
| v1 file set | **One file: `ai-docs/learnings.md`.** The 11 table-shaped files fail the inclusion test today under round-4 codebase investigation. |
| Granularity on `learnings.md` | File-level frontmatter (one block at file head) **+** per-entry fenced YAML block above each `### YYYY-MM-DD …` heading. Both layers are token-positive because `self-improve` + `learnings-escalation-audit` both walk-every-entry today. |
| File-level frontmatter shape | `^---$\nschema_version: 1\nkind: learnings\n---$` at the top of the file. `kind:` is a one-line schema discriminator; `schema_version:` is per-file independent (round-1 Q3), starting at 1, bumped on schema-floor changes. |
| Per-entry block shape | Fenced ```` ```yaml … ``` ```` immediately above each `### YYYY-MM-DD — …` heading. Fenced over `^---$` per round-2 GitHub-renderer finding. PyYAML detects fenced code blocks cleanly. |
| Per-entry field floor | **`escalated:`** (always present; quoted string mirroring `**Escalated?**` value, whitespace-trimmed; `"no"` when bold-key line absent) **+** **`kind:`** (always present; quoted string mirroring `**Kind:**` value, whitespace-trimmed; `"correction"` when bold-key line absent — the **per-entry** Kind, distinct from the file-level `kind: learnings` discriminator) **+** **`superseded_by:`** (present only when entry has a `**Superseded by:**` line; quoted string mirroring its value). **NOT** `id:` / `date:` / `category:` — recoverable from heading at zero per-entry cost. |
| Backfill strategy | All 136 existing entries get the per-entry block in the same PR — one-shot byte-deterministic file-format migration commit. Authorised by the Boundary rule 1 general-form revision below. |
| Boundary rule 1 revision | General-form *additive YAML preamble* exception added between the existing `Escalated?` / `Superseded by:` subagent exception and the named 2026-05-19 carve-out. Verbatim proposed text in Scope item 5. Both prior exceptions preserved. |
| Consumer-side update timing | Bundled in the same PR (round-3 v1-scope answer — "Migration + consumer-side bundled. Net token-positive from day 1."). `self-improve` Step 1 / Step 1b + `learnings-escalation-audit` Step 1 / Step 2 adopt PyYAML / `awk` tuple-extraction; entry-body Reads only on flagged entries. |
| Reference doc | NONE in v1. Per-entry FM contract documented inline in `AGENTS.md § Learning Log → Entry format`. A separate `ai-docs/frontmatter-convention.md` is deferred. |
| 11 table-shaped files | Deferred per-file with explicit re-entry criterion (a concrete LLM-side script-extraction consumer is proposed). The issue-body "processed by python scripts" goal is preserved for them by their existing table shape — markdown table parsers handle them today. |

## Technical constraints

- File-level + per-entry blocks MUST be parseable by `PyYAML` `yaml.safe_load` without custom tags.
- File-level block MUST be the first non-empty bytes of `ai-docs/learnings.md` (line 1 = `---`).
- Per-entry fenced ```` ```yaml ```` block MUST sit immediately above each `### YYYY-MM-DD — …` heading (no blank line between the closing ```` ``` ```` and the heading). Exactly one blank line separates entries (existing convention preserved).
- Per-entry block MUST emit `escalated:` AND `kind:` for every entry (`escalated: "no"` and `kind: "correction"` defaults for entries currently lacking the respective bold-key lines, mirroring the AGENTS.md default-when-omitted rules). `superseded_by:` is emitted only when the entry has a `**Superseded by:**` markdown line.
- `escalated:` value is a quoted string identical to the right-hand side of `**Escalated?**` after the colon, whitespace-trimmed (e.g. `escalated: "skill:task, hook"`). No re-parsing into a YAML list — preserve the existing comma-separated format so the round-trip is mechanical.
- `kind:` value is a quoted string identical to the right-hand side of `**Kind:**` after the colon, whitespace-trimmed (e.g. `kind: "correction"` or `kind: "validation"`). This is the **per-entry** Kind discriminator (`correction` | `validation`) — NOT the file-level `kind: learnings` (file-class discriminator). YAML key collision is by design: file-level `kind:` lives in the top-of-file `---`-delimited block; per-entry `kind:` lives in the fenced ```yaml``` block above each `###` heading. Parsers MUST address them by source location, not by key name.
- `superseded_by:` value is a quoted string identical to the right-hand side of `**Superseded by:**` after the colon, whitespace-trimmed (e.g. `superseded_by: "2026-05-15 (\"mutually exclusive markers\")"`). Embedded double quotes are YAML-escaped as `\"`.
- `self-improve` Step 1 / Step 1b + `learnings-escalation-audit` Step 1 / Step 2 MUST extract tuples via PyYAML / awk on the fenced blocks; entry-body Reads only on entries flagged by the tuple scan.
- The Boundary rule 1 revision MUST preserve (verbatim, in source order): (a) the existing `Escalated?` / `Superseded by:` subagent-driven exception, (b) the 2026-05-19 compaction-recovery-protocol named carve-out. The new general-form clause is inserted between (a) and (b) and is additive — neither prior exception is replaced or narrowed.
- The per-entry block contract is documented inline in `AGENTS.md § Learning Log → Entry format` (the existing template gains a ```` ```yaml ```` block above the `###` heading line, with a one-paragraph note explaining the field floor + the no-id/no-date/no-category/no-kind rationale). No separate reference doc.
- The `ai-docs/corrections-log.md § Boundary rule 1 Exception` section gains a new sub-section (sibling to the existing 2026-05-19 carve-out) documenting the additive-YAML-preamble general-form exception, citing this PR.
- Backfill commit MUST be byte-deterministic — running the backfill script twice in a row produces no diff on the second run. The backfill is purely additive and reads each entry's existing bold-key lines as source-of-truth.
- Consumer-side updates land in the same PR (single PR, multiple commits OK).
- **AGENTS.md char-cap awareness.** Current size is **~37 KB** (already in the 35k–39.9k *minor* warning band per the AGENTS.md char-cap AXIOM table). The Boundary-rule-1 revision (~8 lines, ~700 chars) + the Entry-format template update (~10 lines, ~600 chars) net add ~1.3 KB. Post-revision target: still **< 40 KB** (hard cap). The design phase MUST measure with `wc -c AGENTS.md` after both edits land and budget proactive extraction if the post-edit number is ≥ 39 KB.
- `.claude/agents/self-improve.md` (~22.8 KB) + `.claude/agents/learnings-escalation-audit.md` (~12.3 KB) Step-1 updates stay under the 40k cap; design phase confirms with `wc -c` after edits.
- Propagation Rule check: edits to `AGENTS.md § Learning Log` sync to `.claude/agents/self-improve.md` AND `.claude/agents/learnings-escalation-audit.md` (Learning-Log group). This PR satisfies the sync inherently — all three are edited together.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/learnings.md` line 1 is `---`; lines 2–3 are `schema_version: 1` + `kind: learnings`; line 4 is `---`; line 5 is blank; line 6 begins the existing body (`# Learnings` title or first entry per design call). |
| AC2 | `python3 -c "import yaml; raw=open('ai-docs/learnings.md').read(); print(yaml.safe_load(raw.split('---',2)[1]))"` returns `{'schema_version': 1, 'kind': 'learnings'}` (a dict, not `None` / error). |
| AC3 | Every `### YYYY-MM-DD — …` heading in `ai-docs/learnings.md` has a fenced ```` ```yaml … ``` ```` block immediately above it (no blank line between the closing ```` ``` ```` and the `###`). Count of fenced YAML blocks == count of `### YYYY-MM-DD —` headings (136 today). Verified by `awk` row-count parity. |
| AC4 | Every per-entry fenced block parses via `yaml.safe_load` to a dict containing at minimum `escalated:` (string) AND `kind:` (string, one of `"correction"` / `"validation"`). Blocks for entries with a `**Superseded by:**` markdown line also contain `superseded_by:` (string). No block contains `id:` / `date:` / `category:`. |
| AC5 | For every entry, `escalated:` YAML value === the right-hand side of `**Escalated?**` in the markdown body (whitespace-trimmed). For entries lacking the bold-key line, `escalated: "no"` (default per AGENTS.md). Verified by a one-shot diff script. |
| AC5b | For every entry, `kind:` YAML value === the right-hand side of `**Kind:**` in the markdown body (whitespace-trimmed). For entries lacking the bold-key line, `kind: "correction"` (default per AGENTS.md). Verified by the same one-shot diff script (`kind:` and `escalated:` are checked in the same pass). |
| AC6 | For every entry with a `**Superseded by:**` line, `superseded_by:` YAML value === the right-hand side of the bold-key line (whitespace-trimmed). For entries without the line, no `superseded_by:` field appears in the YAML block. |
| AC7 | `AGENTS.md § Learning Log → Boundary rule 1` gains the *additive-YAML-preamble* general-form exception clause (verbatim text per Scope item 5), inserted between the existing `Escalated?` / `Superseded by:` subagent exception and the 2026-05-19 named carve-out. Both prior exceptions remain byte-identical. |
| AC8 | `AGENTS.md § Learning Log → Entry format` shows the updated template: a ```` ```yaml ```` block above the `###` heading, plus a one-paragraph note documenting the floor (`escalated:` always; `superseded_by:` when present) and the no-redundant-field rationale. |
| AC9 | `ai-docs/corrections-log.md § Boundary rule 1 Exception` gains a new sub-section (sibling to the existing 2026-05-19 carve-out) documenting the additive-YAML-preamble general-form exception, citing this PR as the authorising migration. |
| AC10 | `.claude/agents/self-improve.md § Inputs` + Step 1 / Step 1b updates: workflow extracts `(date, escalated, kind, superseded_by)` tuples from per-entry YAML blocks via PyYAML / awk before any per-entry-body Read; entry-body Reads limited to entries flagged by the tuple scan. The diff cites a specific implementation pattern (concrete `awk` / `python3 -c` invocation). |
| AC11 | `.claude/agents/learnings-escalation-audit.md § Inputs` + Step 1 / Step 2 updates: same tuple-extraction pattern; full-entry Reads only on ⚠️ Mismatch / ❌ Broken / 🌱 Stale-validation candidates flagged by the tuple scan. The diff cites a specific implementation pattern. |
| AC12 | A self-test of the new tuple-extraction commands documented in AC10 / AC11 — invoked against the post-migration `learnings.md` — produces a non-empty tuple list whose count equals the number of `### YYYY-MM-DD —` headings (136), whose `escalated:` distribution matches the existing `grep '\*\*Escalated\?\*\*'` distribution byte-for-byte, AND whose `kind:` distribution matches the existing `grep '\*\*Kind:\*\*'` distribution byte-for-byte. The two views (YAML preamble vs. markdown bold-key lines) are consistent for both fields. |
| AC13 | `cargo build` + `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt -- --check` + `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all pass (zero Rust diff is expected — instruction-files-only PR; gates are formality). |
| AC14 | `actionlint .github/workflows/*.yml` passes (no workflow files modified is expected). |
| AC15 | Char-cap AXIOM: post-revision `wc -c AGENTS.md` < 40000 (current ~37 KB; revision adds ~1.3 KB → ~38.3 KB target, still within `minor` band). `.claude/agents/self-improve.md` (~22.8 KB → < 40 KB) and `.claude/agents/learnings-escalation-audit.md` (~12.3 KB → < 40 KB) stay well under the cap. All other `.claude/skills/**/*.md` + `.claude/agents/**.md` + `.claude/rules/*.md` + `ai-docs/{code-style,doc-convention,context,agent-writing-style,corrections-log}.md` stay < 40 KB. |
| AC16 | Backfill commit's `ai-docs/learnings.md` diff is purely additive — no character of any existing `### YYYY-MM-DD — …` heading, `**What happened:**` / `**Rule:**` / `**Kind:**` / `**Escalated?**` / `**Superseded by:**` body line, blank-line separator, or prose paragraph changes. Verified by reproducing the pre-PR file via `sed`-strip-fenced-yaml of the post-PR file and `diff`-ing against the pre-PR file (zero diff). |
| AC17 | Backfill is byte-deterministic: running the backfill script twice in a row against the post-PR file produces no diff on the second run. |
| AC18 | Token-positive verification (design-phase calculation): the *Token-math sketch* in Scope holds — typical `/improve` Step 1 entry-body Read drops from ~60 KB (full body walk) to ~3–10 KB (flagged entries only), and `/ai-audit` Phase 1 shows comparable reduction. A runtime CI metric is deferred (see Deferred); v1 verification is the design-phase argument, not a runtime probe. |

## Open questions

None remaining. Round-5 Tableset? answer locked the v1 file set; round-3 reframing locked the optimisation target; round-2 answers locked the per-entry shape, field floor, and backfill authorisation. The Token-positive verification rigour (formerly Open Q2) is resolved as *design-phase calculation only*; the CI-metric path lives in *Deferred*.
