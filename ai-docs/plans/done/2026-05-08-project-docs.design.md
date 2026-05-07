# Design: Project docs — README description, facade `lib.rs` doc, badges, CONTRIBUTING, ROADMAP

**Issue:** #60
**Date:** 2026-05-08
**Spec:** [`2026-05-08-project-docs.spec.md`](2026-05-08-project-docs.spec.md)

## Approach

Five user-facing project docs items land in one PR, organised as one logically-complete subtask per AC group so each lands as its own commit on the feature branch (per AGENTS.md `## Workflow`). Implementation order follows the dependency direction: generator before its CI gate; facade `lib.rs` doc before README badges that link to its rendered output; README description block before CONTRIBUTING (which cross-links to it).

**Why this approach:**
- **One PR, sequential subtask commits.** Five user-facing artefacts that share a "first round of project polish" theme — one PR keeps the change set discoverable; sequential commits keep `git log` legible and bisect-friendly. The spec already pre-decided "single PR", so this is the implementation shape.
- **Generator first, badge/gate second.** `ROADMAP.md` cannot be reviewed before the generator that produces it exists, and the CI sync-gate cannot pass before the committed `ROADMAP.md` matches generator output. So the natural order is: write `gen-roadmap.sh`, run it, commit `ROADMAP.md`, then add the CI step in a separate commit. This keeps each commit self-consistent (each commit's tree passes its own gates).
- **`lib.rs` doc rewrite is one commit.** It is one logical unit (overview + quickstart + 5 sections + ecosystem map + design notes). Splitting per-section produces commits that intermediate-state-fail the doctest (`AC9`), which violates the "every commit builds clean" workflow contract.
- **Trivial commits batched.** README description block + the two new badges are small text edits in the same file — one commit. CONTRIBUTING is its own commit because it's a new file at standard depth (~80–120 lines).

**Rejected alternatives:**
- **Separate PRs per item.** Five PRs with mostly-doc churn each. Reviewer cost dominates; the items are coupled (CONTRIBUTING points to AGENTS.md and to the facade `lib.rs` docs; ROADMAP references plan files; badges link to the rendered crate doc). Spec already pre-decided "single PR".
- **Squash-merge into one commit.** AGENTS.md `## Workflow` rule is `gh pr merge --merge`; no squash. Subtask-per-commit is the load-bearing structure for `git bisect` and reviewer mental model.
- **Generator written in Rust as a workspace member.** A new crate to materialise a 100-line markdown file is YAGNI; POSIX bash + awk/sed runs everywhere CI runs, ships zero compile time, and the spec already pre-decided this.
- **Run the CI sync-gate on all three OSes (`ubuntu-latest`, `macos-latest`, `windows-latest`).** Windows bash-via-`shell: bash` invokes Git-Bash MSYS2's `awk`/`sed`, which differ subtly from GNU on macOS coreutils and on POSIX BSD awk on macOS. Re-running an idempotent generator on three OSes adds two cache-cold runs, ~3–5 min CI time each, for zero value: the test is "does the committed file match the generator output" — a content-equality check that is platform-independent in spirit and only at risk of false-negatives on Windows due to CRLF / awk-flavour drift. **Decision:** sync-gate runs on `ubuntu-latest` only (rationale below in *Open questions resolved*). The generator is still POSIX bash + portable awk/sed so a developer on macOS can run it locally.
- **`lib.rs` quickstart shape: config-style with one property.** The conceptual hook of the project is signals/slots (the `tokio` analogue is `async`/`await`). A counter with a `count_changed` signal + a slot exercises the *signature* feature; a property-only example shows the reflection layer but underplays the dispatch model. **Decision:** counter-style with one signal, one slot, one property (count + count_changed + reset slot). This matches `examples/hello_object.rs` conceptually but the snippet is freshly authored as a tighter standalone unit (rationale below).
- **Mirror `INDEX.md` byte-for-byte in `ROADMAP.md`.** The two files have different audiences: `INDEX.md` lives under `ai-docs/` for agents and the maintainer; `ROADMAP.md` lives at repo root for external visitors. ROADMAP needs intro prose ("This file is auto-generated…"), the dependency tree, and the three plan tables (active / completed / deferred). It does **not** need the "Suggested next steps" prose section (internal sequencing advice for the maintainer), the trailing maintenance-plans paragraph (a footnote about cross-cutting plans that don't fit the dependency tree), or the tracking-issue footnote (issue-tracker state, not roadmap content). **Decision:** generator emits header + dependency tree + three plan tables; "Suggested next steps", maintenance-plans paragraph, and the tracking-issue footnote are all dropped (out-of-context for an external roadmap audience). Rationale below.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add project-description block at top of README (above existing badges) + add CI-status and license badges immediately below; existing codecov + docs badges retained, ordered: CI, docs, codecov, license | `README.md` | — |
| 2 | Rewrite `src/lib.rs` crate-level rustdoc: keep existing inner attributes + `document_features!()` invocation in their current relative positions; replace the `//!` body with overview + `no_run` quickstart + 5 per-concept sections (signals, properties, object tree, event loop, timers) + ecosystem map + design notes; new content lands **before** `# Feature flags` heading | `src/lib.rs` | — |
| 3 | Create `CONTRIBUTING.md` at repo root with high-leverage AGENTS.md excerpts + dual-license-contribution clause + pointer to `AGENTS.md` for canonical reference | `CONTRIBUTING.md` | — |
| 4 | Create `scripts/gen-roadmap.sh` (POSIX bash + awk/sed; executable bit set; deterministic output) — parses `ai-docs/plans/INDEX.md` Active/Completed/Deferred tables + the `## Dependency order` fenced code block; emits `ROADMAP.md` with header + dependency tree (renamed `## Dependency tree` in output) + three plan tables. Stops emission before `## Suggested next steps` — that section is internal sequencing prose for the maintainer audience and is dropped, alongside the maintenance-plans paragraph and the tracking-issue footnote | `scripts/gen-roadmap.sh` | — |
| 5 | Run the generator and commit the produced `ROADMAP.md` at repo root | `ROADMAP.md` | 4 |
| 6 | Add CI sync-gate: a new `roadmap-sync` job in `.github/workflows/ci.yml` (Linux-only) — checks out, runs `scripts/gen-roadmap.sh`, runs `git diff --exit-code ROADMAP.md`; new `roadmap-sync-pass` aggregator alongside the existing `*-pass` aggregators | `.github/workflows/ci.yml` | 5 |

Six tasks, ordered. Each task = one commit on the feature branch. No task depends on more than one prior task; tasks 1, 2, 3, 4 are independent and could in principle be reordered, but the listed order matches the natural narrative (README first establishes the project description that lib.rs and CONTRIBUTING reference; generator before its CI gate).

### Open questions resolved (concrete decisions for the implementer)

These resolve the spec's "Open questions" section so the implementer lands them verbatim.

#### 1. Project-description block shape — **tight tagline + bullets** (1 paragraph + 4-bullet *Current scope* + 4-bullet *Forward scope* + 3-bullet *Non-goals*)

Reasoning: the README already has a `## Status` table with crate-level checkmarks, so a 3–5-paragraph essay would duplicate the table. A tagline + bullets form is denser, scans in <10 seconds (typical README first-read budget), and survives translation to a future crates.io short-description. The existing 3-line description ("A GUI and object framework… no foreign ABI…") is currently the *only* description — we replace it with a richer block of the same shape, not paragraphs.

Concrete shape (the implementer authors prose at this depth):

```
# quartzite

[badges row: CI, docs, codecov, license]

A GUI and object framework for Rust drawing on Qt's signals/slots and
property/reflection model — implemented in idiomatic Rust with no native
dependencies, no foreign ABI, and no codegen outside proc-macros.

## Current scope

- Object model: `ObjectBase`, parent/child trees, named lookup, reflection metadata.
- Signals/slots: typed `Signal<Args>`, dynamic dispatch via `Object::invoke_method`,
  cross-thread queued connections.
- Event loop: `Application` singleton, per-thread `EventLoop`, queued dispatcher.
- Timers: `Timer` object with `AppDriver` / `PoolDriver` / `ThreadDriver` execution
  contexts.

## Forward scope

- Painting API (`quartzite-paint-api`) — Painter trait + thin abstraction layer.
- Renderer (`quartzite-renderer`) — vello + wgpu + winit integration.
- Widgets (`quartzite-widgets`) — Widget trait built on the painting API.
- Style system (`quartzite-style`) — declarative styling on top of widgets.

## Non-goals

- Not a Qt port or a Qt binding — Qt is design lineage, not API surface.
- No FFI / native dependencies — pure-Rust toolchain.
- No GPU rendering yet — arrives with the renderer (#73).
```

The existing `## Status` table, `## Usage`, and downstream sections stay as-is.

#### 2. `lib.rs` quickstart concrete content — **counter-style with signal + slot** (one property + one signal + one slot)

Snippet for the implementer to land verbatim inside the `//!` block under a `# Quickstart` heading. `no_run` per spec; default features (`std + derive`). The example does **not** call `Application::run` (which would block) — it just constructs the object, connects, and emits, demonstrating the dispatch model end-to-end:

````markdown
//! # Quickstart
//!
//! ```no_run
//! use quartzite::prelude::*;
//!
//! #[derive(Extend, DeriveObject)]
//! #[root]
//! struct Counter {
//!     #[base]
//!     object_base: ObjectBase,
//!     #[prop(notify = count_changed)]
//!     pub count: i32,
//!     #[signal]
//!     pub count_changed: Signal<(i32,)>,
//! }
//!
//! #[object_impl]
//! impl Counter {
//!     #[slot]
//!     fn reset(&mut self) {
//!         self.count = 0;
//!     }
//! }
//!
//! let mut c = Counter {
//!     object_base: ObjectBase::new(),
//!     count: 0,
//!     count_changed: Signal::new(),
//! };
//!
//! // Connect a slot to the count_changed signal.
//! c.count_changed.connect(|args| println!("count is now {}", args.0));
//!
//! // Writing the property emits count_changed automatically.
//! c.write_property("count", Value::Int(42));
//!
//! // Invoke the slot dynamically through the reflection layer.
//! c.invoke_method("reset", &[]);
//! ```
````

Reasoning: this snippet exercises every conceptual feature the per-concept sections then expand on (object derive, property + notify, signal, slot, dynamic dispatch). It is fresh-authored — not copied from `examples/hello_object.rs` — to avoid a maintenance fork; doctest `cargo test --doc -p quartzite` covers compilation. Length is short enough (~25 lines of code) to stay on the rustdoc landing page without scrolling past the header.

The `Application` event-loop API is **not** demonstrated in the quickstart (mentioned in the *Event loop* section instead with a deep-link to `examples/timer.rs`) because constructing an `Application` would force the example to either block (`exec()`) or set up a teardown path that distracts from the dispatch model. `tokio`'s landing page makes the same trade-off — its quickstart shows `tokio::main` + a single `await`, not the runtime builder.

#### 3. CI sync-gate OS coverage — **`ubuntu-latest` only**

Reasoning:
- The check is content equality (`git diff --exit-code ROADMAP.md`); platform variance buys nothing on the success path.
- On the failure path, Windows-bash variance (Git-Bash MSYS2 `awk`/`sed` flavour, line-ending handling) would produce false negatives that block PRs without indicating a real bug. Local development and the generator work everywhere; the CI gate's job is to enforce sync, not to validate cross-platform shell portability.
- The existing CI matrix already covers `windows-latest` for `cargo build` / `cargo test` / `cargo clippy` per #133 / PR #141. The roadmap-sync gate is a different concern.
- Adding a third OS triples runner cost and adds 2 cache-cold setup runs (no Cargo cache reuse) for a check that runs 1–2 second of awk on a 100-line file.

Workflow change shape:

```yaml
roadmap-sync:
  name: ROADMAP sync
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v6
    - name: Regenerate ROADMAP.md
      run: ./scripts/gen-roadmap.sh
    - name: Verify no drift
      run: git diff --exit-code ROADMAP.md

roadmap-sync-pass:
  name: ROADMAP sync
  needs: roadmap-sync
  runs-on: ubuntu-latest
  if: always()
  steps:
    - run: |
        if [[ "${{ needs.roadmap-sync.result }}" != "success" ]]; then exit 1; fi
```

Action versions to use (per AGENTS.md `## Dependency Versions` — *Query the registry before pinning*): `actions/checkout` — implementer queries `gh api /repos/actions/checkout/releases --jq '.[0].tag_name'` at implementation time and uses the observed major. The other CI jobs in `ci.yml` currently use `@v6`; if that has not changed by implementation time, match it. (Do NOT pin from this design doc — that defeats the registry-query rule.)

#### 4. AGENTS.md bullets to excerpt into `CONTRIBUTING.md` — concrete list

Standard depth ≈ 80–120 lines. The excerpt is **prose-style summary**, not verbatim AGENTS.md text — external contributors aren't agents and don't need the same imperative voice. Each section ends with "see `AGENTS.md` § <heading>" for the canonical text.

Sections (in order):

1. **One-paragraph welcome** — what `quartzite` is + invitation + canonical reference pointer.
2. **Workflow** — branch-before-edit (never commit on local `master` for PR work); branch naming (`feat/`, `fix/`, `chore/`, `docs/` prefixes per existing branch history); merge with `gh pr merge --merge` (no squash); refresh `Cargo.lock` via `cargo build` before commit.
3. **Staging discipline** — name files explicitly; never `git add -A` / `git add .` (one sentence each).
4. **Before commit (the test loop)** — bullet list of the four required gates (`cargo build`, `cargo test`, `cargo clippy --workspace -- -D warnings`, `cargo fmt -- --check`) + `cargo build -p quartzite --no-default-features` for the no-default-features check + `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` for the doc gate.
5. **Workflow files** — if your PR touches `.github/workflows/*.yml`, run `actionlint` on each modified file before staging.
6. **Code style** — pointer to `ai-docs/code-style.md` as the canonical reference; one-sentence summary of the most-likely-relevant rules: max 100 cols (rustfmt default), `#![deny(missing_docs)]` workspace-wide, `thiserror` for error types, no blanket `#[allow]` without justification.
7. **Documentation conventions** — pointer to `ai-docs/doc-convention.md`; one-sentence summary: every public item gets a `///` summary in third-person present indicative + `# Examples`, with `# Parameters` / `# Errors` / `# Panics` / `# Safety` per the convention's section-order rule.
8. **Tests** — pointer to AGENTS.md `## Rust Test Conventions`; one-sentence summary: unit tests under `#[cfg(test)] mod tests`, integration tests in `tests/`, `rstest` for parameterised tests, `mockall` for trait mocks, `pretty_assertions` for diffs.
9. **License & contribution clause** — verbatim copy of the README's existing dual-license `### Contribution` paragraph (the spec calls for this).
10. **For agent-driven development** — one-paragraph note: this repo uses Claude Code workflows (`.claude/skills/`); agents follow `AGENTS.md`; humans are encouraged to read it but not strictly required to follow agent-only rules (e.g., the `## Corrections Log` is for agents).

What is **not** included — these are agent-internal concerns, not contributor concerns:
- The `/task` workflow steps (1–8) — internal to the Claude Code skill; external contributors don't need them.
- The Propagation Rule for editing instruction files.
- The Corrections Log format.
- The Permissions section (machine-enforced via `.claude/settings.json` + branch protection — external contributors don't write to those).
- The PR review comment resolution mechanics — those are agent-level workflow.
- The Communication section ("wtf?", "submit to PR" semantics) — agent↔user protocol.

Reasoning for inclusion vs. exclusion: a contributor's first PR needs to know how to (a) make a change locally (workflow + staging), (b) verify it (gates), (c) match house style (code-style + doc-convention pointers), (d) write tests (test conventions). Everything else is `AGENTS.md`'s job, and `CONTRIBUTING.md` says so plainly.

#### 5. Generator format choices for `ROADMAP.md` — distill, don't mirror

Output skeleton (the generator emits this verbatim text + parsed `INDEX.md` content slotted in):

```markdown
# quartzite — Roadmap

> **Auto-generated** from [`ai-docs/plans/INDEX.md`](ai-docs/plans/INDEX.md) by
> [`scripts/gen-roadmap.sh`](scripts/gen-roadmap.sh). Do not edit by hand —
> changes here will be reverted by the CI sync-gate. Edit `INDEX.md` instead
> and re-run the generator.

Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked

## Dependency tree

[verbatim copy of the ```-fenced block under INDEX.md's `## Dependency order` heading]

## Active plans

[the Active-plans table from INDEX.md, plan links rewritten so relative paths
resolve from the repo root: `(done/2026-...)` → `(ai-docs/plans/done/2026-...)`]

## Completed plans

[the Completed-plans table from INDEX.md, links rewritten as above]

## Deferred plans

[the Deferred-plans table from INDEX.md, links rewritten as above]
```

**Heading triggers — input vs. output (load-bearing for the awk state machine).**

- **INDEX.md input headings** the awk state machine matches (verbatim spelling, exactly as they appear in INDEX.md — verified via `grep -n '^## ' ai-docs/plans/INDEX.md` at design time):
  - `## Active plans` — opens the active-plans table extraction state.
  - `## Completed plans` — opens the completed-plans table extraction state.
  - `## Deferred plans` — opens the deferred-plans table extraction state.
  - `## Dependency order` — opens the fenced-code-block extraction state for the dependency tree.
  - `## Suggested next steps` — **terminator**: the awk state machine stops emitting from this heading onward. Everything below it in INDEX.md (the suggested-next-steps numbered list, the maintenance-plans paragraph, and the tracking-issue footnote blockquote) is dropped from `ROADMAP.md`.

- **ROADMAP.md output headings** the `printf` literals emit:
  - `## Active plans`, `## Completed plans`, `## Deferred plans` — verbatim from INDEX.md.
  - `## Dependency tree` — **renamed** in output (input is `## Dependency order`; the rename targets external readers for whom "dependency tree" reads as the more familiar visualisation term).
  - No `## Suggested next steps` heading is emitted — the section is dropped entirely.

**What the generator drops (and why).**

- `## Suggested next steps` and its numbered list — internal sequencing advice for the maintainer ("Start graphics-stack…", "Expand prelude as new crates land…"); same audience-mismatch as the maintenance-plans paragraph below. Including it would also force the link-rewrite pass to handle parent-up references (`](../code-style.md…)`, `](../doc-convention.md)` inside that section's prose) that copy verbatim from INDEX.md and 404 from the repo root — dropping the section eliminates that rewrite class outright.
- The trailing "Maintenance plans (cross-cutting, all ✅): …" paragraph — maintenance-history footnote, out-of-scope for an external roadmap audience.
- The "Tracking issues for further deferred items not represented as plans here" blockquote — issue-tracker state, not roadmap content; better answered by GitHub's issue tracker.
- The "Serialization-layer track (#107)" one-line blockquote footnote — needs context unavailable to an external reader.

Why drop them collectively: ROADMAP's audience is "what's the project doing and where is it going?" The dropped sections answer "what cross-cutting work has happened?", "what's the issue-tracker state?", and "what should the maintainer queue next?" — different questions answered better by `INDEX.md`, the GitHub issue tracker, and the maintainer's own planning notes respectively. Dropping `## Suggested next steps` also keeps the link-rewrite pass narrow (only table-row plan links), which is the simpler implementation per the reviewer's recommendation.

**Generator implementation shape (POSIX bash + awk):**
- Script reads `ai-docs/plans/INDEX.md` once, writes `ROADMAP.md`.
- Uses `awk` with state machine matching the input headings listed above (`/^## Active plans$/{ in_active=1 }` etc.) to extract the four target blocks (active table, completed table, deferred table, dependency-order code-block). The state machine **terminates output** at `/^## Suggested next steps$/` — anything below that line in INDEX.md is never emitted.
- Uses a small `sed` pass on the extracted table rows only to rewrite relative plan links: `](done/` → `](ai-docs/plans/done/`, `](deferred/` → `](ai-docs/plans/deferred/`, `](2026-` → `](ai-docs/plans/2026-` (the active-plans table has bare `2026-...` links for in-progress plans). No parent-up rewrites needed: the dropped `## Suggested next steps` section was the only place INDEX.md held `](../code-style.md…)` / `](../doc-convention.md)` references; the three plan tables and the dependency-order block contain only the three rewrite-classes listed.
- The header text and section dividers (including the `## Dependency tree` rename) are emitted as `printf` literals interleaved with the awk-extracted blocks.
- Determinism: no timestamps in the output; same input bytes produce same output bytes; `LC_ALL=C` set at script top to fix sort/awk locale.
- Hashbang: `#!/usr/bin/env bash`. POSIX-bash (no `[[ ]]`-only Bash-isms inside awk programs); `awk` invoked as `awk` (BSD-compatible) without GNU-only `gensub` / `--re-interval`. `sed` invoked with portable basic regex (no `-E` / `-r`); use `sed 's|](done/|](ai-docs/plans/done/|g'`-style alternation pipes via separate `-e` clauses.
- Idempotency check at end: optionally `git diff --exit-code ROADMAP.md` returns 0 — but the script itself does NOT call git (the CI gate does that); the script only writes to `ROADMAP.md`.

## Risks

- **`document_features` invocation accidentally moved.** The macro must remain inline within the `//!` block, immediately after the `# Feature flags` heading (per `ai-docs/doc-convention.md` `## Feature flags rendering` and AC12). The implementer rewrites the surrounding `//!` body but must NOT touch the `# Feature flags` heading or the `#![doc = document_features::document_features!()]` line. Mitigation: AC12 is explicit; reviewer checklist verifies; new content lands strictly *before* the `# Feature flags` heading.
- **Quickstart doctest fails on `--no-default-features`.** The snippet uses `derive` macros and `Application` is std-only. The doctest is gated under default features (`std + derive`); the existing AC10 already requires `cargo build -p quartzite --no-default-features` to pass — `cargo test --doc` is not run on no-default-features (and the workspace doc-test infrastructure assumes default features). Mitigation: keep the snippet inside the default-features `//!` block (no `cfg_attr`-gated doc); rely on AC9 to confirm `cargo test --doc -p quartzite` passes.
- **`document_features` heading-level mismatch.** The current source has `# Feature flags` (H1 in `//!`-rendered rustdoc). New per-concept sections must use the same heading level (H1 — `# Quickstart`, `# Object model`, `# Signals`, etc.) so the rendered TOC stays consistent. Mitigation: spec already implies "5 per-concept sections" + ecosystem map + design notes; implementer uses H1 for all top-level sections. (The `##` form is also acceptable per the doc-convention's "H1 or H2 — match sibling sections" guidance, but H1 matches what `# Feature flags` already uses.)
- **Generator parsing fragility.** The generator's awk state machine depends on the exact section headings in `INDEX.md` — four extraction triggers (`## Active plans`, `## Completed plans`, `## Deferred plans`, `## Dependency order`) plus one terminator (`## Suggested next steps`, which stops output). A future edit renaming any of the five would silently break the gate (a missing extraction trigger = empty section in output; a missing terminator = the dropped sections leak into ROADMAP.md). Mitigation: AC7 catches drift on every PR; the generator script is short enough (~50 lines) that a renaming PR will obviously update both files together. Add a leading comment in `gen-roadmap.sh` listing the five load-bearing `INDEX.md` headings (four extraction + one terminator) explicitly.
- **Generator non-portability.** If the implementer accidentally uses GNU-only `sed -i` or `awk` `gensub`, the script breaks on macOS local development and possibly on Windows Git-Bash. The CI gate runs on `ubuntu-latest` (GNU coreutils) only — it will not catch portability bugs that surface only on BSD-awk / macOS. Mitigation: (a) the in-source comment banner mandated by AC6 test-design lists every banned construct so the next editor sees the warning before reaching for the wrong tool; (b) AC6 implementer responsibility requires a one-time macOS / BSD-awk-container portability cross-check with byte-identical output before commit, documented in the progress file. Reviewer checklist verifies both.
- **Idempotency fragility.** If the generator emits a timestamp, the CI sync-gate fails on every push. Mitigation: AC6 explicitly forbids non-determinism; design names "no timestamps in output" as the specific rule.
- **CI sync-gate aggregator naming collision.** Adding `roadmap-sync-pass` as a new aggregator job parallels the existing `*-pass` jobs (`build-pass`, `test-pass`, `clippy-pass`, `features-pass`). The branch protection rule (per AGENTS.md `## Permissions` — "machine-enforced rules live in `.claude/settings.json` and on `origin`") will require the new job to be added to the required-checks list before it can block PRs. Mitigation: this is a post-merge user action — see *Post-merge owner actions* below.
- **README badge ordering.** The spec preserves codecov + docs badges and adds CI + license. Order matters for consistent visual scan. **Decision:** CI, docs, codecov, license — left-to-right matches the conceptual flow (the build runs, produces docs and coverage, all under a license). License is last because it's the least-changing badge.
- **`CONTRIBUTING.md` drift from AGENTS.md.** A future change to AGENTS.md may invalidate a CONTRIBUTING.md excerpt. Mitigation: every CONTRIBUTING section ends with "see `AGENTS.md` § <heading>"; if AGENTS.md changes, the pointer remains valid even if the summary becomes stale. The next `/improve` cycle catches stale summaries.

## Test Design

This task is documentation + a shell script + a CI workflow. There are no Rust `#[cfg(test)]` modules to add. Verification is performed via existing CI gates and per-AC manual / automated checks:

### AC1, AC2 (README)

- Verification: visual review of `README.md` against the AC text.
- No automated test; no doctest changes.

### AC3 (lib.rs comprehensive doc)

- Verification: `cargo doc --no-deps --workspace` clean (AC8) — ensures intra-doc links resolve.
- Visual review confirms presence of: overview prose, `# Quickstart` section with `no_run` doctest, 5 per-concept sections (`# Signals`, `# Properties`, `# Object tree`, `# Event loop`, `# Timers` — names finalised at implementation time), ecosystem map, design notes.
- Versioning-policy section absent (negative AC; reviewer checklist).

### AC4 (CONTRIBUTING.md)

- Verification: visual review against the bullets-list in *Open questions resolved → 4*.
- No automated test.

### AC5, AC6 (ROADMAP.md + generator)

- Verification: run `./scripts/gen-roadmap.sh && git diff --exit-code ROADMAP.md` — must exit 0 (idempotent).
- Output review: `## Dependency tree` heading present (renamed from INDEX.md's `## Dependency order`) + three plan tables present (`## Active plans`, `## Completed plans`, `## Deferred plans` verbatim) + dropped sections absent (no `## Suggested next steps`, no maintenance-plans paragraph, no tracking-issue footnote).
- Determinism: run twice; output bytes identical (`sha256sum ROADMAP.md` matches across runs).
- **Implementer responsibility — portability cross-check.** The CI sync-gate runs only on `ubuntu-latest` (per *Open questions resolved → 3*), so GNU-only constructs slipping into `gen-roadmap.sh` would not surface in CI; they would only break for macOS / BSD contributors running the generator locally. To compensate: the implementer **must** run `./scripts/gen-roadmap.sh` on macOS (or in a BSD-awk container, e.g. `docker run --rm -v "$PWD":/w -w /w alpine:latest sh -c 'apk add --no-cache bash && ./scripts/gen-roadmap.sh'` — alpine ships busybox awk, a closer match to BSD awk than GNU) and confirm the output is **byte-identical** to the Linux run (`sha256sum ROADMAP.md` matches). Document this verification in the progress file's per-subtask notes for task 5 (the "run the generator and commit `ROADMAP.md`" subtask).
- **Comment banner in `scripts/gen-roadmap.sh`** — add at the top of the script, immediately after the hashbang, a comment block listing the banned constructs so future editors get an in-source warning before reaching for the wrong tool. Required entries:
  - `sed -E` / `sed -r` — extended regex flags are GNU/macOS-only and not in POSIX. Use POSIX BRE; for character classes prefer POSIX `[[:alpha:]]` / `[[:digit:]]` / `[[:space:]]` over GNU shortcuts.
  - `awk` `gensub(...)` — GNU awk extension. Use chained `gsub(...)` calls instead.
  - `awk --re-interval` — already enabled by default in POSIX awk; the flag itself is GNU-only.
  - `sed -i` (in-place edit) — flag semantics differ between GNU (`-i`) and BSD (`-i ''`). Pipe through `tee` to a tempfile or rewrite the whole file via `printf`/`awk` redirection instead.
  - Bash `[[ ... ]]` test compounds — bash-specific; use POSIX `[ ... ]` or `case` instead.
  - Bash `(( ... ))` arithmetic — bash-specific; use POSIX `$(( ... ))` arithmetic expansion or `expr`.
  - `mapfile` / `readarray` — bash 4+ only; use a `while IFS= read -r line; do ...; done < file` loop instead.

  Reference: POSIX.1-2008 utilities reference (`awk`, `sed`, `sh`).

### AC7 (CI sync-gate)

- Verification: `actionlint .github/workflows/ci.yml` clean (AC11).
- Push the branch; observe the new `roadmap-sync` and `roadmap-sync-pass` jobs in the PR's status checks; both green.
- Drift test: temporarily edit `ROADMAP.md` by hand on a scratch branch, push, observe `roadmap-sync` red — confirms the gate fails on drift. Revert before committing.

### AC8 (doc-gate)

- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`. The new `lib.rs` content includes intra-doc links to per-sub-crate types (`[`quartzite_core::ObjectBase`]`, etc.); broken links fail the gate.

### AC9 (quickstart doctest)

- `cargo test --doc -p quartzite` — the `no_run` snippet must compile under default features.

### AC10 (standard gates)

- `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`, `cargo build -p quartzite --no-default-features` — all clean.

### AC11 (actionlint)

- `actionlint .github/workflows/ci.yml` clean after the new `roadmap-sync` job lands.

### AC12 (`document_features` placement preserved)

- Verification by file inspection: the `# Feature flags` heading + `#![doc = document_features::document_features!()]` line remain inline within the `//!` block, after all new content. No relocation.

## Open questions

None blocking implementation — the five spec-listed open questions are all resolved above (project-description shape: tagline + bullets; quickstart: counter-style snippet; CI sync-gate: ubuntu-latest only; CONTRIBUTING bullets: enumerated; generator format: distill, don't mirror).

Implementation-time choices the implementer is empowered to make without further design-review:
- Exact prose wording inside the `lib.rs` overview / per-concept-section bodies, provided each section has the structure specified (short prose + small example + sub-crate deep-link).
- The exact 5 per-concept section heading names — reasonable variants (`# Signals` vs `# Signals and slots`, `# Object tree` vs `# Object hierarchy`) are interchangeable; pick the form that reads cleanest.
- Per-section deep-link targets — sub-crate landing pages (`[`quartzite_core::signal`]`) vs. specific item pages (`[`quartzite_core::Signal`]`) — implementer picks based on what produces the most useful rustdoc landing for a reader who clicks through.
- The exact `actions/checkout` major version pin in the new `roadmap-sync` job — query the registry per AGENTS.md `## Dependency Versions` rule.

A reviewer who wants to constrain any of these can flag them in design-review.

## Post-merge owner actions

These steps fall outside this PR's diff and require the repo owner's attention after merge:

- **Add `roadmap-sync-pass` to branch-protection required checks.** The new aggregator job lands in `ci.yml` as part of this PR, but adding it to `origin`'s required-checks list is a `gh api`-or-UI action that only the repo owner can perform (it is not a workflow file edit). Without this step the gate runs but does not block PRs. One-time manual step on `master`'s branch-protection rule, identical in shape to the post-merge step PR #141 required for `windows-pass` and the `docs.yml` first-deploy step that required GitHub Pages source-config.
