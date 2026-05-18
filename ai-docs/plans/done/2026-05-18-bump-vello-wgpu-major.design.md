# Design: Bump vello 0.8 → 0.9 + wgpu 28 → 29 (coupled major)

**Issue:** #439
**Date:** 2026-05-18
**Spec:** [`ai-docs/plans/2026-05-18-bump-vello-wgpu-major.spec.md`](./2026-05-18-bump-vello-wgpu-major.spec.md)

## Approach

Bump both crates in lock-step in one PR with a deliberate three-commit shape (manifest+lockfile+source, then snapshot regen, then any follow-up touch-ups). Linux/vulkan is the only lane we regenerate; macOS/Windows stay `continue-on-error`.

### Live state confirmed (verified via crates.io sparse index)

- `vello = 0.9.0` is the latest stable.
- `wgpu = 29.0.3` is the latest stable.
- `vello 0.9.0` declares `wgpu = "^29.0.3"`, `peniko = "^0.6.1"`, `skrifa = "^0.42.1"` (per `index.crates.io/ve/ll/vello`). After the bump:
  - The renderer's existing `skrifa = "0.42"` direct dep and vello's transitive `skrifa ^0.42.1` converge on `skrifa 0.42.x`. The current `# skrifa 0.42 coexists with vello's transitive skrifa 0.40` comment in `Cargo.toml` becomes obsolete (AC10).
  - The workspace-wide `peniko = "0.6"` constraint accepts `^0.6.1` under caret semantics — no workspace bump needed.

### API drift to expect (renderer-only)

The renderer's wgpu/vello surface, enumerated via `grep` over `quartzite-renderer/src/`:

| File | Surface | wgpu 28→29 / vello 0.8→0.9 risk |
|---|---|---|
| `render_harness.rs`, `application_builder.rs`, `application.rs`, `window_registry.rs` (tests), `wrapped_handler.rs` (tests) | `wgpu::Instance::default()` (8 call sites total: 1 in `render_harness.rs:148`, 1 in `application_builder.rs:139`, 2 in `window_registry.rs:249,260` test code, 3 in `wrapped_handler.rs:322,561,582` test code; `application.rs` re-exports the builder path) | **Still present** in v29 (routes to `InstanceDescriptor::new_without_display_handle()`). Safe across both prod and test sites. The v29 changelog removes `InstanceDescriptor::default()` / `from_env_or_default` and replaces them with explicit `new_with…` constructors, but `Instance::default()` is preserved. |
| `render_harness.rs` | `instance.request_adapter(&RequestAdapterOptions::default())` | Signature unchanged: `Result<Adapter, RequestAdapterError>`. `RequestAdapterOptions::default()` still valid. Safe. |
| `render_harness.rs` | `adapter.request_device(&DeviceDescriptor { … })` | Signature unchanged: `Result<(Device, Queue), RequestDeviceError>`. The `DeviceDescriptor` struct itself has no field renames in v29 that affect us (we only set `label` + `..Default::default()`). Safe. |
| `render_harness.rs` | `TextureDescriptor { usage: STORAGE_BINDING \| COPY_SRC, … }` | Flag names unchanged. Safe. |
| `render_harness.rs` | `BufferDescriptor { usage: MAP_READ \| COPY_DST, … }` | Flag names unchanged. Safe. |
| `render_harness.rs` | `buffer_slice.map_async(MapMode::Read, …)` | `MapMode::Read` unchanged. Safe. The v29 `WriteOnly<[u8]>` change is **`MapMode::Write`-only** — does not affect our read path. |
| `render_harness.rs` | `device.poll(PollType::wait_indefinitely())` | Method preserved in v29 (`wgpu-types::PollType::wait_indefinitely() -> Self`). Safe. |
| `render_harness.rs` | `TexelCopyTextureInfo` / `TexelCopyBufferInfo` / `TexelCopyBufferLayout` | Already migrated to the post-v27 names (the wgpu 27 rename from `ImageCopy*` happened pre-#439). v29 keeps these names. Safe. |
| `render_harness.rs` | `RendererOptions { use_cpu, antialiasing_support, num_init_threads, pipeline_cache }` | vello 0.9 keeps the **same four public fields** (verified against `vello/src/lib.rs:373` in the v0.9.0 tag). Safe. |
| `render_harness.rs` | `RenderParams { base_color, width, height, antialiasing_method }` | vello 0.9 keeps the **same four public fields**. Safe. |
| `render_harness.rs` | `renderer.render_to_texture(device, queue, scene, view, &RenderParams { … })` | Signature unchanged in vello 0.9. Safe. |
| `vello_painter.rs` | `Scene::new()`, `Scene::reset()`, `Scene::fill`, `Scene::stroke`, `Scene::push_clip_layer`, `Scene::pop_layer`, `Scene::draw_image`, `Scene::draw_glyphs(...).font_size(...).transform(...).normalized_coords(...).brush(...).draw(...)` | vello 0.9's **only** `Scene` change is a non-breaking addition of `DrawGlyphs::font_embolden` / `DrawGlyphs::brush_transform`. Existing builder chains still compile (the new methods are optional). Safe. |
| `vello_painter.rs` | `peniko::*` types (`Color`, `Brush`, `Gradient`, `ColorStop`, `DynamicColor`, `ImageData`, `Blob`, etc.) | Workspace `peniko = "0.6"` already accepts `^0.6.1`, which is what vello 0.9 pulls. No source change needed. |
| `vello_painter.rs` | `vello::Glyph` | Re-exported from `vello_encoding`. Still `pub use vello_encoding::Glyph` in v0.9. Safe. |
| `window_registry.rs` | `wgpu::Instance`, `wgpu::Surface<'static>`, `instance.create_surface(window)`, `wgpu::CreateSurfaceError` | `create_surface` signature unchanged. `Surface<'static>` and `CreateSurfaceError` unchanged. Safe — including the existing field-drop-order safety comment. |
| `wrapped_handler.rs` | `vello::Scene` | See `vello_painter.rs` row. Safe. |
| `application_builder.rs` / `application.rs` | `wgpu::Instance::default()` only | See first row. Safe. |
| `error.rs` | `#[error("surface creation failed: {0}")] Surface(#[from] wgpu::CreateSurfaceError)` | `CreateSurfaceError` still implements `Display` in v29. Safe. |

**Confidence:** based on the wgpu v29.0.0 changelog (the breaking-changes section enumerates: `Surface::get_current_texture` enum rewrite, `InstanceDescriptor` constructor reshape, optional bind-group-layout entries in `PipelineLayoutDescriptor`, MSRV bump to 1.87, `WriteOnly<[u8]>` for write-mapped buffers, `depth_write_enabled` / `depth_compare` becoming `Option<…>`, `primitive_index` becoming a WGSL `enable` extension, `max_inter_stage_shader_components` → `max_inter_stage_shader_variables`) — **none of these touch the renderer's current surface.** The renderer doesn't call `get_current_texture`, doesn't construct an `InstanceDescriptor` explicitly, doesn't build pipeline layouts, doesn't write-map buffers, doesn't construct depth/stencil state, doesn't write WGSL, and doesn't query inter-stage-component limits. The only conceivable surprise is if vello's `Renderer::new` constructor signature shifted (it didn't — verified above).

