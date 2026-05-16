# AGENTS.md proactive extraction (clear 35k early-warning band)

**Source:** issue #418
**Date:** 2026-05-16
**Tracked in:** #418

## Scope

1. Create two new reference pages under `ai-docs/`:
   - **`ai-docs/agent-docs-index.md`** — receives the verbose multi-sentence row descriptions from `AGENTS.md § Agent Docs`. AGENTS.md's table itself keeps the path (column 1) + a one-line description stub (≤ 80 chars per row); the long bodies move out.
   - **`ai-docs/api-naming.md`** — receives the entire `AGENTS.md § API Naming` section body verbatim (the `_unchecked` AXIOM block + the per-shape action table + the rule-prose bullets). AGENTS.md keeps the `## API Naming` level-2 heading and collapses the body to a one-line cross-link stub.
2. Edit `AGENTS.md`:
   - Replace `§ Agent Docs` row bodies with one-line stubs per the partial-extract policy below.
   - Replace `§ API Naming` body with the one-line stub per the whole-extract policy below.
   - Add a row to `§ Agent Docs` for each of the two new reference pages (`ai-docs/api-naming.md` and `ai-docs/agent-docs-index.md`) — both rows are one-line stubs per the partial-extract policy. The `ai-docs/agent-docs-index.md` row is a minor self-reference (the meta-index of its own table) and is acceptable.
