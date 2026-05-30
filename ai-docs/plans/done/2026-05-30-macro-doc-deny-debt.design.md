# Design: Resolve `#[object_impl]` doc-deny debt under `--all-features --all-targets`

**Issue:** #587
**Date:** 2026-05-30

## Approach

### Problem restated (verified live on master 2026-05-30)

Under `cargo clippy --workspace --all-targets --all-features -- -D warnings`
(equivalently `cargo build --workspace --all-targets --all-features`),
`quartzite-macros`'s doc-enforcement emits **51** hard `compile_error!`s of the
form `Annotated item \`<Type>::<member>\` is missing \`///\` documentation`.
They fire only when `--all-features` turns on both `undocumented-allow` and
`undocumented-deny` (deny wins — `util.rs::global_undocumented_level`, line 323)
**and** `--all-targets` compiles the test/example/bench fixture targets.

The full workspace build surfaces all 51 in one pass (cargo compiles crates in
parallel and reports every failure it reaches), so a single
`cargo build --workspace --all-features --all-targets --message-format=short`
gives the authoritative list. **Authoritative breakdown (live, supersedes the
spec's table where they differ):**

| Count | File | Target kind |
|---|---|---|
| 12 | `quartzite-macros/tests/object_impl.rs` | macro-internal fixture |
| 4 | `quartzite-macros/tests/object.rs` | macro-internal fixture |
| 4 | `quartzite-macros/tests/extend.rs` | macro-internal fixture |
| 3 | `quartzite-macros/tests/via_facade.rs` | macro-internal fixture |
| 8 | `examples/combined.rs` | example |
| 4 | `examples/hello_object.rs` | example |
| 2 | `examples/signals_slots.rs` | example |
| 1 | `examples/object_tree.rs` | example |
| 6 | `tests/signal_to_signal.rs` | macro-internal fixture |
| 3 | `tests/single_dep.rs` | macro-internal fixture |
| 3 | `benches/macro_object.rs` | macro-internal fixture |
| 1 | `quartzite-style/tests/third_party_paint.rs` | macro-internal fixture |

Totals: **23** `quartzite-macros` tests, **15** `examples/`, **36** fixtures.
The spec's "9 in `tests/signal_to_signal.rs`" is actually **6 in
`signal_to_signal.rs` + 3 in `single_dep.rs`** (both root-package integration
tests). The `examples/` 15 = combined 8 + hello_object 4 + signals_slots 2 +
object_tree 1. The macro-tests 23 split 12/4/4/3 across the four files. Net 51,
matching the spec's headline; the per-file map above is what implementation
follows.

### Opt-out mechanics (verified against parse/codegen)

The macro resolves a per-member level via
`resolve_undocumented_level(per_item_level, per_invocation_level)`
(`util.rs:337`) which falls back to the global feature level then `Warn`. The
escape hatches, **all parsed unconditionally during expansion** (the
`extract_*_invocation_level` / `extract_undocumented_per_item` calls are NOT
feature-gated — confirmed in `object/parse.rs:126`, `extend/parse.rs:307`,
`object_part/parse.rs:10`, `util.rs`), so they compile cleanly under the
**default** (non-`--all-features`) feature set as well:

- **Per-item:** `#[undocumented(allow)]` on the field/method — one attr per site.
- **Per-block, by macro:**
  - `#[object_impl(undocumented = "allow")]` on an impl block → all its
    `#[invokable]`/`#[slot]` methods.
  - `#[object(undocumented = "allow")]` sibling attr on a `#[derive(Object)]`
    struct → all its `#[prop]`/`#[signal]` fields.
  - `#[extend(undocumented = "allow")]` sibling attr on a `#[derive(Extend)]`
    struct → its `#[base]`/`#[mixin]`/`#[widget_children]` fields.
  - `#[object_part(undocumented = "allow")]` on an `#[object_part]` block.

**Critical fact for mixed structs:** a `#[derive(Extend, Object)]` struct splits
its members across two codegen paths — `#[base]` fields are diagnosed by *extend*
codegen (`extend/codegen.rs:17`), `#[prop]`/`#[signal]` by *object* codegen
(`object/codegen.rs:32`). Covering such a struct with per-block attrs therefore
needs **both** `#[extend(undocumented = "allow")]` **and**
`#[object(undocumented = "allow")]`.

**`"allow"` over `"warn"`:** at `Allow` the diagnostic emits an empty
TokenStream (`util.rs:363`) — strictly silent. At `Warn` it emits a
`#[deprecated]`-based diagnostic that is inert only while no `#![deny(deprecated)]`
is in scope; that is fragile and several fixtures carry `#![allow(deprecated)]`
comments that would become misleading. Use `undocumented = "allow"` for all
per-block opt-outs (matches the tri-state pass fixture
`per_invocation_allow_silences.rs`).

### Chosen strategy — split by target kind (per spec Key decisions)

**examples/ (15 sites): genuine `///` docs.** Examples are user-facing reference
material and must model the documented-by-default norm (AC6 forbids opt-out
attrs here). Add a concise one-line `///` to each struct field and impl method
flagged. Concurrently **remove the now-stale `// Example structs intentionally
lack \`///\` docs…` comment + `#![allow(deprecated)]` inner attribute** from
each example file once every member is documented — documented members emit
nothing, so the suppression is dead and the comment becomes false. (Verify each
example still compiles cleanly after the `#![allow(deprecated)]` removal under
both default and `--all-features`; if any non-doc deprecated warning surfaces,
keep the attribute but fix the comment.)

**Macro-internal fixtures (36 sites): per-block `undocumented = "allow"`
opt-out.** Doc prose on internal fixtures is noise. Per-block is consistently
**fewest-or-equal edits vs per-item** on every fixture file (computed live):

| File | per-item edits | per-block edits | chosen |
|---|---|---|---|
| `object_impl.rs` | 12 | 10 (3 structs ×{extend+object} + 4 impl blocks) | per-block |
| `object.rs` | 4 | 2 (extend+object on `Counter`) | per-block |
| `extend.rs` | 4 | 3 (extend on Widget, Button, Panel) | per-block |
| `via_facade.rs` | 3 | 2 (extend+object on `Sensor`) | per-block |
| `signal_to_signal.rs` | 6 | 4 (extend+object on Emitter, Relay) | per-block |
| `single_dep.rs` | 3 | 2 (extend+object on `Counter`) | per-block |
| `macro_object.rs` (bench) | 3 | 2 (extend+object on `BenchObject`) | per-block |
| `third_party_paint.rs` | 1 | 1 (extend on `ThirdPartyWidget`) | tie → per-item `#[undocumented(allow)]` (1 attr, no sibling-attr noise) |

For the lone `third_party_paint.rs` site, per-item `#[undocumented(allow)]` on
the `#[base] widget_base` field is the minimal, most-local edit and is preferred
over a struct-level sibling attr for a single field.

For fixtures that already carry a file-level `#![allow(deprecated)]` +
"intentionally lack docs" comment (`signal_to_signal.rs`, `single_dep.rs`,
`macro_object.rs` — verify each), the per-block `undocumented = "allow"` attrs
make the `#![allow(deprecated)]` redundant for the doc-diagnostic purpose. Leave
the `#![allow(deprecated)]` in place only if it suppresses other deprecated
usage; update the stale comment to reference the new opt-out mechanism. Do not
churn it otherwise.

**CI gate (1 edit): add `--all-targets` to the Feature-matrix Clippy step**
(`.github/workflows/ci.yml:523`):
`cargo clippy ${{ matrix.features }} --workspace --all-targets -- -D warnings`
(AC3). `libfontconfig1-dev` is already installed in that job (line 514, PR #588).
`actionlint .github/workflows/ci.yml` MUST pass before `git add` (AGENTS.md
AXIOM).

### Rejected alternatives

- **Uniform per-item `#[undocumented(allow)]` on all 36 fixture sites** —
  rejected: more edits on every multi-member fixture (51→36 attrs vs ~25), and
  the spec asks for fewest edits per target.
- **Global `undocumented-allow` feature only** — rejected: `--all-features`
  also enables `undocumented-deny`, which wins (`util.rs:323`); the feature
  alone cannot suppress under the target gate (spec Key decisions row).
- **Relaxing the macro's deny precedence** — explicitly out of scope (spec).
- **Adding `--all-targets` to the features-job Build/Test steps too** — out of
  scope (spec AC3 names only the Clippy step). See Open questions.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Genuine `///` docs on all 15 `examples/` sites; remove/repair stale `#![allow(deprecated)]` + "intentionally lack docs" comments. **Caveat (part-accumulated slots):** in `examples/combined.rs`, the three flagged members `Counter::increment`/`decrement`/`reset` are *reported* at the terminal `#[object_impl]` span (71:1, the block that drains the parts) but are actually *defined* in the `#[object_part]` blocks (lines 36/42/61) — put the `///` doc on the real `#[object_part]` method definition, NOT on the reported terminal `#[object_impl]` span. | `examples/combined.rs`, `examples/hello_object.rs`, `examples/signals_slots.rs`, `examples/object_tree.rs` | — |
| 2 | Per-block `undocumented = "allow"` opt-out on the 23 `quartzite-macros` test sites | `quartzite-macros/tests/object_impl.rs`, `quartzite-macros/tests/object.rs`, `quartzite-macros/tests/extend.rs`, `quartzite-macros/tests/via_facade.rs` | — |
| 3 | Per-block `undocumented = "allow"` opt-out on `signal_to_signal.rs` (6) + `single_dep.rs` (3); per-block on `benches/macro_object.rs` (3); per-item `#[undocumented(allow)]` on `third_party_paint.rs` (1) — 13 fixture sites | `tests/signal_to_signal.rs`, `tests/single_dep.rs`, `benches/macro_object.rs`, `quartzite-style/tests/third_party_paint.rs` | — |
| 4 | Add `--all-targets` to Feature-matrix Clippy step; run `actionlint` before staging | `.github/workflows/ci.yml` | 1, 2, 3 |
| 5 | Full-gate verification: AC1/AC2 full-gate green, AC5 pre-existing gates green (default-feature clippy, `cargo test --workspace`, `--all-features` doc gate) | — | 1, 2, 3, 4 |

(5 subtasks — within the ≤7 limit; no issue split needed.)

## Handoff plan

This design has **M = 5** subtasks. Per `.claude/skills/task/SKILL.md` Step 8 +
`.claude/agents/design.md` § Rules, the `## Handoff plan` is **mandatory for
every M ≥ 1**; non-terminal groups are **exactly 3** consecutive subtasks; the
terminal group is **1..=3**; the handoff destination is `/context-reset` per
`.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry), and
is named in prose at **every** group entry including the first.

- **Entry into Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry)
  before starting Group A.
- **Group A:** subtasks 1–3 — initial implementation chunk (the three
  doc/opt-out edit clusters; 3 subtasks, equals the non-terminal cap).
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — terminal group (CI gate edit + actionlint, then
  full-gate verification; 2 subtasks, within the 1..=3 range). No handoff after
  the terminal group; it completes Step 8 in its own `/context-reset` subagent.

## Risks

- **Mixed-derive struct under-coverage:** forgetting that `#[base]` (extend) and
  `#[prop]`/`#[signal]` (object) need *separate* per-block attrs leaves residual
  errors. Mitigation: design's per-file edit table enumerates both sibling
  attrs per struct; the AC1 full-gate build is the catch-all.
- **`"warn"` vs `"allow"` regression via `#![deny(deprecated)]`:** using `"warn"`
  would re-fire as a deprecated error in any file that later gains
  `#![deny(deprecated)]`. Mitigation: standardise on `undocumented = "allow"`
  (empty TokenStream — strictly silent).
- **Removing `#![allow(deprecated)]` from examples surfaces an unrelated
  deprecated warning** (clippy `-D warnings` would then fail). Mitigation:
  subtask 1 verifies each example compiles under default + `--all-features`
  after removal; restore the attribute (comment repaired, not removed) if any
  non-doc deprecated lint appears.
- **Default-feature regression:** opt-out attrs must parse when
  `undocumented-deny` is inactive. Mitigation: verified the
  `extract_*_invocation_level` parse paths are not feature-gated; AC5's
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace` confirm no default-feature regression.
- **`actionlint` skipped before `git add` of the workflow:** AGENTS.md AXIOM —
  repeat offence. Mitigation: subtask 4 runs `actionlint .github/workflows/ci.yml`
  as a gating step before staging (AC4).
- **Doc gate interaction:** `examples/` are not part of the `cargo doc` surface
  (binaries/examples aren't doc-checked the same way), so genuine `///` docs do
  not risk new `missing_docs`/intra-doc-link failures; AC5 doc gate re-run
  confirms.

## Test Design

No new product logic is introduced — this is fixture/doc/CI hygiene. "Tests" are
the acceptance-gate command runs (subtask 5), not new `#[cfg(test)]` modules.

- **Location:** workspace-level gate commands (no source test module added; the
  edited files are themselves test/example/bench fixtures).
- **Entry points / scenarios:**
  - **AC1 (happy path):** `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0 — zero `missing '///' documentation` errors.
  - **AC2:** `cargo build --workspace --all-targets --all-features` exits 0.
  - **AC5 default-feature gate:** `cargo clippy --workspace --all-targets -- -D warnings` stays green (opt-out attrs parse with deny inactive).
  - **AC5 test gate:** `cargo test --workspace` stays green (fixtures still compile + run; examples unaffected at runtime).
  - **AC5 doc gate:** `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` stays green (new example docs introduce no broken intra-doc links).
  - **AC4:** `actionlint .github/workflows/ci.yml` passes (gates subtask 4 before `git add`).
  - **AC3 (structural):** grep-confirm the Feature-matrix Clippy line reads `cargo clippy ${{ matrix.features }} --workspace --all-targets -- -D warnings`.
- **Edge case to re-check during impl:** after each fixture file edit, a focused
  `cargo build -p <pkg> --all-features --all-targets --message-format=short 2>&1 | grep "missing \`///\`"` for that package should return empty before moving on (incremental verification, since the per-crate abort masks downstream).
- **Fixtures / helpers needed:** none new.

## Open questions

- **Features-job Build/Test `--all-targets`:** AC2 requires
  `cargo build --workspace --all-targets --all-features` green *locally*, but
  the CI features-job **Build** (`ci.yml:517`) and **Test** (`:520`) steps run
  `cargo build/test ${{ matrix.features }}` **without** `--all-targets`. The
  spec scopes the CI change to the **Clippy** step only (AC3 + Out-of-scope row)
  — so this design leaves Build/Test as-is. Confirm that is intended: once the
  Clippy `--all-targets` step is green, the equivalent Build would also pass, so
  leaving Build/Test target-narrow only means CI doesn't *separately* exercise
  the `--all-targets` build there. No action unless the product owner wants the
  features-job Build/Test widened too (would be a spec amendment).