**Expected outcome of step 1:** `cargo build -p quartzite-renderer` is green after the manifest+lockfile bump with **zero source edits**. Spec AC2 + AC8 still leave room for any drift cargo surfaces — if a surprise variant lands, the subtask 3 step handles it case-by-case.

### Alternatives evaluated

| Alternative | Verdict |
|---|---|
| **A. Single PR, three commits (manifest → goldens → review touchups).** | **Chosen.** Matches the spec's commit-ordering directive. CI on the first commit may be red on `gpu-tests` (Linux); the second commit lands the goldens and CI goes green. Reviewers see the source diff isolated from the binary diff (goldens are PNGs and hard to review inline). |
| B. Split into two PRs — one for the lockfile/source bump, one for the goldens. | Rejected. The bump can't merge with red `gpu-tests` on master (branch protection), and the goldens can't merge before the bump (no code change to justify the binary diff). The PRs would be entangled. Worth the cost of one transiently-red CI run on push 1. |
| C. Lockfile-only bump (no source pass), let cargo pick versions opportunistically. | Rejected. `quartzite-renderer/Cargo.toml` pins the major (`vello = "0.8"`, `wgpu = "28"`) — `cargo update` alone cannot cross a major boundary. The manifest **must** be edited. |
| D. Skip the snapshot regen, mark Linux `gpu-tests` `continue-on-error` for one commit. | Rejected. The spec explicitly forbids flipping the Linux flag (AC9) — the regen is mandatory and the per-bump drift is precisely what `update-snapshots.sh` exists to handle. |
| E. Defer the AC10 comment cleanup to a follow-up PR. | Rejected. The comment is in the same file as the `vello`/`wgpu` lines and is factually wrong post-bump. Cost to fix is one line; cost to leave is reviewer confusion on the next renderer-touching PR. Fold it in. |

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Bump `Cargo.toml` deps + refresh `Cargo.lock`. Set `vello = "0.9"` and `wgpu = "29"`. Update the stale skrifa-coexistence comment to reflect post-bump reality (AC10 — vello 0.9 now pulls `skrifa ^0.42.1`, converging with our direct `skrifa = "0.42"`; the old "0.40 coexists with 0.42" framing no longer applies). Run `cargo update -p vello -p wgpu` then `cargo build -p quartzite-renderer` to confirm the lockfile lines up. | `quartzite-renderer/Cargo.toml`, `Cargo.lock` | — |
| 2 | Run `cargo build` workspace-wide and `cargo build -p quartzite --no-default-features --features libm` (AC2 + AC7). If either fails, apply renderer source-side fixes for whatever API drift the compiler surfaces — most likely candidates per § Approach are `vello_painter.rs` (Scene/brush/glyph paths) and `render_harness.rs` (adapter/device/usage flag drift), though the changelog audit suggests zero churn. Each drift item gets a one-line comment in the commit message so AC8's PR-body enumeration can reference it. | `quartzite-renderer/src/*.rs` (only files cargo flags) | 1 |
| 3 | Run `cargo test --workspace` on Linux. Non-snapshot tests must pass; the GPU-snapshot suites (`quartzite-widgets/tests/snapshots.rs`, `quartzite-style/tests/snapshots.rs`) are expected to fail because vello 0.9's rasterizer drift (bicubic `ImageQuality::High`, half-pixel-offset fix in #1606, clip-layer fixes in #1637) will produce different pixels than the current goldens. Capture which goldens diverge to inform subtask 4's regen scope. | All test files — observe-only | 2 |
| 4 | Regenerate Linux/vulkan goldens via `scripts/update-snapshots.sh` (defaults to vulkan on Linux). The script writes into `<crate>/tests/snapshots/vulkan/`. Move the regenerated PNGs into `shared/` per the spec's AC5 (which targets `shared/`, since Linux is the only lane being refreshed and the existing goldens live in `shared/`). Verify exactly the files listed in spec AC5 are regenerated: `quartzite-widgets/tests/snapshots/shared/{box_layout,button,grid_layout,label,line_edit}.png` and `quartzite-style/tests/snapshots/shared/{button_checked,button_disabled,button_focused,button_hovered,button_idle,button_pressed,label,scroll_area_chrome,text_edit_plain,text_edit_read_only}.png`. Re-run `cargo test --workspace` and confirm exit 0. | `quartzite-widgets/tests/snapshots/shared/*.png`, `quartzite-style/tests/snapshots/shared/*.png` | 3 |
| 5 | Run the full gate: `cargo clippy --workspace --all-targets -- -D warnings` (AC4), `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` (AC6), `cargo fmt -- --check`. Address any new lint/doc warnings introduced by the bump (none expected from the changelog audit; if `vello` re-exports widen and a previously-redundant `use` becomes `unused_imports`, fix it in this subtask). Push and write the PR body that enumerates the wgpu/vello API drift handled by subtask 2 (AC8). | `quartzite-renderer/src/*.rs` (only if a lint fires) | 4 |

