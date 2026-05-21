# Design: Rustdoc strip internal refs

**Issue:** #336
**Date:** 2026-05-21

## Approach

Six concerns, treated as one PR because each one is small and the regression-gate / convention codification / review-gate propagation are only meaningful once the in-tree leaks are gone:

1. **Drop the ten Family-A / Family-B leaks** by mechanically rewriting each doc-comment to be self-contained (intra-doc link, plain prose, or sentence-dropped) per the Scope §3 + Key-decisions matrix in the spec. Round-1 design-review surfaced two additional Pattern-B hits at `quartzite-core/src/snapshot.rs:138` and `quartzite-core/src/snapshot/tree.rs:69` (both `\bthis implementation\b` matches in published-surface `///` lines outside `#[cfg(test)]`). They are addressed by additional rows in the replacement-prose table below — Scope §3 is the canonical-as-of-spec-pickup list, but Scope §5 mandates re-running both audit patterns after edits and zero published-surface matches at completion (AC1); the design therefore rewrites every published-surface hit the patterns surface, not only the originally-enumerated eight. No new tests — `cargo doc --no-deps --workspace --all-features` under `-D warnings -D missing-docs` is the structural gate; the audit-pattern re-runs are the prose gate (AC1).
2. **Exclude `quartzite-test-helpers` from the published-rustdoc surface** by adding `[lib] doc = false` to its `Cargo.toml`. Single-line manifest change — no CI workflow edit, no AGENTS.md doc-gate command change, and an explicit Out-of-scope note in the spec.
3. **Codify the rule in `ai-docs/doc-convention.md`** by appending one subsection under `## Scope`, immediately before `## Section order (strict)`. Subsection cites both audit patterns inline (terse fenced code blocks) so a doc author or review agent can re-run them locally without chasing a link out to the spec.
4. **Propagate the gate to the Review group + design-review** by adding one bullet to `self-review.md` §6, one bullet to `review-findings.md` §6, one bullet to `code-review/SKILL.md` `## Gate checklist`, and one checklist item to `design-review.md` §3. Each entry links into the new `doc-convention.md` anchor — keeps the four review-gate files terse and below the 35k cap (AC13).
5. **Ship the regression-prevention CI gate in this PR**, not as a fast-follow — decision below.
6. **Land the regression script at `scripts/check-rustdoc-internal-refs.sh`** consistent with the existing three shell scripts in `scripts/`. Wire it into `.github/workflows/ci.yml`'s `docs:` job (which runs on both `push` and `pull_request`, so PR CI exercises it before merge — `docs.yml` is master-only and would not catch leaks pre-merge). One `actionlint` re-run on the modified workflow file is mandatory before commit per AGENTS.md.

### Key decision row 1 — gate ships in this PR (not fast-follow)

**Decision: ship the CI gate in this PR.**

Rationale (three legs):

