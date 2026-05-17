# Tighten clippy: pedantic + nursery + size-aware lints

**Source:** issue #423
**Date:** 2026-05-17
**Tracked in:** #423

## Scope

1. **Enable four clippy lint groups / lints workspace-wide** via Rust 1.74+ `[workspace.lints.clippy]` in the root `Cargo.toml`:
   - `clippy::pedantic` — `warn`, `priority = -1`
   - `clippy::nursery` — `warn`, `priority = -1`
   - `clippy::large_stack_frames` — `warn` (listed separately so it survives a future `nursery` rollback)
   - `clippy::large_stack_arrays` — `warn` (listed separately so it survives a future `pedantic` rollback)
2. **Opt every workspace member in** by adding `[lints] workspace = true` to each member crate's `Cargo.toml` and the root `quartzite` package itself. Members at time of writing:
   - `quartzite-core`, `quartzite-macros`, `quartzite-runtime`, `quartzite-geometry`, `quartzite-events`, `quartzite-event-types`, `quartzite-paint-api`, `quartzite-paint`, `quartzite-renderer`, `quartzite-style-types`, `quartzite-style`, `quartzite-style-dispatch`, `quartzite-widgets`, plus the root `quartzite` package.
3. **Materialise the two size-aware thresholds explicitly** in a new `clippy.toml` at workspace root:
   - `stack-size-threshold = 524288` (512 KiB — clippy default)
   - `array-size-threshold = 524288` (512 KiB — clippy default)
4. **Curate a workspace-level allow-list** for the new lints that are wrong-for-this-codebase. Allow entries live in `[workspace.lints.clippy]` (Cargo TOML), each with a one-line comment justifying the allow. Design phase audits the actual first-run clippy output and decides per-lint; the issue's *Expected allow-list* (pedantic + nursery candidates) is the starting hypothesis, not a contract.
5. **Make every clippy warning produced by the new groups go away** — either by fixing the code (idiomatic shape the lint suggests) or by adding a justified workspace-level allow — so that `cargo clippy --workspace -- -D warnings` stays green on the post-PR tree.
6. **Update AGENTS.md *Code Style → Linter posture* row** (and the corresponding line in `ai-docs/code-style.md § Linter posture`) to mention the `[workspace.lints]` mechanism so future readers know where workspace-wide lint policy lives.

## Out of scope

- Promoting the new lints to `level = "deny"` directly in `[workspace.lints.clippy]`. The existing `cargo clippy --workspace -- -D warnings` CLI gate continues to be the single point of escalation.
- Per-crate allow-list refinement (vs workspace-level uniformity). v1 picks workspace-level for every allow; per-crate split is a follow-up only if a single crate consistently violates a lint that's healthy elsewhere.
- The `clippy::restriction` lint group. Project-style restrictions already live in `panic-index.md` discipline + AGENTS.md *Library safety idioms*.
- New CI workflow file. The existing `ci.yml` clippy job picks up `[workspace.lints]` automatically — no workflow edit needed.
- Rust MSRV change. The workspace MSRV is already `1.95` (verified in root `Cargo.toml`), well above the `[workspace.lints]` requirement of Cargo 1.74.
- Tightening the size-aware thresholds below the 512 KiB clippy default. v1 keeps defaults; threshold tuning is a deferred follow-up if first-run data warrants.
- Cleaning up pre-existing per-crate `#![warn(clippy::missing_errors_doc)]` / `#![warn(clippy::missing_panics_doc)]` / `#![warn(clippy::doc_markdown)]` / `#![warn(clippy::undocumented_unsafe_blocks)]` attributes in `lib.rs` files that become redundant under `pedantic = warn`. They become no-ops but are harmless; removal is a separate follow-up.
- Enforcement of the new lints on the `cargo build -p quartzite --no-default-features --features libm` derive-free path. AGENTS.md *Build & Test* only requires `cargo clippy --workspace -- -D warnings` on the default-feature build; the no-default-features path remains a build-only gate.

## Deferred

- what | why | separate issue needed?
- Per-crate allow-list refinement | v1 workspace-level uniformity, only split if a crate consistently violates a healthy-elsewhere lint | yes, if/when data warrants
- Threshold tightening below 512 KiB defaults (e.g., 64 KiB for `quartzite-paint`) | first-run data needed to identify candidate crates | yes, post-merge follow-up
- Removing now-redundant per-crate `#![warn(clippy::...)]` attributes superseded by `pedantic = warn` | cleanup; behaviour-neutral | yes, separate cleanup PR
- Tightening to `level = "deny"` directly in `[workspace.lints.clippy]` | retains feature-branch `cargo clippy --workspace` (without `-D warnings`) workflow | yes, only if `-D warnings` gate is ever removed
- Enabling `clippy::restriction` subset (e.g., `clippy::unwrap_used`, `clippy::dbg_macro`) | overlaps with `panic-index.md` discipline; separate decision | yes, if explicit project-style enforcement is desired beyond panic-index

## Key decisions