## Handoff plan

`M = 5`, split into two groups of 3 + 2. Subtask 1 carries the most blast-radius surprise (lockfile resolution); subtask 4 carries the binary-diff bulk. Splitting between subtasks 3 and 4 puts the natural CI-failure boundary (the test run that confirms which goldens drift) at the group boundary, so the handoff happens with full diagnostic context but before the goldens land.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry); `/task` enters subtask 1 with fresh context.
- **Group A:** subtasks 1–3 — manifest bump, source fixes for any surfaced drift, and full `cargo test --workspace` run that quantifies which goldens diverge. Group ends with a known-failing snapshot suite and full visibility into what subtask 4 must regenerate.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — golden regen and final gate runs — terminal group (2 subtasks; within the 1..=3 range).

## Risks

- **Cargo.lock conflict on rebase.** If another PR lands a workspace dep update before this merges, `Cargo.lock` may need a re-resolve. **Mitigation:** re-run `cargo update -p vello -p wgpu && cargo build` after rebase; `Cargo.lock` is committed and reviewed.
- **vello 0.9 produces visually identical pixels — no goldens drift.** Possible because the changelog highlights (bicubic high-quality sampling, half-pixel-offset fix, atlas-residency reuse, inactive `clip_leaf` lane fix) only fire for code paths our current goldens don't exercise (we draw plain colour fills, strokes, and ASCII glyphs at 1× scale; no high-quality image sampling, no transformed images, no deeply-nested clips). **Mitigation:** subtask 3 reads `cargo test --workspace` exit code — if every snapshot test passes, subtask 4 is a no-op and we proceed straight to subtask 5. Spec AC5 says "regenerated if regeneration was needed" implicitly; if no goldens drift, the commit is dropped and the PR ships with subtasks 1+2+5 only. _The spec is explicit ("regenerated via update-snapshots.sh and committed in the same PR") — but if there is nothing to regenerate because the rasterizer is byte-stable for our scenes, no PNG commit is required._ **PR-body disclosure:** in the byte-stable case, subtask 5's PR body must include an explicit line documenting the byte-stable outcome (e.g. _"vello 0.9 produced byte-identical pixels for all 15 existing goldens; no snapshot commit required. AC5 satisfied vacuously."_) so reviewers can see AC5 was checked rather than silently dropped.
- **Drift surface that the changelog audit missed.** Low probability — the audit covered every wgpu/vello call site in the renderer — but a non-publicised internal-type change in `vello::Glyph` or `peniko::Brush::Solid`'s shape could surface. **Mitigation:** subtask 2 has license to apply renderer source fixes for whatever cargo flags; each fix is recorded for the PR body (AC8).
- **Linux GPU adapter unavailable during regen.** `update-snapshots.sh` defaults to vulkan on Linux but requires a working ICD. **Mitigation:** the dev machine for this task is Linux/X11 with Mesa; the existing snapshot test path already validates that path works (it's how the current goldens were generated). If GPU init fails, the harness's `harness_or_skip` helper would have already skipped — i.e. there'd be no snapshot test to fail in subtask 3, which would itself be a diagnostic signal.
- **macOS / Windows snapshot lanes diverge silently.** They run `continue-on-error: true` per the #192 policy (spec AC9), so silent divergence is the accepted v1 stance. **Mitigation:** none required — AC9 explicitly preserves this. Document in the PR body that Win/Mac lanes were not refreshed.
- **AGENTS.md "axiom" gates.** `actionlint` is irrelevant (no workflow file touched). The 40k-char instruction-file cap is irrelevant (no instruction file touched). The Propagation Rule is irrelevant (no instruction-file edit). The `self-review` axiom applies to every code-producing commit — subtasks 2/4/5 each get the `self-review` spawn before push, per `/task` Step 10. Subtasks 1 (Cargo.toml + Cargo.lock + comment edit; no `.rs` diff) and 3 (observation only, no commit) fall under the AGENTS.md self-review axiom's last row ("Docs-only / instruction-file-only commit — optional"); self-review is **optional but still recommended** for subtask 1 because the comment edit lives on the same `Cargo.toml` line that controls the bump, and a misleading comment surviving the PR is a reviewer-confusion footgun.

## Test Design

This is a dependency bump, not a feature; no new test scenarios are introduced. The validation matrix is:

- **Compile (`cargo build`, `cargo build -p quartzite --no-default-features --features libm`)** — covers AC2 + AC7. Already required by `/task` Step 9 + Step 12.
- **Unit + integration tests (`cargo test --workspace`)** — covers AC3. The renderer's own `#[cfg(test)] mod tests` in `vello_painter.rs` and `render_harness.rs` exercise: painter stack invariants, brush classification (4 variants + `Unknown`), text path with italic/underline/strikethrough, `RenderHarness` zero-extent error, accessor + Debug, no-op render-to-clear-colour smoke. These pass without touching goldens. The GPU snapshot suites (`quartzite-widgets/tests/snapshots.rs`, `quartzite-style/tests/snapshots.rs`) are the visual regression net — they verify the regen in subtask 4 landed correctly.
- **Lint (`cargo clippy --workspace --all-targets -- -D warnings`)** — covers AC4.
- **Doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`)** — covers AC6.
- **Format (`cargo fmt -- --check`)** — implicit `/task` Step 9 gate.

No new fixtures, no new helpers. The existing snapshot regen script is the only "test fixture mutation" tool used.

## Open questions

None. The spec resolved them upfront (single PR vs. split, Linux-only regen, peniko constraint, MSRV) and the changelog audit confirmed the renderer surface doesn't touch any of the wgpu 29 breaking-change list items.