- **Symmetry with the convention codification.** Scope §8 codifies the rule in `doc-convention.md` and Scope §9 codifies it in the review gates in this same PR. A fast-follow for the CI gate leaves the only mechanical (non-human, non-review-agent) enforcement layer absent at merge — which is exactly the window when the next PR can re-introduce a leak. The cost of the gate is one ~30-line shell script + one workflow step + one workflow-file `actionlint` re-run; the cost of a fast-follow PR is a duplicate spec → design → review → merge cycle and a coverage gap in between.
- **No new dependencies.** The script is `ripgrep` (already a contributor tool, but the ci.yml runner has `rg` available via the `actions-rust-lang/setup-rust-toolchain@v1` step's environment; the script falls back to `grep -E` if `rg` is missing — see *Subtask 5* below). No new actions, no new toolchain pins.
- **The fast-follow exception in the issue body was for "evaluate cost vs value".** Cost is one file + one workflow step. Value is regression prevention for a class of leak that has already recurred once (PR #515 missed the two `dark_palette.rs` lines that this PR cleans up). The cost-vs-value tilt is clearly toward shipping it now.

If the CI gate fails its own self-review or `actionlint` (or if subtasks 1–4 reveal scope creep), the gate subtask is the natural cut point — fold the work into a follow-up issue without unwinding the doc rewrites. See *Risks* below.

### Replacement prose for each Scope §3 entry

Wording chosen by reading each file in context (see investigation log in the doc this was generated from). Implementer is free to adjust the surface of each rewrite, but the **information content** below must be preserved (AC4 — no information loss for downstream readers, only loss of internal-process pointers):

| File | Current text (cited fragment) | Replacement strategy |
|---|---|---|
| `quartzite-style-dispatch/src/dispatch.rs:60–62` | "If this blanket impl turns out unusable in real code, it can be removed without breaking callers that implement the trait directly (pre-publish, per `AGENTS.md` § *API Stability*)." | Drop the parenthetical clause + drop the entire sentence. The substantive content ("blanket impl may be unusable in real code; callers implementing the trait directly are unaffected") is downstream-irrelevant — it is a contributor-facing rationale about the project's API-stability posture. The preceding paragraph ("Lifetime caveat") already conveys why the impl is shaky for a downstream reader. |
| `quartzite-core/src/snapshot.rs:27` | `` `signals_blocked` flag \| **Reset to `false`.** Persisting this state is tracked in [#39](https://github.com/maratik123/quartzite/issues/39). \| `` | Drop the trailing sentence. Replacement row: `` `signals_blocked` flag \| **Reset to `false`.** Callers that need to preserve this state must re-set it after restore. \| `` — i.e. behaviour content stays; the tracking-issue pointer leaves. |
| `quartzite-widgets/src/widget_base.rs:100` | `/// [`ObjectId`]s of installed event filters (dispatch deferred to plan #47).` | Replacement: `/// [`ObjectId`]s of installed event filters. Dispatch through these filters is not yet implemented; the field is reserved for future use.` — downstream reader sees the same field semantics and learns the field is currently inert, without the internal "plan #47" pointer. |
| `quartzite-widgets/src/layout/mod.rs:16–17` | "Implemented by the renderer's `ObjectTree` wrapper during layout passes (plan #47). For unit tests, …" | Drop the `(plan #47)` parenthetical. Replacement: "Implemented by the renderer's `ObjectTree` wrapper during layout passes. For unit tests, a `HashMap<ObjectId, WidgetBase>`-backed stub is used instead." — sentence is otherwise unchanged. |
| `quartzite-renderer/src/render_harness.rs:300–306, 312–313` | "**Trait-bound finalisation** (per spec AC1's "or equivalent" escape hatch): the harness takes a closure rather than a [`WidgetExt`] bound, …" + "The widget-specific shorthand is provided by the test-side helper (`tests/support/mod.rs`)." | Two edits in the same doc comment. (i) Drop the "(per spec AC1's "or equivalent" escape hatch)" parenthetical and strip the bold "**Trait-bound finalisation**" prefix (it was a spec-AC label, not a heading). Replacement opening: "The harness takes a closure rather than a [`WidgetExt`](https://docs.rs/quartzite-widgets/latest/quartzite_widgets/trait.WidgetExt.html) bound, because `WidgetExt` lives in `quartzite-widgets` and that crate is the renderer's *dev-dependency* — taking the bound directly would close a regular dependency cycle." (ii) Drop the `(`tests/support/mod.rs`)` parenthetical at lines 312–313 (repo-internal test-fixture path; not on docs.rs). Replacement tail: "The widget-specific shorthand is provided by the test-side helper." Documented as **Scope §3-adjacent — caught by the spec's Scope §1 forbidden-paths list (Family A — `tests/` is a repo-internal directory not present in published rustdoc) even though Pattern A's regex does not include a `\btests/\b` token.** Per the design's reading of round-1 review (Issue 3), the regex-token gap is recorded as a known limitation in the Risks section rather than fixed via a spec amendment to `## Audit patterns`. |
| `quartzite-core/src/snapshot.rs:138` | `/// The maximum version supported by this implementation ([\`CURRENT_SCHEMA_VERSION\`]).` | Drop "this implementation" without information loss — the constant being cited *is* the cap. Replacement: `/// The maximum [\`u32\`] version this enum variant signals as supported; equal to [\`CURRENT_SCHEMA_VERSION\`].` Rationale: the field is the `supported` field of `DeserializeError::UnsupportedVersion` (a `u32`); naming the type and routing the reader directly to the const preserves the behavioural claim ("the cap is `CURRENT_SCHEMA_VERSION`") without the "this implementation" coupling-to-development-process token. Verified: rewrite does not match Pattern A (no internal-artefact tokens, no `\b#[0-9]{1,4}\b` collision) or Pattern B (no `\bthis implementation\b`, `cargo *`, `scripts/`, etc.). Also reword the sibling `#[error(...)]` attribute on the same variant (`"unsupported schema version {found} (this implementation supports up to {supported})"`) to drop "this implementation" — the `#[error]` line is not matched by Pattern B's `^\s*(///\|//!)` anchor, but the rendered `Display` output IS the downstream-visible text and the same self-sufficiency principle (AC4) applies. Sample reword: `"unsupported schema version {found} (supported up to {supported})"`. |
| `quartzite-core/src/snapshot/tree.rs:69` | `/// Validates that \`self.schema_version\` is supported by this implementation.` | Drop "this implementation" by pinning the cap to the named const. Replacement: `/// Validates that \`self.schema_version\` is at or below [\`CURRENT_SCHEMA_VERSION\`](crate::snapshot::CURRENT_SCHEMA_VERSION).` Rationale: the method body (lines 98–101 of the same file) literally compares `self.schema_version > crate::snapshot::CURRENT_SCHEMA_VERSION` — naming the const in the doc-comment makes the behavioural contract explicit and removes the development-process coupling. The intra-doc link resolves under `[workspace.lints.rustdoc] broken_intra_doc_links = "deny"` (the existing `# Errors` section on lines 73–74 already uses the same path). Verified: rewrite does not match Pattern A or Pattern B. |
| `quartzite-style-types/src/dark_palette.rs:5` | "…seeded from the dark-theme values defined in `design-system/README.md` § *Dark theme*." | Drop the `design-system/README.md § *Dark theme*` clause. Replacement: "…seeded from the dark-theme values defined by this crate's constants." The constant's RGBA values ARE the dark-theme specification from a downstream reader's perspective — the citation is repo-internal and unactionable. |
| `quartzite-style-types/src/dark_palette.rs:12` | "Every [`ColorRole`] slot is set to the RGBA value that `design-system/README.md` § *Dark theme* specifies for that role, converted from sRGB hex to 3-decimal linear floats." | Drop the `design-system/README.md § *Dark theme*` citation. Replacement: "Every [`ColorRole`] slot is set to a dark-theme RGBA value converted from sRGB hex to 3-decimal linear floats." Same downstream-reader reasoning. |
| `src/lib.rs:203–204` | "…disable the `std` feature for the `no_std + derive`-free path. Verify locally with `cargo build -p quartzite --no-default-features`." | Drop the final sentence entirely (per Key-decisions row: spec explicitly says "no replacement prose needed"). The preceding clause "disable the `std` feature for the `no_std + derive`-free path" already conveys the behavioural content. |

### Doc-convention.md subsection

Insertion point: between current line 17 (end of `## Scope` "Does NOT apply to" list) and current line 19 (start of `## References` `## ` header). New subsection title: **`### Self-sufficiency: no repo-internal references`** (h3 under the existing `## Scope` h2 — parallel to no-explicit-h3 "Applies to / Does NOT apply to" sublist but promoted to h3 to give the audit-patterns block a stable named anchor for the four review-gate files to link to per AC12).

Proposed body (terse — keeps `doc-convention.md` under the 35k cap; final char delta target ≤ 2 kB):

```markdown
### Self-sufficiency: no repo-internal references

Every `///`, `//!`, and `#[doc = "..."]` doc-comment in the published rustdoc surface must stand alone for a downstream reader on docs.rs. Two families of repo-internal references are forbidden:

- **Family A — internal-artefact citations.** GitHub issue / PR numbers (`#NN`, `github.com/.../issues/N`); repo-internal paths (`ai-docs/...`, `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `design-system/...`, `.claude/...`, `scripts/...`, `.github/...`); internal-artefact references ("plan #N", "spec AC<n>", "the X spec", "tracked in", "deferred to a future plan / spec / follow-up", "per AGENTS.md"). Replace with self-contained behaviour wording, an intra-doc link to an in-workspace item, or a docs.rs / external-spec URL.
- **Family B — contributor-tooling instructions.** Verify-locally / how-to-verify command incantations directed at a contributor (`cargo build -p X`, `cargo test`, `cargo clippy`, `RUSTDOCFLAGS=...`, `cargo doc --no-deps`); references to repo-internal scripts (`scripts/<name>.sh`) or workflow files (`.github/workflows/...`); coupling-to-the-development-process language ("this PR", "this commit", "this implementation"). Drop the sentence or replace with a worked API-use example.

**Non-conforming (Family A):** `/// Every ColorRole slot is set to the RGBA value that design-system/README.md § Dark theme specifies for that role.`
**Conforming:** `/// Every ColorRole slot is set to a dark-theme RGBA value converted from sRGB hex to 3-decimal linear floats.`

**Non-conforming (Family B):** `//! Verify locally with cargo build -p quartzite --no-default-features.`
**Conforming:** *(drop the sentence; surrounding prose already conveys the behavioural content.)*

**Local enforcement.** Two ripgrep audits scoped to `--type rust` excluding `tests/`, `benches/`, `quartzite-test-helpers/src/**`, and `#[cfg(test)]` regions:

```bash
# Pattern A — internal-artefact citations
rg --type rust -n '^\s*(///|//!).*(\bissue #[0-9]|\bPR #[0-9]|github\.com/.+/(issues|pull)/|ai-docs/|AGENTS\.md|CLAUDE\.md|CONTRIBUTING\.md|design-system/|\.claude/|\bspec AC[0-9]|\bplan #[0-9]|tracked in|deferred to a future (plan|spec)|\b#[0-9]{1,4}\b)'

# Pattern B — contributor-tooling instructions
rg --type rust -n '^\s*(///|//!).*(\bVerify locally|\bcargo build -p|\bcargo test\b|\bcargo clippy\b|\bcargo fmt\b|RUSTDOCFLAGS|cargo doc --|scripts/[a-z]|\bthis PR\b|\bthis commit\b|\bthis implementation\b)'
```

Both must return empty against the published surface. CI runs them via `scripts/check-rustdoc-internal-refs.sh` (wired into `.github/workflows/ci.yml`'s `docs:` job).
```

Measured delta: 2,648 bytes (verbatim `wc -c` on the body block above). Pre-edit baseline: 27,595 (AC13). Post-edit `wc -c` projection for `doc-convention.md`: 30,243 — comfortably under the 35k cap (AC13).

### Review-gate edits

Each edit ≤ 200 chars per file — comfortably preserves the AC13 headroom on the five instruction files.

1. **`.claude/agents/self-review.md` §6 Documentation** — append one bullet after line 121 (`document_features` rendering bullet):

   > - **No repo-internal references in doc-comments** ([`ai-docs/doc-convention.md` → Self-sufficiency: no repo-internal references](../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references)). For every `///` / `//!` / `#[doc = "..."]` line added or modified by this diff in a non-test, non-`quartzite-test-helpers` source file, re-run Pattern A and Pattern B from the linked subsection. Any match → REJECT (`major`) with the matched line and the family.

2. **`.claude/agents/review-findings.md` §6 Documentation** — append one bullet after the current `Feature-gated documentation sync` bullet (line 100):

   > - **No repo-internal references in doc-comments** ([`ai-docs/doc-convention.md` → Self-sufficiency: no repo-internal references](../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references)). Run Pattern A and Pattern B (regexes in the linked subsection) across all workspace `src/**` excluding `tests/`, `benches/`, `quartzite-test-helpers/src/**`, and lines inside `#[cfg(test)]` regions. Any non-test match → `major`.

3. **`.claude/skills/code-review/SKILL.md` `## Gate checklist`** — the existing checklist is a **table** with `| Before | Check |` columns (rows: Step 2 / Step 3 / Step 4 / Step 5 / Commit), not a bullet list. Verified by reading the file. Per round-1 review (Issue 2, Option a — preferred for structural consistency), append **one new row** to the table, slotted between the existing `Step 4` row and `Step 5` row so the doc-convention gate sits adjacent to the other Step-4-tier "convention" check:

   > `| Step 4 | doc-comments in changed published-surface files do not match Pattern A / Pattern B from [doc-convention](../../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references) (the standard `review-findings.md` §6 finding fires when violated) |`

   Rationale for table-row over prose-subsection (Option b): the existing `Step 4` row already cites the doc-convention gate in compressed form ("all six checks pass (build, test, clippy, fmt, doc, doc convention)"); a parallel row keeps the table the single source of truth for "checks at each step" and avoids splitting the gate's surface between the table and a new prose subsection.

4. **`.claude/agents/design-review.md` §3 Actively check the checklist** — append one bullet after the `Design-system visual rules` item (line 34):

   > - **No repo-internal references in planned doc-comment text** — when the design document contains inline rustdoc snippets (e.g. proposed `///` / `//!` text for an API), scan those snippets with Pattern A and Pattern B from [`ai-docs/doc-convention.md` § Self-sufficiency](../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references). Any match in a planned doc-comment block = `major`; matches in design-doc prose outside `///` / `//!` blocks are out of scope (the design doc is contributor surface, not rustdoc).

Each link uses the same named anchor `#self-sufficiency-no-repo-internal-references` so `rg -l "self-sufficiency-no-repo-internal-references" .claude/ ai-docs/doc-convention.md` returns five files (the four review-gate files + the convention itself) — satisfies AC12's "grepping any of the four files for a shared named anchor returns a hit in each file".

Post-edit `wc -c` projection (pre-edit baseline → projected post-edit total):

| File | Pre-edit | Edit shape | Projected post-edit |
|---|---|---|---|
| `ai-docs/doc-convention.md` | 27,595 | new subsection, body measured 2,648 B | 30,243 |
| `.claude/skills/code-review/SKILL.md` | 9,293 | one new table row (~210 B incl. rationale already in design doc; row text ~190 B in-file) | ~9,490 |
| `.claude/agents/review-findings.md` | 20,543 | one new bullet (~620 B) | ~21,160 |
| `.claude/agents/self-review.md` | 23,605 | one new bullet (~600 B) | ~24,200 |
| `.claude/agents/design-review.md` | 5,475 | one new bullet (~590 B) | ~6,070 |

All five files comfortably under the 35k early-warning cap (AC13). **AC13 re-confirmation (round 3):** the round-3 changes touch only `scripts/check-rustdoc-internal-refs.sh` (a new file, not an instruction file under the 35k cap) and this design document; none of the five instruction-file deltas above are altered. Pre-edit baselines and projected post-edit totals stand unchanged.

### Regression script + CI wiring

**Script: `scripts/check-rustdoc-internal-refs.sh`** — shape:

- Bash, `set -euo pipefail`, header comment naming the spec date.
- **Header documents the three known `#[cfg(test)]`-enclosed false-positive sites in-tree at design time as worked examples** (so the next contributor reading the script sees real workspace shapes): `quartzite-style/src/default_style_tests.rs:2` (`#[cfg(test)] #[path = "default_style_tests.rs"] mod tests;` shape attaching a sibling file), `quartzite-runtime/src/timer_drivers.rs:450` (inline `#[cfg(test)] mod tests { ... }` shape), `quartzite-renderer/src/render_harness.rs:545` (inline `#[cfg(test)] mod tests { ... }` shape).
- `rg` preferred; fall back to `grep -E -nR` if `rg` is unavailable (defensive — CI's Rust toolchain step pins `rg` via the dev shell but local pre-commit invocation might lack it).
- Walks `**/*.rs` excluding `tests/`, `benches/`, `quartzite-test-helpers/src/`, and `target/`. Path exclusion alone does NOT catch the three `#[cfg(test)]`-enclosed false positives above (`default_style_tests.rs` lives at `src/default_style_tests.rs`, not under `tests/`; the other two are inline `#[cfg(test)] mod tests` blocks inside `src/` files). Per round-3 design-review, the gate adds a **`#[cfg(test)]`-region filter (option (b) — backward-scan heuristic)** as a post-processing pass over `rg`'s raw hits.
- Runs Pattern A and Pattern B in two separate `rg` invocations into a combined hit list.
- For each hit `<file>:<line>:<match>`, the backward-scan heuristic decides whether the hit is inside a `#[cfg(test)]` region. If it is, the hit is dropped; otherwise it is retained as a real violation.
- Exits `1` if any retained hit remains; prints the offending `file:line` prefix and the matched text.

**Backward-scan heuristic for the `#[cfg(test)]`-region filter (option (b)).** The script applies a per-file pass: for each `rg` hit at `<line>` in `<file>`, read `<file>` lines `1..<line>` and decide enclosure-state from a single scan. Two file-level shapes are handled:

1. **Inline `#[cfg(test)] mod tests { ... }` shape** (timer_drivers.rs, render_harness.rs). Walk the prefix once tracking brace depth and the most-recent `#[cfg(test)]` attribute. A hit is `#[cfg(test)]`-enclosed iff there exists a preceding line of the form `#[cfg(test)]` followed (after any number of additional attribute lines) by `mod <name> {` whose opening `{` is at brace-depth 0, AND the running brace depth at the hit line is strictly greater than the brace depth at that `mod <name> {` line.

2. **Sibling-file attachment shape** (`#[cfg(test)] #[path = "default_style_tests.rs"] mod tests;`, default_style_tests.rs). The hit is in a file at `<crate>/src/<name>.rs` whose name does NOT match the parent module's expected `mod <name>;` lookup path — the heuristic detects this by checking, for the hit's file `<file>`, whether any sibling `.rs` file in the same directory contains a `#[cfg(test)] #[path = "<basename>"] mod <ident>;` line referring to `<file>`. If yes, the entire file is treated as a `#[cfg(test)]` region (every hit in it dropped).

Pseudocode sketch (bash + awk):

```bash
# Combined hit list from Patterns A and B
hits=$(rg --type rust -n "$PATTERN_A|$PATTERN_B" \
  -g '!tests/**' -g '!benches/**' \
  -g '!quartzite-test-helpers/src/**' -g '!target/**')

# Build the set of sibling-attached test files once (shape 2 detection)
sibling_attached_files=$(rg --type rust -n \
  '#\[cfg\(test\)\][[:space:]\n]*#\[path[[:space:]]*=[[:space:]]*"[^"]+\.rs"\][[:space:]\n]*mod[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*;' \
  --multiline | awk -F'"' '{print $2}' | sort -u)

retained=()
while IFS=: read -r file line _; do
  # Shape 2: whole-file skip
  if grep -Fxq "$(basename "$file")" <<<"$sibling_attached_files"; then
    continue
  fi
  # Shape 1: scan prefix lines 1..line, track brace depth + last-#[cfg(test)] mod position
  enclosed=$(awk -v target="$line" '
    BEGIN { depth = 0; cfg_test_mod_depth = -1; saw_cfg_test = 0 }
    NR > target { exit }
    /^[[:space:]]*#\[cfg\(test\)\]/ { saw_cfg_test = 1; next }
    /^[[:space:]]*mod[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*\{/ {
      if (saw_cfg_test && depth == 0) { cfg_test_mod_depth = depth }
      saw_cfg_test = 0
    }
    { saw_cfg_test = 0 }
    { for (i = 1; i <= length($0); i++) {
        c = substr($0, i, 1)
        if (c == "{") depth++
        else if (c == "}") { depth--; if (cfg_test_mod_depth >= 0 && depth <= cfg_test_mod_depth) cfg_test_mod_depth = -1 }
      } }
    END { print (cfg_test_mod_depth >= 0 ? "yes" : "no") }
  ' "$file")
  [[ "$enclosed" == "no" ]] && retained+=("$file:$line")
done <<<"$hits"

if (( ${#retained[@]} > 0 )); then
  printf '%s\n' "${retained[@]}" >&2
  exit 1
fi
```

The above is a **sketch, not a literal copy-paste**; the implementer is free to adjust awk syntax / brace-depth bookkeeping / multi-line attribute handling as long as the three known sites are filtered AND a deliberate violation introduced into a non-`#[cfg(test)]` file is still caught (AC7).

**Sketch corrections to apply at implementation time.** Round-3 design review surfaced three concrete bugs in the pseudocode above. The implementer MUST apply these fixes when authoring the script (subtask 5) — the sketch alone would not generalise / would not compile / would silently misbehave:

- **Shape-1 awk — guard the `saw_cfg_test` reset.** Block 3 (`{ saw_cfg_test = 0 }`) currently runs unconditionally on every line, which resets `saw_cfg_test` on intermediate attribute lines (e.g. `#[path = "..."]` interposed between `#[cfg(test)]` and `mod tests`). The workspace has no inline-shape with intermediate attributes today, but the sketch as written would not generalise — a future contributor adding `#[cfg(test)] #[other_attr] mod tests { ... }` would see the inline-shape detection silently fail. Fix: guard the reset so it fires only on lines that are neither `#[...]`-attribute-shaped nor blank (i.e. lines matching `^\s*[^#\s]`) — typical awk form: `/^[[:space:]]*[^#[:space:]]/ { saw_cfg_test = 0 }` as block 3, replacing the unconditional reset.
- **Shape-2 regex — drop the literal `\n` inside the POSIX bracket class.** The current `[[:space:]\n]*` treats `\n` as the two literal characters `\` and `n` (a POSIX bracket class is not a backslash-escape context). `[:space:]` already covers newline. Fix: replace both occurrences with `[[:space:]]*` in the shape-2 detection regex. Additionally verify at implementation time that `rg --multiline` actually returns multi-line matches for the corrected regex — `rg`'s `--multiline` flag changes line-boundary semantics, and the implementer should confirm against one of the three known sites before relying on it.
- **Option-(a) fallback — narrow the per-file baseline check to non-`#[cfg(test)]` matches.** The Risks-section fallback says "any increase fails the gate" using raw `rg "$PATTERN_A|$PATTERN_B" <file> | wc -l`. This conflates two cases: a NEW test-side hit (allowed per AC5 — `#[cfg(test)]`-enclosed lines are exempt) and a regression escape (forbidden — new published-surface leak in a previously-clean file). Fix: if option (a) is selected during subtask-5 implementation, narrow the baseline check to count only matches NOT preceded by `#[cfg(test)]` — apply a slimmer shape-1 awk to each of the three skipped files (the same backward-scan-for-`#[cfg(test)]`-enclosure that option (b) uses, but scoped to one file at a time, which is cheaper) and compare the post-filter count against the baseline. **Alternative — accept the over-restriction.** Document in the script header that option (a)'s per-file baseline check rejects ANY new match including test-side ones, treating it as graceful-degradation noise (the AC5 exemption is forfeited for those three files only); future test-side edits to those files require re-baseline. Pick one of the two at implementation time and write the choice into the script header.

**Implementation-limitation note for the script header.** This is a heuristic, not a full Rust parser. The script handles the two shapes enumerated above (`#[cfg(test)] mod tests { ... }` inline and `#[cfg(test)] #[path = "..."] mod tests;` sibling-file attachment), because those are the only two shapes present in the workspace at design time. More exotic shapes — `#[cfg(any(test, feature = "bar"))]`, nested `#[cfg(test)]` inside another `cfg`, doc-comments inside `cfg(test)` *expression* blocks rather than `mod` blocks — would need follow-up if they appear. Document this in the header so future contributors do not assume parser-grade coverage.

**Fallback to option (a) if the heuristic exceeds the script-complexity budget.** If during subtask-5 implementation the backward-scan logic balloons past ~60 lines of bash/awk (vs. the ~30 LOC sketch above) or the awk-brace-depth tracking proves fragile, fall back to option (a) — a hardcoded skip-list naming the three sites verbatim. In that case the script MUST also add a small CI assertion that the three skipped files have not gained NEW post-skip hits (e.g. a per-skipped-file baseline check: `rg "$PATTERN_A|$PATTERN_B" <file> | wc -l` against an in-script baseline count for each of the three files; any increase fails the gate). This protects AC7's contract for new violations in previously-clean test-file regions. See the corresponding Risks bullet.

**CI wiring: one new step in the `docs:` job of `.github/workflows/ci.yml`.** Placed immediately after the existing `cargo doc` step so a `cargo doc` failure (broken intra-doc link from the rewrites) surfaces before the gate failure. The step invokes `scripts/check-rustdoc-internal-refs.sh`, which applies Pattern A + Pattern B and the `#[cfg(test)]`-region filter (option (b) — backward-scan, per *Backward-scan heuristic* above) before deciding pass/fail. On the post-subtask-1 tree the gate exits 0 only because the filter strips the three known `#[cfg(test)]`-enclosed hits (`default_style_tests.rs:2`, `timer_drivers.rs:450`, `render_harness.rs:545`).

```yaml
- name: Check rustdoc has no repo-internal references
  run: bash scripts/check-rustdoc-internal-refs.sh
```

`docs.yml` (master-only Pages deploy) is **not** modified — the gate's purpose is pre-merge regression prevention; running it on master push is redundant once the PR gate passes. `actionlint .github/workflows/ci.yml` must re-pass after the step is added (AGENTS.md AXIOM).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rewrite all ten published-surface doc-comment leaks per the replacement-prose table above: the eight Scope §3 entries + the two round-1-design-review additions (`snapshot.rs:138`, `snapshot/tree.rs:69`) + the in-table near-miss strip on `render_harness.rs:312–313` (`tests/support/mod.rs` parenthetical). **Subtask 1 ALSO rewrites the adjacent `#[error("…")]` attribute on `quartzite-core/src/snapshot.rs:134`** (sibling of the rewritten doc-comment on line 138 — same `\bthis implementation\b` token in `"unsupported schema version {found} (this implementation supports up to {supported})"`). Although Pattern B's `^\s*(///\|//!)` anchor does not match the `#[error(...)]` attribute line, the rendered `Display` output IS downstream-visible — `thiserror::Error::source()` surfaces it to callers — so AC4's "no information loss for downstream readers" principle applies identically. Sample reword: `"unsupported schema version {found} (supported up to {supported})"`. Re-run both audit patterns scoped to the published surface and verify zero matches. | `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-core/src/snapshot.rs`, `quartzite-core/src/snapshot/tree.rs`, `quartzite-widgets/src/widget_base.rs`, `quartzite-widgets/src/layout/mod.rs`, `quartzite-renderer/src/render_harness.rs`, `quartzite-style-types/src/dark_palette.rs`, `src/lib.rs` | — |
| 2 | Add `[lib] doc = false` to `quartzite-test-helpers/Cargo.toml`. Run `cargo doc --no-deps --workspace --all-features` under `-D warnings -D missing-docs` and confirm `target/doc/quartzite_test_helpers/` is absent and the `target/doc/index.html` redirect target still resolves. | `quartzite-test-helpers/Cargo.toml` | 1 |
| 3 | Add the `### Self-sufficiency: no repo-internal references` subsection to `ai-docs/doc-convention.md` (under `## Scope`, before `## Section order (strict)`) per the body proposed in *Approach → Doc-convention.md subsection*. Verify `wc -c` stays under 35k. | `ai-docs/doc-convention.md` | 2 |
| 4 | Propagate the gate to the four review-gate files per *Approach → Review-gate edits*. Verify each file's `wc -c` stays under 35k. Verify `rg -l "self-sufficiency-no-repo-internal-references"` returns the four review-gate files + `doc-convention.md`. | `.claude/agents/self-review.md`, `.claude/agents/review-findings.md`, `.claude/skills/code-review/SKILL.md`, `.claude/agents/design-review.md` | 3 |
| 5 | Author `scripts/check-rustdoc-internal-refs.sh` (Bash, `set -euo pipefail`, header lists the three known `#[cfg(test)]`-enclosed false-positive sites as worked examples — `default_style_tests.rs:2`, `timer_drivers.rs:450`, `render_harness.rs:545`). The script runs both patterns scoped to the published surface, applies the option-(b) backward-scan `#[cfg(test)]`-region filter (handling both inline `#[cfg(test)] mod tests { ... }` and sibling-file `#[cfg(test)] #[path = "..."] mod tests;` shapes), and exits non-zero on any retained match. Verify it produces zero hits locally against the rewritten tree **after the `#[cfg(test)]`-region filter strips the three known false positives**. If the backward-scan heuristic balloons past ~60 LOC during implementation, fall back to option (a) (hardcoded skip-list + per-file baseline new-hit assertion) per the Risks bullet. | `scripts/check-rustdoc-internal-refs.sh` | 1 |
| 6 | Wire the script into `.github/workflows/ci.yml`'s `docs:` job as one new step immediately after `cargo doc`. Run `actionlint .github/workflows/ci.yml` and verify clean. Demonstrate the gate by introducing one Pattern-A and one Pattern-B violation **in a non-`#[cfg(test)]` file** (e.g. a transient `///` line in one of the eight subtask-1 files) so the demo exercises the gate's pattern-matching path, not its `#[cfg(test)]`-region skip path; run the script, observe the failure, then revert (AC7). | `.github/workflows/ci.yml` | 5 |

Total: 6 subtasks. All atomic.

## Handoff plan

Two groups (3 + 3), final group is terminal (size 3, within the `1..=3` range).

- **Handoff at entry to Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) — every-group handoff contract; the parent `/task` Step 8 binds this at the start of the first group as well.
- **Group A:** subtasks 1–3 — published-surface rewrites (1), `quartzite-test-helpers` exclusion (2), doc-convention.md codification (3). Establishes the in-tree clean state + the canonical convention text that the review gates will link into in Group B.
- **Handoff between Group A and Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context — the next group's review-gate edits, regression script, and `actionlint` re-run benefit from a clean slate after the rewrite + manifest + convention changes in Group A.
- **Group B:** subtasks 4–6 — review-gate propagation (4), regression script (5), CI wiring + `actionlint` + AC7 demo (6). Terminal group (3 subtasks; within the 1..=3 range).

## Risks

- **`cargo doc` regression from a rewrite that drops the intra-doc link a downstream reader relied on.** Mitigation: each replacement-prose row in *Approach* preserves any intra-doc-link target the original cited (or drops only the prose around it); subtask 1 re-runs `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` and fails on warnings.
- **`[lib] doc = false` accidentally removes the crate from `cargo test --workspace` or breaks dev-dependents.** Mitigation: AC10 explicitly re-runs `cargo test --workspace` after the manifest change. Per Cargo docs, `doc = false` affects only the `cargo doc` target, not `cargo test` / `cargo build` — but subtask 2 verifies empirically rather than relying on the doc.
- **Audit pattern false positive from a legitimate `#NN` reference in unmodified prose** (e.g. an existing colour code that already passes the `{1,4}` digit bound + word boundary). Mitigation: subtask 1 re-runs both patterns *after* the rewrites and reads each remaining match in context per the spec's *False-positive notes*. The Pattern-A `\b#[0-9]{1,4}\b` defence-in-depth token was verified at spec time against the workspace — only the three known Family-A hits surface; subtask 1 confirms this still holds.
- **`actionlint` complains about the new CI step** (shell-syntax, env-var quoting). Mitigation: subtask 6 runs `actionlint .github/workflows/ci.yml` mandatorily before `git add` per AGENTS.md AXIOM; failures are fixed in place.
- **Regression script's `#[cfg(test)]` blind spot — known false positives at design time.** Path exclusion alone (`tests/`, `benches/`, `quartzite-test-helpers/src/`, `target/`) does NOT catch three `#[cfg(test)]`-enclosed false-positive sites that live under `src/`: `quartzite-style/src/default_style_tests.rs:2` (Pattern A — `AGENTS.md`, sibling-file shape attached via `#[cfg(test)] #[path = "..."] mod tests;` in `default_style.rs:267-269`), `quartzite-runtime/src/timer_drivers.rs:450` (Pattern B — `cargo test`, inline `#[cfg(test)] mod tests { ... }` shape at line 425), and `quartzite-renderer/src/render_harness.rs:545` (Pattern B — `cargo test`, inline `#[cfg(test)] mod tests { ... }` shape at line 442). These are explicitly named in the spec's *Out-of-scope* §2 (the first only as the third site of the same shape — the spec enumerates only two, but the design treats `default_style_tests.rs:2` identically) and AC5. **Mitigation: option (b) — backward-scan heuristic** (see *Backward-scan heuristic for the `#[cfg(test)]`-region filter* in Approach). The heuristic handles both shapes (inline `#[cfg(test)] mod` and sibling-file `#[cfg(test)] #[path = "..."] mod`) and is preferred over option (a) hardcoded skip-list because AC7 requires that a new violation introduced into one of the three skipped files still triggers the gate — a hardcoded skip-list would silently swallow it. Documented as heuristic (not parser-grade); exotic `cfg` shapes (`cfg(any(test, feature = "bar"))`, nested `cfg`) are explicit follow-up rather than in-scope.
- **Fallback path if option (b) over-spends the script-complexity budget.** If the backward-scan heuristic balloons past ~60 LOC during subtask-5 implementation (vs. the ~30-LOC sketch in Approach), the script falls back to **option (a) — hardcoded skip-list naming the three sites verbatim** (`default_style_tests.rs:2`, `timer_drivers.rs:450`, `render_harness.rs:545`). The fallback MUST add a CI assertion that the three skipped files have NOT gained NEW post-skip hits: a per-file baseline count check (e.g. `rg "$PATTERN_A|$PATTERN_B" <file> | wc -l` against an in-script baseline empirically re-derived for each of the three files at script-authoring time — exact baseline integers are produced by the implementer and checked into the script as named constants). Any increase fails the gate, preserving AC7's "new violations are caught" contract even under the fallback. Mitigation is graceful degradation: option (a) is documented up-front rather than discovered mid-implementation. Decision criterion for option-(a) fallback: documented in the script header along with the per-file baselines.
- **Pattern A regex has no `\btests/\b` token.** A doc-comment that says e.g. `tests/support/mod.rs` (as the round-1-discovered `render_harness.rs:312–313` near-miss does) is in violation of Scope §1's Family-A forbidden-paths list, but Pattern A's regex does not include a `\btests/\b` token and therefore does not fire. Mitigation: Scope §1 / §3 lists `tests/` as a Family-A forbidden path; review-gate prose (the `doc-convention.md` subsection + the four review-gate files) names "repo-internal paths" generally, so a human or LLM reviewer catches the leak. This iteration deliberately does NOT amend the spec's `## Audit patterns` to add the token (per round-1 review's "prefer NOT to amend" — the regex amendment is a spec change, and one identified near-miss is a thin justification). Documented here so a future PR that re-discovers the gap can revisit the trade-off with more data.
- **AC13 char-count creep.** Each instruction-file edit is bounded ≤ 250 chars by design, but a verbose rewrite during implementation could push one of the five files over 35k. Mitigation: subtask 3 and subtask 4 each end with a `wc -c` check on every touched instruction file as the last step before commit.

### Risk-and-verification consolidation (round-3 re-statement)

The three known `#[cfg(test)]`-enclosed false-positive sites at design time — `quartzite-style/src/default_style_tests.rs:2` (sibling-file `#[path]` shape), `quartzite-runtime/src/timer_drivers.rs:450` (inline `#[cfg(test)] mod`), and `quartzite-renderer/src/render_harness.rs:545` (inline `#[cfg(test)] mod`) — are explicitly handled by the regression script's **option-(b) backward-scan `#[cfg(test)]`-region filter** (preferred), with documented graceful degradation to **option (a) — hardcoded skip-list + per-file baseline new-hit assertion** if the heuristic over-spends the script-complexity budget. Both options preserve AC7's "new violations are caught" contract: option (b) by detecting any non-`#[cfg(test)]` violation regardless of file, option (a) by tripping the baseline-exceeded assertion when any of the three skipped files gains a NEW match beyond its design-time baseline. Subtask 5 + subtask 6 verification narratives (below) require the AC7 demo violation to be introduced in a NON-`#[cfg(test)]` file so the demo exercises the gate, not its skip path.

## Test Design

No new Rust tests. The gate surface is exclusively documentation + CI:

- **Subtask 1 verification:**
  - `cargo doc --no-deps --workspace --all-features` under `RUSTDOCFLAGS="-D warnings -D missing-docs"` → exit 0, no warnings (AC2).
  - `cargo test --workspace` → all green (AC10, sanity).
  - Re-run Pattern A + Pattern B against `quartzite-style-dispatch/src/`, `quartzite-core/src/`, `quartzite-widgets/src/`, `quartzite-renderer/src/`, `quartzite-style-types/src/`, `src/` excluding `tests/` and `benches/` and `quartzite-test-helpers/src/` and `#[cfg(test)]` regions → zero matches (AC1).
  - **Rewrite-location enumeration for AC1 (10 sites + 1 near-miss strip).** All ten lines below must be rewritten per the replacement-prose table; the eleventh entry is an in-table-paragraph parenthetical strip co-located with site (5) and not a separately-pattern-caught hit:
    1. `quartzite-style-dispatch/src/dispatch.rs:60–62` — Family A — drop "per `AGENTS.md` § *API Stability*" rationale sentence.
    2. `quartzite-core/src/snapshot.rs:27` — Family A — drop `[#39](github.com/.../issues/39)` and "tracked in" trailing sentence.
    3. `quartzite-core/src/snapshot.rs:138` — Family B — drop `\bthis implementation\b` token; rewrite to `[\`u32\`] version … equal to [\`CURRENT_SCHEMA_VERSION\`]`.
    4. `quartzite-core/src/snapshot/tree.rs:69` — Family B — drop `\bthis implementation\b` token; rewrite to "at or below [`CURRENT_SCHEMA_VERSION`]".
    5. `quartzite-renderer/src/render_harness.rs:300–306` — Family A — drop "(per spec AC1's 'or equivalent' escape hatch)" parenthetical + bold prefix.
    5b. *(Near-miss strip, same doc comment as site 5, lines 312–313.)* Family A — Scope §1 `tests/` path token — drop `(`tests/support/mod.rs`)` parenthetical. Not a Pattern-A regex hit (Pattern A has no `\btests/\b` token); caught by Scope §1's forbidden-paths list.
    6. `quartzite-style-types/src/dark_palette.rs:5` — Family A — drop "`design-system/README.md` § *Dark theme*" citation.
    7. `quartzite-style-types/src/dark_palette.rs:12` — Family A — drop "`design-system/README.md` § *Dark theme*" citation.
    8. `quartzite-widgets/src/widget_base.rs:100` — Family A — drop "(dispatch deferred to plan #47)" parenthetical.
    9. `quartzite-widgets/src/layout/mod.rs:16–17` — Family A — drop "(plan #47)" parenthetical.
    10. `src/lib.rs:203–204` — Family B — drop "Verify locally with `cargo build -p quartzite --no-default-features`" sentence.
  - **Adjacent attribute cleanup.** `quartzite-core/src/snapshot.rs:134` `#[error("... this implementation supports up to ...")]` — same `\bthis implementation\b` token, but in a `#[error(...)]` attribute that Pattern B's `^\s*(///\|//!)` anchor does not match. Rewrite anyway for AC4 consistency (the `Display` rendering reaches downstream readers). Counted as a documented adjacency, not as one of the ten regex-caught sites.
  - Expected post-rewrite Pattern A + Pattern B output against the enumerated globs: **zero matches** (AC1 assertion).
- **Subtask 2 verification:**
  - `cargo doc --no-deps --workspace --all-features` (same flags) → exit 0 (AC2).
  - `ls target/doc/ | grep -i test_helpers` → no output (AC6).
  - `cargo test --workspace` → all green (AC10).
- **Subtask 3 verification:** `wc -c ai-docs/doc-convention.md` → strictly less than 35000 (AC13). The new subsection's section-order placement: confirmed by `grep -n '^## ' ai-docs/doc-convention.md | head` showing the new `### Self-sufficiency: ...` h3 falls between `## Scope` (line 8) and `## References` (line 19 today).
- **Subtask 4 verification:** `wc -c` on each of the four review-gate files → all under 35000 (AC13). `rg -l "self-sufficiency-no-repo-internal-references" .claude/ ai-docs/doc-convention.md` → five files (AC12).
- **Subtask 5 verification:** `bash scripts/check-rustdoc-internal-refs.sh` against the post-subtask-1 tree → exit 0, no stdout, **after the `#[cfg(test)]`-region filter (option b) strips the three known false positives `default_style_tests.rs:2`, `timer_drivers.rs:450`, and `render_harness.rs:545`**. Confirm by adding a `-v` / `--debug` mode (or single-shot `--list-skipped` flag) during script authoring that prints the filtered-out hits — verify the printed list contains exactly those three sites and nothing more. If option (a) fallback is selected: verify the per-file baseline new-hit assertion fires when a deliberate new violation is added to one of the three skipped files (see AC7 demo below).
- **Subtask 6 verification:**
  - `actionlint .github/workflows/ci.yml` → exit 0 (AC8).
  - **AC7 demo: deliberate violation MUST go in a non-`#[cfg(test)]` file** so the demo exercises the gate's real pattern-matching path rather than its skip path. Pick a freshly-rewritten published-surface file (e.g. `quartzite-style-types/src/dark_palette.rs`, `src/lib.rs`, or any of the eight files touched by subtask 1) and add a transient `/// See AGENTS.md` line outside any `#[cfg(test)]` region (Pattern A); run the script, observe `exit 1` + the matched line; revert. Repeat with a Pattern B violation (e.g. add `/// Verify locally with cargo build` to the same non-`#[cfg(test)]` site); observe `exit 1`; revert. **Do NOT use a `#[cfg(test)]`-enclosed line for the demo** — that would exercise the skip path, not the gate, and pass silently regardless of whether the gate works. If option (a) fallback is in effect, additionally demonstrate the per-file baseline new-hit assertion: add one extra Pattern-B match to `timer_drivers.rs`'s `#[cfg(test)]` block (taking its retained-hit count from 1 to 2), observe `exit 1` + the baseline-exceeded message, revert.
  - Full workspace `cargo clippy --workspace --all-targets -- -D warnings` clean (AC9, sanity gate).

No fixtures, no helpers, no new `#[cfg(test)]` blocks. Self-review and review-findings will exercise the new review-gate bullets on this PR's own diff — the diff includes doc-comment edits, so subtask 4's checklist items run against subtask 1's output.

## Open questions

None. Spec resolved all design-affecting ambiguity in *Key decisions*. The two design-internal calls — (a) ship-in-this-PR vs fast-follow for the CI gate, (b) inline-copy vs anchor-link form for the audit-pattern citation in `doc-convention.md` — are both resolved above (ship in this PR; inline copy of both patterns in fenced code blocks for stand-alone local reproducibility).