| Question | Decision |
|---|---|
| Lint level for the four additions | `warn`, escalated to `deny` via the existing `-D warnings` CI gate. Keeps feature-branch `cargo clippy --workspace` (no `-D warnings`) workflow viable. |
| `priority` on the two group enables (`pedantic`, `nursery`) | `priority = -1` so specific `clippy::* = "allow"` entries (default priority `0`) override the group `warn`. Cargo's order-dependent eval otherwise produces surprising results. |
| `large_stack_frames` / `large_stack_arrays` listed separately even though each is a group member | Survives a future per-group rollback. If `nursery` becomes too noisy and is turned off, `large_stack_frames` stays on; same for `large_stack_arrays` under `pedantic`. |
| Threshold values for size-aware lints | 512 KiB clippy defaults (`stack-size-threshold = 524288`, `array-size-threshold = 524288`) made explicit in a new `clippy.toml` so they're easy to find and tune. |
| Allow-list scope (workspace-wide vs per-crate) | Workspace-wide in `[workspace.lints.clippy]`. Uniformity; per-crate refinement is a follow-up only when a single crate consistently violates a healthy-elsewhere lint. |
| Allow-list comment requirement | Each `clippy::* = "allow"` entry MUST carry a one-line `#`-comment justifying the allow (matches `AGENTS.md` *Linter posture*: "no blanket `#[allow]` without justification"). Where the project's own infrastructure overlaps a pedantic lint (e.g., `missing_panics_doc` vs `panic-index.md`; `large_stack_frames`/`large_stack_arrays` vs `unsafe-index.md`), the comment cross-references the relevant doc. |
| Allow-list contents | Design phase audits the actual first-run clippy output and decides per-lint. The issue's *Expected allow-list* is the starting hypothesis (and the design agent should explain divergence from it). |
| Roll-out shape | Single PR, big-bang tightening — pre-publish, no downstream consumers, no need for a phased "warn first, then deny later" introduction. |
| AGENTS.md / `code-style.md` update | Update only the *Linter posture* row(s) to mention `[workspace.lints]` as the location of workspace-wide policy. Do not relocate or rewrite the existing "strict clippy enforced (`-D warnings`); no blanket `#[allow]` without justification" wording. |
| Pre-existing per-crate `#![warn(clippy::...)]` attributes (e.g., in `src/lib.rs`) | Leave in place. They are no-ops under `pedantic = warn` but harmless. Cleanup is a separate follow-up (see *Deferred*). |
| Root `quartzite` package `[lints] workspace = true` | YES — the root package is itself a workspace member (it carries `[package]` + `src/lib.rs`) and must opt in alongside the 13 leaf crates. |
| `[lints] workspace = true` exemptions | None. Every member crate opts in; if a crate cannot survive the workspace lints, surface in the PR description per AC8 rather than silently exempt it. |
| Surfacing un-fixable / un-justifiable first-run hits | If a new-lint hit can't be cleanly fixed AND the workspace-allow comment isn't trivially justified, surface the case in the PR description for reviewer judgement before adding the allow (AC8). |

## Technical constraints

- **Cargo version:** `[workspace.lints]` requires Cargo 1.74+. Workspace MSRV is `1.95` per root `Cargo.toml` — well above the floor. No MSRV change needed.
- **CI mechanism:** the existing `ci.yml` clippy job runs `cargo clippy --workspace -- -D warnings`. `[workspace.lints]` flows through this command unchanged — no new workflow file, no new CLI flag, no `actionlint` work.
- **`actionlint` gate:** N/A — no workflow file changes anticipated.
- **Allow-list comment format:** comments inside `[workspace.lints.clippy]` use TOML `#`-line-comments above the relevant lint entry (TOML doesn't support inline value comments cleanly; one comment per allow line is fine).
- **`clippy.toml` placement:** workspace root, not per-crate. Clippy auto-discovers `clippy.toml` from the workspace root.
- **No backward-compat concerns:** pre-`cargo publish`, no downstream consumers (AGENTS.md *API Stability* AXIOM applies — the design phase is free to refactor any code the new lints flag).
- **Tracing / `_Simple._` markers:** if a `clippy::pedantic` fix changes the shape of a marked-simple fn, the cascade rule in AGENTS.md *Code Style → `#[inline]` and the `_Simple._` doc tag* applies (strip + cascade re-test). Design phase considers this when planning fixes.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Root `Cargo.toml` carries a `[workspace.lints.clippy]` section enabling `pedantic`, `nursery`, `large_stack_frames`, `large_stack_arrays` at `warn`. The two group entries (`pedantic`, `nursery`) carry `priority = -1`. |
| AC2 | Root `Cargo.toml` `[workspace.lints.clippy]` section carries every `clippy::* = "allow"` entry needed to keep `cargo clippy --workspace -- -D warnings` green, each with a one-line `#`-comment justifying the allow. |
| AC3 | A new `clippy.toml` at workspace root carries `stack-size-threshold = 524288` and `array-size-threshold = 524288`. |
| AC4 | Every workspace member crate's `Cargo.toml` (all 13 leaf crates **and** the root `quartzite` package) carries `[lints] workspace = true`. |
| AC5 | `cargo clippy --workspace -- -D warnings` runs clean against the post-PR tree. |
| AC6 | `cargo build`, `cargo test --workspace`, `cargo fmt -- --check`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all PASS unchanged. |
| AC7 | The `AGENTS.md` *Code Style → Linter posture* row AND the `ai-docs/code-style.md § Linter posture` section both mention the `[workspace.lints.clippy]` mechanism (location of workspace-wide lint policy + `clippy.toml` for thresholds). |
| AC8 | If any of the four new lint additions fires against existing code in a way that can't be cleanly fixed AND the workspace-allow comment isn't trivially justified, the PR description enumerates the case(s) for reviewer judgement before adding the allow. |
| AC9 | `cargo build -p quartzite --no-default-features --features libm` (the derive-free / no_std path) still compiles. (Existing AGENTS.md gate; verifying it does not regress under the new lint table.) |

## Open questions

- _(none — the issue body is self-contained, mechanisms are pinned, allow-list curation is explicitly design-phase work)_
