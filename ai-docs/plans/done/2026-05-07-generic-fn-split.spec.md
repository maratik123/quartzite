# Generic-fn split for binary size (four targets)

**Source:** issue #118
**Date:** 2026-05-07
**Tracked in:** #118

## Scope

Apply the "Generic-fn split for binary size" pattern from `AGENTS.md` to four public functions that take `impl Into<String>` and have > 3 lines of body:

1. `ObjectTree::rename` (`quartzite-runtime/src/object_tree.rs:318`) — ~30-line body; pass `this: &mut ObjectTree` to inner.
2. `ObjectFactory::register` (`quartzite-runtime/src/factory.rs:122`) — two generic params: `impl Into<String>` (split out) and `F: Fn(...)` (box the closure in the outer shell so `inner` is fully non-generic).
3. `Timer::named` (`quartzite-runtime/src/timer.rs:238`) — single `impl Into<String>`, standard split.
4. `ObjectBase::named` (`quartzite-core/src/object_base.rs:99`) — single `impl Into<String>`, standard split.

Pattern for each:
- Outer fn: only the `impl Into<T>` conversion + one call to `inner`; carries `/// _Simple._` doc tag.
- Nested `fn inner(...)`: holds the original body; named `inner`, defined inside the outer fn body (not a sibling impl method); no `_Simple._` (body is non-simple by construction).
- For `ObjectFactory::register`: closure is boxed (`Box<dyn Fn(...) + Send + Sync>`) in the outer shell; `inner` takes the boxed closure and is fully non-generic.

## Out of scope

- Other `impl Into<T>` / `impl AsRef<T>` / `impl ToString` sites beyond the four listed targets. If additional > 3-line-body sites are found during implementation, file a follow-up issue.
- Binary-size measurement (`cargo bloat`). Optional: if available locally, note before/after numbers in PR body.

## Deferred

- Additional `impl Into<T>` sites | discovered during implementation | yes, file follow-up

## Key decisions

| Question | Decision |
|---|---|
| How to handle `F: Fn(...)` in `ObjectFactory::register`? | Approach 1: box the closure in the outer shell; `inner` is fully non-generic. One allocation per call at startup — acceptable. |
| Marker form for outer fns? | `/// _Simple._` doc line (inherent generic methods — own `impl Into<T>` param). No `#[inline]`. |
| Where does `_Simple._` go in the doc? | Under the summary line, before the first `#` heading (`# Parameters`, etc.). See `ai-docs/doc-convention.md`. |
| Helper name and placement? | Named `inner`, nested inside the outer fn body. Not `<outer>_inner`, not a sibling method. |

## Technical constraints

- Refactor is behaviour-preserving: all pre-existing tests must remain green.
- `cargo build -p quartzite --no-default-features` must compile clean (no_std / derive-free path).
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` must be clean.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ObjectTree::rename` body extracted into nested `fn inner(this: &mut ObjectTree, ...)` defined inside the outer fn body. |
| AC2 | `ObjectFactory::register` closure boxed in the outer shell; nested `fn inner` is fully non-generic; only the `impl Into<String>` conversion remains in the outer fn. |
| AC3 | `Timer::named` body extracted into nested `fn inner` defined inside the outer fn body. |
| AC4 | `ObjectBase::named` body extracted into nested `fn inner` defined inside the outer fn body. |
| AC5 | All four outer fns carry `/// _Simple._` placed correctly (under summary, before first `#` heading). |
| AC6 | All nested helpers are named `inner` (not `<fn>_inner`) and are nested inside the outer fn body (not sibling impl methods). |
| AC7 | No `#[inline]` on outer fns; no `_Simple._` on inner fns. |
| AC8 | `cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check && cargo test` all clean. |
| AC9 | `cargo build -p quartzite --no-default-features` clean. |
| AC10 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` clean. |

## Open questions

_(none)_
