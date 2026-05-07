# Project docs: README description, facade `lib.rs` doc, badges, CONTRIBUTING, ROADMAP

**Source:** issue #60
**Date:** 2026-05-08
**Tracked in:** #60

## Scope

1. **README — project description / overview block** at the top, above the existing badges. Covers: GUI/object framework drawing on Qt's signals/slots + property/reflection model; Rust idioms throughout; current scope (object model, signal/slot, event loop, timers, per-thread loops); forward scope (paint-api, renderer, widgets per the `INDEX.md` dependency tree); tight non-goals (≤ 3 items).

2. **`quartzite/src/lib.rs` — comprehensive crate-level rustdoc**, modeled on the `tokio` crate's `lib.rs`:
    - **Overview prose**: what quartzite is, the conceptual model (objects + signals/slots + properties + event loop + threads).
    - **Runnable quickstart**: a fresh tokio-style minimal end-to-end snippet authored specifically for the rustdoc landing page (one derived `Object`, one signal, one slot, one `Application` loop). Annotated `no_run` so it compile-checks but does not execute (avoids the `Application::run` blocking-loop issue).
    - **Per-concept sections**: signals, properties, object tree, event loop, timers — each with a short example and a deep-link to the relevant sub-crate.
    - **Ecosystem map**: per-sub-crate ("when do I depend on `quartzite-core` directly vs. just the facade").
    - **Design notes**: `no_std` posture, threading model, cargo features (auto-rendered by `document_features` per `ai-docs/doc-convention.md` `## Feature flags rendering`).

3. **README badges** (immediately below the project description block): CI status, license. Existing codecov + GitHub Pages docs badges retained.

4. **`CONTRIBUTING.md`** at repo root, **standard depth**: excerpt the high-leverage AGENTS.md bullets external contributors actually need (branch-before-edit, no `git add -A`, `cargo fmt`/`clippy -D warnings`/`cargo test` gate, branch naming, /task workflow), with a "see `AGENTS.md` for the canonical workspace agent rules" pointer for the rest. Include the dual-license-contribution clause already in the README License section.

5. **`ROADMAP.md`** at repo root, **auto-generated** from `ai-docs/plans/INDEX.md` by `scripts/gen-roadmap.sh` (POSIX bash + awk/sed). Output covers: dependency tree, active/completed/deferred plan tables, suggested next steps. **CI sync-gate** added: a step (in existing `ci.yml` or a new tiny workflow) re-runs the generator and `git diff --exit-code ROADMAP.md`; fails the PR if the committed `ROADMAP.md` drifts from generator output.

## Out of scope

- **crates.io / docs.rs badges** — deferred to #136 (release workflow); both URLs only resolve after the first `cargo publish`. #136's body was already updated to absorb this.
- **Versioning-policy section in `lib.rs`** — deferred to #136; the policy text changes once the project is published, so writing it pre-publish is wasted churn.
- **Additional facade conveniences (re-exports, helper functions)** — added on demand as new crates land; no spec required.
- **API stability discussion** — covered by AGENTS.md `## API Stability` (no shims pre-crates.io).
- **Per-file source-license headers** — explicitly rejected; Rust ecosystem convention is `Cargo.toml` `license` field + repo-root `LICENSE-MIT` / `LICENSE-APACHE` only.

## Deferred

- **crates.io / docs.rs badges** | both URLs require first publish | tracked in #136 (already absorbs the badge work).
- **`lib.rs` versioning-policy section** | text changes post-publish | tracked in #136 (add when first publish lands).
- **Facade conveniences (re-exports / helpers)** | on-demand as new crates land | no separate issue; surfaces naturally when paint-api / widgets land.

## Key decisions

