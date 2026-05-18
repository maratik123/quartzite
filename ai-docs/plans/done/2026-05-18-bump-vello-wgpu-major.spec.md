# Bump vello 0.8 → 0.9 + wgpu 28 → 29 (coupled major)

**Source:** issue #439
**Date:** 2026-05-18
**Tracked in:** #439

## Scope

Coupled major bump of two renderer-side dependencies that must move together: `vello 0.8 → 0.9` and `wgpu 28 → 29` (vello 0.9 is built against wgpu 29). Includes any source-side fixes in `quartzite-renderer` to compile against the new APIs, and the Linux GPU snapshot golden regeneration that the rasterizer change will require.

1. `quartzite-renderer/Cargo.toml`:
   - `vello = "0.8"` → `vello = "0.9"`
   - `wgpu = "28"` → `wgpu = "29"`
   - Update the `# skrifa 0.42 coexists with vello's transitive skrifa 0.40` comment to reflect post-bump reality (vello 0.9 pulls `skrifa ^0.42.1` per its crates.io manifest, so the coexistence note no longer applies as written).
2. Run `cargo update` to refresh `Cargo.lock`; commit the lockfile alongside the manifest change.
3. Renderer source-code fixes for whatever API drift `cargo build` surfaces. Typical surfaces called out by the issue body:
   - `quartzite-renderer/src/vello_painter.rs` — `Scene` / brush / glyph paths (vello).
   - `quartzite-renderer/src/render_harness.rs` — adapter / device acquisition and offscreen readback (wgpu).
   - `RequestAdapterOptions` field set, `BufferUsages` / `TextureUsages` flag renames, `Surface::get_current_texture` error variants, `RenderParams` shape — fix each as encountered.
4. Regenerate Linux/vulkan GPU snapshot goldens after the source-side fixes compile and pass non-snapshot tests:
   - `quartzite-widgets/tests/snapshots/shared/*.png`
   - `quartzite-style/tests/snapshots/shared/*.png`
   - Regeneration mechanism: `scripts/update-snapshots.sh` (the script existing project tooling already uses).
5. Commit ordering for the PR (per issue #439): (a) `Cargo.toml` + `Cargo.lock` + renderer source fixes, then (b) regenerated Linux goldens, then (c) push as one PR. CI on the first push is expected to fail `gpu-tests` on Linux until goldens land in commit (b).
6. PR body must enumerate the wgpu / vello API changes that required source-side fixes (so reviewers can spot-check).

## Out of scope

- `bincode 2 → 3` — placeholder release (`compile_error!`); documented exception in `ai-docs/plans/done/2026-05-10-object-property-serialization-layer.design.md § Open questions`.
- `rstest 0.25 → 0.26` — addressed in #438.
- Re-bootstrapping Windows / macOS snapshot goldens. Per the #192 follow-up policy, those lanes remain `continue-on-error: true` until contributors with those platforms regenerate locally. This task does **not** flip that flag.
- Workspace-wide `peniko` constraint changes. Verified via crates.io `index.crates.io/ve/ll/vello`: vello 0.9 requires `peniko ^0.6.1`; our existing `peniko = "0.6"` constraint (in `quartzite-renderer`, `quartzite-style`, `quartzite-widgets`, `quartzite-paint`, `quartzite-paint-api`) accepts that under cargo caret-semantics. No workspace peniko bump required.
- Miri exclusion-list changes. Per issue #439's coordination note, the renderer crate is already excluded from Miri at the workflow level (`--exclude quartzite-renderer`), so #436's deny-list inversion is unaffected by this bump.

## Deferred

- Windows / macOS snapshot golden re-bootstrapping | wrong platform for this PR's author; lanes stay `continue-on-error` | tracked via #192 follow-up (already an open thread, no new issue needed)

## Key decisions

| Question | Decision |
|---|---|
| Cargo.toml version-constraint shape for the bumped crates | `vello = "0.9"`, `wgpu = "29"` — AGENTS.md *Dependency Versions* says `0.x` for `0.x.y` and `x` for `x.y.z`, never pin minor or patch. |
| Workspace peniko bump? | No. vello 0.9 still accepts `peniko ^0.6.1`; our existing `peniko = "0.6"` is compatible. |
| Snapshot-regeneration platform | Linux/vulkan only (the lane this contributor can drive). Win/Mac remain `continue-on-error`. |
| Single-PR vs split-PR | Single PR with the commit ordering laid out in Scope item 5 — Linux `gpu-tests` may go red on the first push and green after the golden-refresh commit. |
| MSRV impact | None. wgpu 29 MSRV = 1.87; vello 0.9 MSRV = 1.88; workspace `rust-version = "1.95"` already exceeds both. |

## Technical constraints

- Workspace MSRV is `1.95` (root `Cargo.toml`); both new majors are well below.
- `quartzite-renderer` is the **only** crate that depends directly on `wgpu` or `vello` — the blast radius for those two majors is renderer-internal. `peniko` is shared workspace-wide but is **not** being bumped (see Key decisions).
- `quartzite-renderer` is excluded from Miri at the workflow level — Miri lanes are unaffected.
- `cargo clippy --workspace --all-targets -- -D warnings` is the project-wide lint gate (AGENTS.md *Build & Test*); the bump must not introduce new clippy warnings.
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` (AGENTS.md *Build & Test*) — if any renderer item that gains a new generic parameter / new error variant requires a new `///` line, that line must be added in the same commit.
- AGENTS.md *API Stability* permits clean breaks of any renderer-side public API affected by the bump — no shims required.
- Win/Mac CI snapshot lanes retain `continue-on-error: true` per the #192 follow-up policy.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-renderer/Cargo.toml` declares `vello = "0.9"` and `wgpu = "29"`; `Cargo.lock` resolves both to their new majors. |
| AC2 | `cargo build` exit 0 on Linux. |
| AC3 | `cargo test --workspace` exit 0 on Linux (after the Linux golden regeneration in AC5 lands in the same PR). |
| AC4 | `cargo clippy --workspace --all-targets -- -D warnings` exit 0. |
| AC5 | Linux GPU snapshot goldens under `quartzite-widgets/tests/snapshots/shared/*.png` and `quartzite-style/tests/snapshots/shared/*.png` regenerated via `scripts/update-snapshots.sh` and committed in the same PR. |
| AC6 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` exit 0. |
| AC7 | `cargo build -p quartzite --no-default-features --features libm` exit 0 (no-std/libm path unaffected by the renderer-only bump, but verified). |
| AC8 | PR body documents each wgpu / vello API change that required a renderer source-side fix. |
| AC9 | Windows / macOS snapshot lanes still carry `continue-on-error: true` in `.github/workflows/` (unchanged from #192 follow-up policy). |
| AC10 | The post-bump `# skrifa …` comment in `quartzite-renderer/Cargo.toml` is updated or removed to reflect the new vello-transitive skrifa version (vello 0.9 → `skrifa ^0.42.1`). |

## Open questions

None at spec time. Any wgpu / vello API drift surfaced by `cargo build` is implementation-phase work and is captured by AC2 + AC8 (must compile; must be documented in the PR body).
