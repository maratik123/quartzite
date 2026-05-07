# Design: Generic-fn split for binary size (four targets)

**Issue:** #118
**Date:** 2026-05-07
**Spec:** `ai-docs/plans/2026-05-07-generic-fn-split.spec.md`

## Approach

Apply the nested-`inner` pattern from AGENTS.md to each of the four listed public functions. Each outer fn is reduced to: (1) convert the `impl Into<String>` parameter via `.into()`, and (2) delegate to a nested `fn inner(...)` that carries the original body. The outer fn acquires `/// _Simple._` (placed after the summary prose, before the first `#` heading). The inner fn is named exactly `inner`, nested inside the outer fn body, and carries no `_Simple._` or `#[inline]`.

**Rationale:** Only the trivial conversion shell is monomorphized per `T`; the body ships once in the binary. For `ObjectFactory::register` the closure generic `F` is also eliminated by boxing it in the outer shell before passing to inner.

**Rejected alternatives:**
- Sibling private `fn rename_inner(...)` method on `impl ObjectTree`: AGENTS.md explicitly prohibits placing the helper as a sibling impl method — nesting keeps it out of the type's namespace.
- Keeping the existing body as-is with `#[inline]` added: `#[inline]` on a generic fn is redundant (monomorphized bodies are already available cross-crate); the pattern is the correct fix for large bodies.

**Scope note for `ObjectFactory::register` and `ObjectBase::named`:** The spec explicitly lists both functions as targets even though their current bodies are short (1 line and ~2 lines respectively, below the AGENTS.md "~3 lines" threshold). The design follows the spec's explicit decision. The risk section notes that these two are borderline and the binary-size benefit is negligible compared to the pattern conformance benefit.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Apply nested-`inner` split to `ObjectTree::rename`; add `/// _Simple._` to outer fn doc | `quartzite-runtime/src/object_tree.rs` | — |
| 2 | Apply nested-`inner` split to `ObjectFactory::register`; box closure in outer shell; add `/// _Simple._` | `quartzite-runtime/src/factory.rs` | — |
| 3 | Apply nested-`inner` split to `Timer::named`; add `/// _Simple._` | `quartzite-runtime/src/timer.rs` | — |
| 4 | Apply nested-`inner` split to `ObjectBase::named`; add `/// _Simple._` | `quartzite-core/src/object_base.rs` | — |
| 5 | Run full verification suite | — | 1, 2, 3, 4 |

All four refactors are independent; tasks 1–4 may be executed in any order.

## Per-function inner signatures

### 1. `ObjectTree::rename`

```rust
pub fn rename(&mut self, id: ObjectId, new_name: impl Into<String>) {
    fn inner(this: &mut ObjectTree, id: ObjectId, new_name: String) {
        // original ~25-line body (span, name lookup, update, signal emit)
    }
    inner(self, id, new_name.into())
}
```

The `trace_span!` initialisation uses `new_name` (already a `String` inside `inner`). No behavioral change.

### 2. `ObjectFactory::register`

The `Constructor` type alias (`type Constructor = Box<dyn Fn() -> Box<dyn Object> + Send + Sync>`) already exists in `factory.rs` and should be reused in `inner`'s signature.

```rust
pub fn register<F>(&mut self, class_name: impl Into<String>, ctor: F)
where
    F: Fn() -> Box<dyn Object> + Send + Sync + 'static,
{
    fn inner(this: &mut ObjectFactory, class_name: String, ctor: Constructor) {
        this.registry.insert(class_name, ctor);
    }
    inner(self, class_name.into(), Box::new(ctor))
}
```

### 3. `Timer::named`

```rust
pub fn named(name: impl Into<String>, interval: Duration) -> Self {
    fn inner(name: String, interval: Duration) -> Timer {
        // original struct-literal body
    }
    inner(name.into(), interval)
}
```

### 4. `ObjectBase::named`

```rust
pub fn named(name: impl Into<String>) -> Self {
    fn inner(name: String) -> ObjectBase {
        // `Self` is not in scope inside a nested fn — use the concrete type name.
        ObjectBase {
            name: Some(name),
            ..ObjectBase::new()
        }
    }
    inner(name.into())
}
```

## Risks

- **`ObjectTree::rename` span ordering:** The span is initialized with the already-converted `new_name`. After the split, the span moves into `inner` where `new_name: String` is already available. No behavioral change.
- **`Timer::named` parameter threading:** `inner` cannot close over `interval`; it receives it as a second `Duration` parameter. Straightforward.
- **`ObjectFactory::register` closure boxing:** One allocation per `register` call (startup-only, acceptable per spec decision). The outer fn after the split contains two expressions: `Box::new(ctor)` (calling simple `Box::new`) and `inner(...)` (non-simple). Per the AGENTS.md budget rule — at most one call to a non-simple function — the outer fn qualifies as simple and the `/// _Simple._` tag is correct.
- **`ObjectBase::named` and `ObjectFactory::register` body size:** Both are below the "~3 lines" threshold. Spec-mandated — no mitigation needed.
- **Doc gate:** The `/// _Simple._` tag is a regular doc line; does not affect `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc`.
- **`no_std` path:** Refactor is purely structural — no new imports, no `std`-only primitives. `quartzite-core` `no_std` path unaffected.

## Test Design

Pure structural refactoring — observable behavior is unchanged. Existing tests are the correctness oracle.

- **`ObjectTree::rename`:** Existing tests cover rename-updates-name, name-index update, no-op when same name, absent id, `name_changed` signal emission. No new cases.
- **`ObjectFactory::register`:** Existing tests cover registered-class-creates-instance, unregistered-class-returns-None. No new cases.
- **`Timer::named`:** Existing doctest asserts `name() == Some("heartbeat")`. No new cases.
- **`ObjectBase::named`:** Existing doctest asserts `name() == Some("sensor-1")`. No new cases.

**Verification command (task 5):**
```bash
cargo build && \
cargo clippy --all-targets -- -D warnings && \
cargo fmt -- --check && \
cargo test && \
cargo build -p quartzite --no-default-features && \
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace
```

## Open questions

- `ObjectFactory::register` and `ObjectBase::named` have bodies shorter than the "~3 lines" threshold. Spec-mandated for this PR; if deemed noise, a follow-up can revert these two.