| Question | Decision |
|---|---|
| Scope shape for this `/task` | All five items (project description, lib.rs doc, badges, CONTRIBUTING, ROADMAP) in one PR. |
| `lib.rs` quickstart shape | Fresh tokio-style minimal snippet authored specifically for the rustdoc landing page (not a copy of `examples/`). |
| `lib.rs` quickstart doctest mode | `no_run` (compile-checked, doesn't execute — sufficient to keep the example from bit-rotting; matches `tokio` / `winit` for `Application::run`-style examples). |
| `ROADMAP.md` location | Repo root (`ROADMAP.md`) — GitHub repo view auto-detects + renders prominently. |
| `ROADMAP.md` generator | POSIX bash + awk/sed at `scripts/gen-roadmap.sh`, manual invocation (regenerate before commit). |
| `ROADMAP.md` sync-gate | CI step re-runs the generator and `git diff --exit-code ROADMAP.md`; fails the PR on drift. |
| README non-goals depth | Tight: 1–3 items max ("not a Qt port / binding", "no FFI / native deps", "GPU rendering arrives with #73 — not yet"). |
| `lib.rs` versioning-policy section | Defer to #136. |
| `CONTRIBUTING.md` depth | Standard — excerpt key AGENTS.md bullets + pointer for canonical reference. |
| Per-file source-license headers | None; Rust ecosystem convention. |

## Technical constraints

- **Doc-gate:** `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` must remain clean. `lib.rs` edits land under existing `#![deny(missing_docs)]`.
- The `lib.rs` quickstart doctest must compile under default features (`std + derive`); `no_run` lets it skip execution.
- The existing `#![doc = document_features::document_features!()]` invocation in `src/lib.rs` (PR #149 placement) must remain inline within the `//!` block, immediately after `# Feature flags`. New crate-level doc lands **before** the `# Feature flags` heading per `ai-docs/doc-convention.md` `## Feature flags rendering`.
- `ROADMAP.md` generator must parse markdown tables + the dependency-tree code-block from `ai-docs/plans/INDEX.md`. POSIX bash + `awk`/`sed`, no GNU-specific extensions.
- Sync-gate CI step: ensure portability across the multi-platform CI matrix (`ubuntu-latest`, `macos-latest`, `windows-latest` per #133 / PR #141). If bash-on-Windows variability becomes a problem, gate the sync-check on `ubuntu-latest` only — design agent picks.
- License badge: static `https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue` (pre-publish; no `crates.io` license-detection yet).
- CI badge: `https://github.com/maratik123/quartzite/actions/workflows/ci.yml/badge.svg`.
- AGENTS.md `## Workflow` rule: `cargo build` before each commit to refresh `Cargo.lock`. Apply on every commit during implementation.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `README.md` has a project-description block at the top covering: GUI/object framework, Qt-inspired design lineage (signals/slots + property/reflection), Rust-idiomatic, current scope (object model + signal/slot + event loop + timers + per-thread loops), forward scope (paint-api / renderer / widgets), tight non-goals (≤ 3 items). |
| AC2 | `README.md` has CI-status and license badges; existing codecov + docs badges retained; crates.io and docs.rs badges absent (deferred to #136). |
| AC3 | `quartzite/src/lib.rs` crate-level rustdoc is comprehensive: overview prose, runnable quickstart (`no_run` doctest, fresh tokio-style minimal snippet), 5 per-concept sections (signals, properties, object tree, event loop, timers — each with example + sub-crate deep-link), ecosystem map, design notes (no_std posture, threading model, cargo features). Versioning-policy section is **absent** (deferred to #136). |
| AC4 | `CONTRIBUTING.md` exists at repo root, standard depth: excerpts AGENTS.md highlights (branch-before-edit, no `git add -A`, `cargo fmt`/`clippy -D warnings`/`cargo test` gate, branch naming, `/task` workflow), with a pointer to `AGENTS.md` as canonical reference, and a dual-license-contribution clause. |
| AC5 | `ROADMAP.md` exists at repo root, generated by `scripts/gen-roadmap.sh` from `ai-docs/plans/INDEX.md`. Output covers: dependency tree, active/completed/deferred plan tables, suggested next steps. |
| AC6 | `scripts/gen-roadmap.sh` is POSIX bash + awk/sed; executable bit set; deterministic output (idempotent re-run produces identical bytes). |
| AC7 | CI sync-gate: a step in `.github/workflows/ci.yml` (or a new tiny workflow) re-runs `scripts/gen-roadmap.sh` and `git diff --exit-code ROADMAP.md`; fails on drift. |
| AC8 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` clean (matches CI doc-gate). |
| AC9 | The `lib.rs` quickstart doctest compiles cleanly under default features (`cargo test --doc -p quartzite`). |
| AC10 | All standard gates clean: `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`, `cargo build -p quartzite --no-default-features`. |
| AC11 | `actionlint` clean on any modified `.github/workflows/*.yml` (per AGENTS.md `## Build & Test → Workflow files`). |
| AC12 | Existing `document_features::document_features!()` invocation in `src/lib.rs` placement preserved per `ai-docs/doc-convention.md` `## Feature flags rendering` (inline within `//!` block, immediately after `# Feature flags` heading). New crate-level doc lands **before** the `# Feature flags` heading. |

## Open questions

None blocking implementation. The design agent will resolve:

- Exact wording / structure of the project-description block (3–5 paragraphs vs. tight tagline + bullet list).
- Exact `lib.rs` quickstart content shape — likely a counter-style example with one signal/slot pair, or a config-style example with one property. Either fits `no_run` and the conceptual focus.
- Whether the CI sync-gate runs on all OS matrix entries or `ubuntu-latest` only (depends on bash-on-Windows behavior of the generator's `awk`/`sed` invocations).
- Exact set of AGENTS.md bullets to excerpt into `CONTRIBUTING.md` (a "high-leverage" judgment call per the standard-depth choice).
- Generator format choices for `ROADMAP.md`: how literally to mirror `INDEX.md` vs. how much to distill / re-narrate.
