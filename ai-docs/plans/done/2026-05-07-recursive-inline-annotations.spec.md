# Annotate concrete simple fns surfaced by recursive `#[inline]` rule

**Source:** issue #115
**Date:** 2026-05-07
**Tracked in:** #115

## Scope

Apply the recursive `#[inline]` rule (AGENTS.md → *Code Style* → `#[inline]` and the `_Simple._` doc tag) across the workspace and annotate every concrete fn that newly qualifies as **simple** under the relaxed definition. *Simple* = no branches/loops AND ≤ 1 call to a non-simple fn, applied transitively (calls to other simple fns are "free").

1. **Named targets — `#[inline]` attribute on each:**
   - `ObjectExt::id` (`quartzite-core/src/traits.rs:227-229`) — body is `self.object_base().id()`; both callees `#[inline]`.
   - `ObjectExt::name` (`quartzite-core/src/traits.rs:244-246`) — same single-delegation shape.
   - `ObjectExt::is_on_current_thread` (`quartzite-core/src/traits.rs`, ~line 260) — single delegation.

2. **Sibling `ObjectExt` default-method audit.** Re-evaluate every other default method on `ObjectExt` against the recursive simple test; add `#[inline]` to each concrete one that now qualifies.

3. **Workspace-wide concrete sweep.** Across **all crates** in the workspace (`quartzite`, `quartzite-core`, `quartzite-runtime`, `quartzite-paint`, `quartzite-geometry`, `quartzite-codegen`, …), annotate every concrete simple fn currently missing `#[inline]`. Typical targets:
   - field getters (`self.field`)
   - trivial wrappers (`.as_deref()`, single delegation)
   - `Default::default()` impls that call `Self::new()` or are struct-literal constructors
   - `const fn` struct-literal constructors
   - any fn whose body is one call into another simple fn

4. **Opportunistic `_Simple._` doc-tag additions** for **generic fns** and **trait method declarations** discovered during the audit/sweep that satisfy the recursive simple test (the doc-tag row of the AGENTS.md table). Add the tag in this PR, do not defer.

5. **Full cascade until quiescent.** After each annotation, `rg <fn-name>` for callers and re-evaluate every caller against the recursive rule. If a caller now qualifies, mark it; cascade depth is unbounded — repeat until no more callers qualify.

## Out of scope

- **Marker stripping for no-longer-simple fns.** Removing `#[inline]` / `_Simple._` from fns whose bodies became non-simple (separate concern; this PR is annotation-only).
- **Refactoring** any fn body to make it simple. Annotate only what already qualifies.

> **In scope (clarification of AC5, broad reading):** *swapping* a wrong-row marker on a fn that **does** satisfy the recursive simple test is in scope. Example: a generic fn currently carrying `#[inline]` that should instead carry `_Simple._` per the AGENTS.md row table. Such a fn "lacks the appropriate marker" even though it has *a* marker. Strip the wrong-row marker and add the right-row one in the same edit.
- **Codegen-output marker mirroring** changes in `quartzite-codegen` beyond what naturally falls out of the sweep — i.e. if codegen *emits* fns that should now carry the marker, the codegen itself is updated; but no broader codegen restructuring.
- **API renames / signature changes.** Annotation-only PR.

## Deferred

None.

## Key decisions

| Question | Decision |
|---|---|
| Sweep coverage | Entire workspace, not just `quartzite-core`. |
| `_Simple._` doc-tag additions found en passant | In scope — add in this PR. |
| Cascade depth | Full cascade until quiescent (no per-PR bound). |
| Generic fns vs concrete fns | Concrete row → `#[inline]` attribute; generic row + qualifying trait method declarations → `_Simple._` doc tag. Mutually exclusive per AGENTS.md table. |

## Technical constraints

- The recursive *simple* test must be applied transitively — a caller's classification depends on the *current* status of its callees. Re-evaluate after each annotation pass.
- Trait-method declarations qualify for `_Simple._` **only** when *every* conforming impl is required to be simple (codegen-driven or contract-enforced). Hand-written impls that could be non-simple disqualify the tag — do not add it speculatively.
- Concrete `#[inline]` and `_Simple._` are mutually exclusive on the same fn.
- No file may exceed AGENTS.md size limits as a side-effect (these are attribute-only edits, so size impact is minimal).
- Documentation gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`) must pass — adding a `_Simple._` line to an existing doc must not break formatting.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ObjectExt::id`, `ObjectExt::name`, and `ObjectExt::is_on_current_thread` each carry `#[inline]`. |
| AC2 | Every other `ObjectExt` default trait method that satisfies the recursive simple test carries `#[inline]`; methods that do not satisfy it remain unmarked. |
| AC3 | Across the entire workspace, every concrete fn that satisfies the recursive simple test (including `Default::default()` wrappers, trivial getters, single-delegation wrappers, struct-literal `const fn` constructors) carries `#[inline]`. |
| AC4 | Every generic fn or trait-method declaration discovered during the sweep that satisfies the recursive simple test (and whose conforming impls are required to be simple, in the trait-method case) carries the `_Simple._` doc tag, formatted per `ai-docs/doc-convention.md`. |
| AC5 | The cascade is quiescent: there exists no fn in the workspace satisfying the recursive simple test that lacks the appropriate marker. (Spot-check by re-running the audit on a sample of just-annotated fns' callers.) |
| AC6 | `cargo build && cargo clippy -- -D warnings && cargo fmt -- --check && cargo test` all pass clean. |
| AC7 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` passes clean. |
| AC8 | `cargo build -p quartzite --no-default-features` (derive-free / `no_std` path) passes clean. |
| AC9 | No fn carries both `#[inline]` and `_Simple._` (mutual exclusion verified by spot grep). |

## Open questions

None at spec time.
