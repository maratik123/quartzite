# Considerations: link-reading by `/task`, `/interview`, `design`, `design-review`

**Status:** considerations only — no decision, no rule changes.
**Date:** 2026-05-07.
**Trigger:** session-end discussion after multiple corrections (`#135` Bencher YAML, `#137` `actions/deploy-pages@v4`, registry-query escalation in PR #150) where issue bodies and specs cited stale or invented external content. Question raised: should the early task-stage skills/agents auto-read the GitHub issue bodies they're invoked against, and should they follow the hyperlinks in those bodies?

This page captures the analysis. It is **not** a rule and does **not** modify any skill or agent. Pick a path explicitly before committing.

---

## Current state

| Surface | Reads issue body? | Reads links from body? |
|---|---|---|
| `/task #N` | yes (passes through to `/interview`) | no — orchestrator forwards body verbatim |
| `/interview` | yes (the body seeds predicted answers) | **no** — body is treated as authoritative; links are decorative |
| `design` agent | reads the *spec*, not the original issue | no |
| `design-review` agent | reads spec + design doc | no |

The implicit contract: the issue body and the spec produced from it must be **self-contained**. Links are secondary references; nobody fetches them automatically. The recent canonical examples that broke this contract (`criterion = "0.5"` in `#135`, `actions/deploy-pages@v4` in `#137`) are exactly the failure mode this consideration addresses.

## Cases this session where link-reading would have helped

| Issue | What went wrong | What link-reading would have caught |
|---|---|---|
| `#135` (Bencher CI) | Initial body invented a YAML pattern from training-data memory | One `WebFetch https://bencher.dev/docs/how-to/github-actions/` at interview time would have produced a spec with the official 3-workflow fork-PR pattern from day one |
| `#137` (cargo doc) | Body cited `actions/deploy-pages@v4` (Node 20, deprecated) | One `gh api /repos/actions/deploy-pages/releases` would have given `@v5` |
| `#117` (codegen-simple-marker) | Spec referenced `find_by_path` (phantom API) | A grep against the codebase at design-time caught it; without that diligence, the implementer would have chased a fn that doesn't exist |

The registry-query rule escalated in PR #150 is a special case of "fetch live sources before pinning". The general principle — fetch what the issue body cites — is broader.

## Cases where link-reading would hurt

- **External tutorial blog posts** (50KB+ of mostly-irrelevant prose) — net cost, not net benefit.
- **Chains of cross-referenced GitHub issues** — `#N → #M → #K` can recurse 3–5 levels and produce 30+ KB of context, most of it tangential.
- **Stale archived docs** — the cited link may be a previous version; reading it actively misleads.
- **Marketing pages** linked from "see also" sections — pure overhead.

## By link category — different handling deserves consideration

| Category | Argument for reading | Argument against | Suggestion |
|---|---|---|---|
| **Internal `ai-docs/**` spec/design/learnings** | These are project-canonical | None | **Always read** — already free since they're local files |
| **Cross-referenced GH issues (`#N`)** | Dependency graph, related context | Risk of recursion; usually decorative | Read **title + state** only; full body only on explicit request |
| **External canonical reference docs** (Bencher, GitHub Actions, crates.io API docs) | Required for spec/AC fidelity | Token cost (often 50–80 KB) | **Read once at interview**; embed key excerpts (verbatim YAML snippets, schema tables) into the spec so downstream agents don't re-fetch |
| **External tutorials / blog posts** | Sometimes have the canonical recipe | Often outdated; high noise | Skim title + skip body unless an interview question explicitly hinges on it |
| **Live registry endpoints** (crates.io API, `gh api /releases`) | Version freshness — already an escalated rule | Cheap (<1 KB JSON) | **Always query** when a body cites a specific version string. This rule already exists; could be enforced more aggressively |
| **Source-code links** (rust-lang/rust, docs.rs items) | Verify API surface exists | Small cost | Read on-demand when spec/design references a specific item |

## By agent — different needs

### `/interview`

The earliest stage and the one with **the most asymmetric leverage** for link-reading. If interview embeds external canonical excerpts into the spec, every downstream agent reads them for free (file read vs. WebFetch). If interview skips them, every downstream agent either re-fetches or works from stale cached knowledge.

**Strongest case for adding link-reading.** Specific affordances worth considering:

- Enumerate hyperlinks in the issue body at intake.
- Categorise them (internal / external-canonical / external-tutorial / live-registry / source-code).
- Auto-fetch internal refs and live-registry queries (cheap, high signal).
- Auto-fetch external-canonical when AC items contain phrases like *"per the official … guide"* or *"match the canonical … pattern"*.
- Surface external-tutorial / blog links as *"the body references X — fetch it?"* instead of auto-reading.

### `design` agent

Operates on the spec, not the issue. **Weaker case for link-reading at design time** *if* interview did its job correctly. But as a safety net against interview oversights, reading external-canonical refs cited *in the spec* (not the original issue) is cheap insurance — design is where architectural mistakes calcify.

A specific risk: when the spec cites a link like *"see [official guide]"* but didn't embed the schema, design must either fetch or guess. Forcing design to fetch in that case avoids invented schemas.

### `design-review` agent

Reviews design against a quality checklist. **Strongest case for selective link-reading** when checklist items say *"does the design's YAML match the official Bencher schema?"* — that is *exactly* a "fetch and compare" question. Without fetching, design-review can only check internal coherence, not external fidelity.

In contrast, design-review reading every linked tutorial would be net-negative.

### `/task` (orchestrator)

Doesn't itself read; coordinates the others. The question for `/task`: should it pass forward "fetched and embedded into spec" or "links to follow"? **Embedded is strictly better** — subagent isolation makes link-following expensive (each agent re-fetches).

## Practical considerations (not decisions)

### Where the rule could live

| Target | What it would say |
|---|---|
| `AGENTS.md ## Workflow` | "Issue bodies must be treated as candidate-truth, not authoritative. /interview verifies live sources for cited versions, schemas, and APIs before producing the spec; the spec is what subagents trust." |
| `interview/SKILL.md` | New step before predicted-answers: "Enumerate hyperlinks from issue body. For each: categorise → internal/canonical/tutorial/registry/code. Read internal & registry. Read external-canonical if AC cites them. Embed verbatim excerpts (YAML, schema tables, threshold values) into the spec." |
| `design.md` agent | "Re-fetch any external-canonical link cited in the spec if its excerpts are not embedded inline. Don't trust 'see [link]' to mean 'safe to skip'." |
| `design-review.md` agent | "For checklist items that compare the design against an external reference (Bencher schema, GitHub Actions YAML, crates.io API), fetch the reference and verify match." |

### Where a hook could enforce it

A `PostToolUse` hook on the `gh issue view` tool could:
- Extract hyperlinks from the JSON output.
- Run the registry-query for any version strings (`@v\d+` action refs, `"x.y.z"` semver dep specs).
- Inject the freshness data into stderr as `additionalContext`.

This would mechanize the "fetch live sources" half automatically, leaving link-categorisation and external-doc reading to the agent.

### Token-budget considerations

`WebFetch` returns 50–80 KB for typical canonical doc pages. `/interview`'s working set might already be 30–50 KB (issue body + AGENTS.md + spec template). Adding a fetch doubles that, before the actual interview prose.

Mitigation: `WebFetch`'s `prompt` parameter lets the model summarise on-fetch. Asking for *"extract verbatim YAML examples + the specific table about <X>"* returns 5–10 KB instead of the full page. The current pattern (used in PR #150's research phase) already does this — formalising it as the standard interview affordance is the question.