3. Anchor-aware cross-references: every stub-link emitted from AGENTS.md MUST point at a real ATX `## ` (or deeper) heading in the destination file. Concretely:
   - `ai-docs/agent-docs-index.md` carries a `## Agent doc rows` (or equivalent slug) heading the extracted rows can be anchored against. Stub links use `[ai-docs/agent-docs-index.md → Agent doc rows](ai-docs/agent-docs-index.md#agent-doc-rows)` form (anchor slug TBD by design, MUST match the heading rendered by GitHub's slugger).
   - `ai-docs/api-naming.md` carries a `## _unchecked AXIOM` (or equivalent) heading. Stub link from AGENTS.md uses the same anchored form.
4. Each destination file opens with a one-sentence first paragraph cross-linking back to `AGENTS.md § <Section>` — matches the precedent set by `ai-docs/workflow.md`, `ai-docs/corrections-log.md`, `ai-docs/key-decisions.md`.
5. Per-commit ordering on the feature branch is **monotonic-decreasing**: every commit boundary keeps `wc -c AGENTS.md < 40,000`, and AGENTS.md's size only **decreases** at each commit boundary. Recommended subtask ordering (design phase confirms grouping):
   - **Step 1 (largest first):** Create `ai-docs/agent-docs-index.md` AND in the same commit replace `§ Agent Docs` row bodies with stubs. Expected net delta ≈ −3,000 to −4,000 chars. Single commit so the AGENTS.md size never transiently grows.
   - **Step 2:** Create `ai-docs/api-naming.md` AND in the same commit replace `§ API Naming` body with the stub + add the two new `§ Agent Docs` rows for the new reference pages. Expected net delta ≈ −2,500 chars.

## Out of scope

- Mechanical pre-commit / CI gate for instruction-file size enforcement — tracked separately in #383.
- Substantive rewriting of extracted content. The extraction is a **move + stub**, not a content edit; binding semantics must be preserved verbatim. Editorial micro-tightening inside the destination body is permitted only when it does not alter any rule's meaning.
- Extraction of `AGENTS.md § Workflow` — second-highest extraction value, second-highest context risk; deferred (see *Deferred*).
- Further extraction of `AGENTS.md § Code Style` — already largely cross-referenced to `ai-docs/code-style.md`; diminishing returns.
- Further extraction of `AGENTS.md § Propagation Rule` — PR #376 already extracted the `agent-writing-style.md` row; remaining rows are mostly one-liners.
- Extraction of `AGENTS.md § Build & Test` — the 40k AXIOM itself lives in that section; recursive extraction would be confusing.
- Edits to `.claude/skills/**` or `.claude/agents/**` files — they have their own per-file budgets, audited independently, and are not in this PR's scope.
- Substantive Rust source changes (none expected; this PR is markdown-only).

## Deferred

- `AGENTS.md § Workflow` extraction | second-largest section at ~7.8k chars but carries AXIOM 1 / AXIOM 2 / CI-fix self-review rule / learnings-staging rule / PR-review-comment recipe — high context risk because multiple downstream skills depend on the rule placement | revisit whenever a future edit pushes AGENTS.md back into the 35k early-warning band; no separate issue needed (the AXIOM itself prescribes the action).
- Second-round extraction analysis after this PR lands | the post-extraction figure will inform whether a third reference page is justified | defer to the next PR that hits the early-warning band; no separate issue needed.

## Key decisions

| Question | Decision |
|---|---|
| Extraction target(s) | Combined `§ Agent Docs` verbose row descriptions **AND** `§ API Naming` body. Two new destination files. Combined net delta ≈ −5,500 to −6,500 chars. Comfortably clears the 35k early-warning band with ≥ 1k safety margin (target post-extraction `wc -c AGENTS.md ≤ 34,000`). |
| Destination filenames | `ai-docs/agent-docs-index.md` (new) for the `§ Agent Docs` verbose rows; `ai-docs/api-naming.md` (new) for the `§ API Naming` body. Naming matches existing `ai-docs/<topic>.md` convention (`workflow.md`, `corrections-log.md`, `key-decisions.md`, `dependency-versions.md`). |
| `§ Agent Docs` extraction policy | **Partial extract.** AGENTS.md's table keeps column 1 (file path) + a one-line description stub per row (≤ 80 chars). Verbose multi-sentence row bodies (examples, sub-rules, embedded prose) move to `ai-docs/agent-docs-index.md` under a `## Agent doc rows` (or equivalent) section. The table's row enumeration MUST stay in AGENTS.md — it is the navigable index consumers (e.g., `.claude/skills/next/SKILL.md`) grep against; only the *bodies* move. |
| `§ API Naming` extraction policy | **Whole extract.** Entire section body (AXIOM block + table + bullets) moves to `ai-docs/api-naming.md`. AGENTS.md keeps the `## API Naming` level-2 heading and collapses the body to a single cross-link line ("See [ai-docs/api-naming.md](ai-docs/api-naming.md#unchecked-axiom) for the `_unchecked` AXIOM and rules.") plus the one-line summary headline. |
| Extraction model | PR #324 model: move verbose body to `ai-docs/<topic>.md`; AGENTS.md keeps a one-line stub plus an anchored link to the moved content. The destination's first paragraph cross-links back to `AGENTS.md § <Section>`. |
| Anchor-link style | Existing repo convention — `[ai-docs/<topic>.md → <Section>](ai-docs/<topic>.md#anchor-slug)`. Anchor slugs must match the GitHub-rendered heading slug (lowercase, hyphens, no punctuation). Design phase fixes concrete slugs and verifies them with `/ai-audit` Phase 2 Step 2.6.4. |
| Per-commit ordering | Monotonic-decreasing. Step 1 (largest-first): `agent-docs-index.md` creation + `§ Agent Docs` stub-replacement in **one** commit (net ≈ −3.5k chars). Step 2: `api-naming.md` creation + `§ API Naming` stub-replacement + two new `§ Agent Docs` rows for the new reference pages in **one** commit (net ≈ −2.5k chars). Every commit boundary leaves AGENTS.md strictly smaller than at the previous boundary, never transiently > 40,000. |
| Workspace gates | No Rust source changes expected. Required gates: `cargo build` (refreshes `Cargo.lock` per `AGENTS.md § Workflow`), `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS=... cargo doc` per AGENTS.md recipe. Markdown content is not directly subject to clippy/doc; gates exist to confirm no Rust slipped in. |
| Propagation Rule audit | `AGENTS.md`-only edit (no rule add / no exemption) does not directly trigger sync-group propagation. **However**, the Procedure grep MUST be run post-extraction (`grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md`) to confirm no downstream file references a `§ Agent Docs` or `§ API Naming` body line that the extraction has moved. Specifically: `.claude/skills/next/SKILL.md` is a known consumer of the `§ Agent Docs` row enumeration — its grep target (file-path column) must still hit because rows do NOT disappear, only their *bodies* move. |
| Rule-5 blacklist | No additions. The 40k extraction rule is structural (file-size budget) — not a question-time pre-resolved rule the spec-writer subagent enforces against draft questions. |

## Technical constraints

- **Hard upper bound:** every commit boundary on the feature branch reports `wc -c AGENTS.md < 40,000`.
- **Soft target:** post-extraction `wc -c AGENTS.md ≤ 34,000` (clears the early-warning band with ≥ 1,000-char safety margin).
- **Anchor resolution:** every `[…](ai-docs/<topic>.md#anchor)` link emitted from AGENTS.md must resolve to a real ATX `## ` (or deeper) heading slug in the destination file. `/ai-audit` Phase 2 Step 2.6.4 is the canonical verifier; running it locally before push is a hard gate.
- **Row enumeration stability for `§ Agent Docs`:** the file-path column of every row stays in AGENTS.md. Downstream consumers (e.g., `.claude/skills/next/SKILL.md`) that grep for a row path must continue to find it. The partial-extract policy moves **body content only**, not row identity.
- **Content fidelity:** the extracted body must remain a faithful reproduction of the source. Acceptable shape: (a) verbatim move + section heading, OR (b) move-and-lightly-restructure (e.g., promote a sub-bullet to a sub-heading) without changing any binding semantic. Pre/post diff during self-review must show only formatting / cross-link prose changes, not rule-text changes.
- **Self-reference:** the new `ai-docs/agent-docs-index.md` row in `AGENTS.md § Agent Docs` references the very table it lives in. Accept the minor self-reference; the navigability benefit (a reader can hop from AGENTS.md to the index of agent-doc rows) outweighs the loop.
- **Demonstrator capture:** the post-merge `/ai-audit` Phase 2 Sub-check 9 run is the canonical demonstrator. The PR body must record `wc -c AGENTS.md` before and after, plus the `/ai-audit` Sub-check 9 before/after diff.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/agent-docs-index.md` exists with a first paragraph cross-linking back to `AGENTS.md § Agent Docs`, an ATX `## ` (or deeper) heading slug each AGENTS.md stub link targets, and the verbose row descriptions previously in `AGENTS.md § Agent Docs` reproduced faithfully (move + light restructure permitted; no semantic change). |
| AC2 | `ai-docs/api-naming.md` exists with a first paragraph cross-linking back to `AGENTS.md § API Naming`, the `_unchecked` AXIOM block reproduced verbatim, the per-shape action table reproduced verbatim, and the rule-prose bullets reproduced verbatim (move + light restructure permitted; no semantic change). |
| AC3 | `AGENTS.md § Agent Docs` table retains every existing row's file path + a ≤ 80-char one-line description stub, AND adds a new row for `ai-docs/agent-docs-index.md` AND a new row for `ai-docs/api-naming.md`. Row enumeration is monotonically extended; no row is removed. |
| AC4 | `AGENTS.md § API Naming` keeps the `## API Naming` level-2 heading; the body is collapsed to a one-line stub with an anchored cross-link to `ai-docs/api-naming.md`. The stub link resolves to a real heading slug in the destination. |
| AC5 | Post-extraction `wc -c AGENTS.md` reports **≤ 34,000** chars (strict; ≥ 1,000-char safety margin below the 35,000-char early-warning band). |
| AC6 | Every commit boundary on the feature branch reports `wc -c AGENTS.md < 40,000`. Verified by iterating the branch's commits (`git rev-list <merge-base>..HEAD` + per-commit `git show <sha>:AGENTS.md \| wc -c`) and confirming each is `< 40,000`. AGENTS.md's size is monotonically non-increasing across the branch's commits. |
| AC7 | Every `[…](ai-docs/<topic>.md#anchor)` link introduced into AGENTS.md by this PR resolves — the anchor slug exists as a real ATX heading in the destination file. Verified by `/ai-audit` Phase 2 Step 2.6.4 (or equivalent anchor-aware grep) locally before push. |
| AC8 | Post-merge `/ai-audit` Phase 2 Sub-check 9 emits **zero** findings against AGENTS.md (the pre-extraction `AGENTS.md: 39,960 chars — early warning` finding goes away). Demonstrator evidence (before/after byte counts + Sub-check 9 output) recorded in the PR body. |
| AC9 | Workspace gates PASS green: `cargo build`, `cargo test`, `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. |
| AC10 | Propagation-rule grep (`grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md`) finds zero downstream files referencing an `§ Agent Docs` row body or `§ API Naming` body text that the extraction moved without a corresponding cross-reference update. Specifically: `.claude/skills/next/SKILL.md` continues to find the `§ Agent Docs` rows it consumes (row enumeration unchanged). |
| AC11 | Pre/post extraction diff of each rule body shows only formatting / cross-link prose changes — no rule-text changes. Spot-checked during `self-review`. |

## Open questions

None — Q1 (extraction targets) resolved by round-1 answer; Q2 (destination filenames) resolved to `ai-docs/agent-docs-index.md` + `ai-docs/api-naming.md` per round-2 user guidance; Q3 (commit decomposition) resolved to a two-commit monotonic-decreasing split per round-2 user guidance. Concrete anchor slugs (e.g., `#agent-doc-rows` vs `#agent-doc-row-bodies`) are a design-time call, not a spec-time one.