### Failure modes of auto-fetching

- **WebFetch hits a 403** (bencher.dev did, this session). Fall back to: ask the user to paste, or query the source repo's contents API for raw markdown.
- **The cited link is dead.** Surface to user before silently giving up.
- **The cited link is a paywalled API spec.** Out of reach; flag explicitly in the spec rather than guess.
- **The agent fetches a chain (link A cites B cites C).** Hard cap at depth 1 unless user asks for deeper.

### What this would NOT change

- The append-only nature of the issue body / spec.
- Subagent isolation (no cross-context sharing of fetched content; embed-into-spec is still the path).
- AGENTS.md `## Dependency Versions` registry-query rule (already exists, just gets used more aggressively).

## Bottom line as a tradeoff

| Adding link-reading | Cost | Benefit |
|---|---|---|
| To `/interview` | 1× fetch + embed at intake | Every downstream agent works from canonical schemas; fewer corrective PR rounds; one fewer recurring class of mistake (stale citations) |
| To `design` | Optional safety-net fetch | Catches interview oversights before they calcify |
| To `design-review` | Per-checklist external fetch | Detects "design diverges from canonical schema" findings that internal-only review can't catch |
| To `/task` orchestrator | None directly — just policy choice | Decides where in the chain the fetch lives |

The biggest leverage is at `/interview`. The cheapest enforcement is a hook on `gh issue view` that injects freshness data. The most invariant-preserving framing: *"the spec is the source of truth; /interview's job is to make the spec authoritative."*

These are considerations, not a recommendation. Worth picking one path before committing — they trade off against each other.

## Next-step paths (if pursued)

In rough increasing order of disruption:

1. **No change.** Keep the current convention; treat the `#135` / `#137` failure mode as acceptable cost. The registry-query rule already covers the highest-frequency sub-case.
2. **Hook only.** Add a `PostToolUse` hook on `gh issue view` (and possibly `gh pr view`) that runs the registry-query against any version strings in the output and injects freshness data as `additionalContext`. Mechanises the easiest half. Doesn't address external-canonical-doc reading.
3. **`/interview` only.** Update `interview/SKILL.md` with a Step-0 link-enumeration pass and embed-into-spec affordance. Doesn't touch `design` or `design-review`.
4. **All four surfaces.** Update `/task`, `/interview`, `design`, `design-review` per the table above. Largest behavioural shift; biggest reduction in corrective rounds.

Each path is internally consistent. Path 1 accepts ongoing cost; path 4 invests upfront. Paths 2 and 3 are intermediate.
