# parent / children accessors on ObjectExt

**Source:** issue #55
**Date:** 2026-05-05
**Tracked in:** #55

## Scope

1. Add a process-global tree-access helper (e.g. `try_with_tree`) registered by `Application` on construction and cleared on drop.
2. Add `ObjectExt::parent() -> Option<ObjectId>` — uses the global accessor; returns `None` when object is root or no tree is registered.
3. Add `ObjectExt::parent_in(tree: &ObjectTree) -> Option<ObjectId>` — explicit tree parameter variant of `parent()`.
4. Add `ObjectExt::children() -> Vec<ObjectId>` — uses the global accessor; returns an empty `Vec` when object has no children or no tree is registered.
5. Add `ObjectExt::children_in(tree: &ObjectTree) -> &[ObjectId]` — explicit tree parameter variant returning a slice (lifetime tied to tree) instead of an owned `Vec`.
6. Document threading constraints (v1: single-threaded; process-global is safe).
7. Tests: parent of root → `None`; children in insertion order; zero-child object → empty slice; called outside `Application` → safe fallback.

## Out of scope

- Subtree-scoped queries and reactive name-change notifications (issue #54).
- Multi-window or multi-tree scenarios (issue #53).
- Per-thread tree registration (issue #51 — thread-local variant is deferred until per-thread event loops land).
- Mutable parent/children manipulation (reparenting).

## Deferred

- Thread-local `OnceCell<&ObjectTree>` variant — blocked on per-thread event loops (#51); no separate issue needed, will extend this design when #51 lands.

## Key decisions

| Question | Decision |
|---|---|
| Tree-access mechanism | Process-global `try_with_tree` (registered by `Application`); both ergonomic (`parent()`) and explicit-param (`parent_in(tree)`) variants exposed |
| Behavior outside `Application` scope | Return `None` / empty (never panic) — no tree registered is not a broken global invariant |
| Return type for children (global variant) | `Vec<ObjectId>` — slice lifetime cannot escape the `try_with_tree` closure |
| Return type for children (explicit-param variant) | `&[ObjectId]` — lifetime tied to `&ObjectTree`, no allocation |
| Naming convention for explicit-param variants | `_in` suffix: `parent_in` / `children_in` — descriptive; unsuffixed names are the common ergonomic default |

## Technical constraints

- `ObjectTree` already stores `parent_map: HashMap<ObjectId, ObjectId>` and `children_map: HashMap<ObjectId, Vec<ObjectId>>` — the explicit-param methods delegate directly to `ObjectTree::parent_of` / `ObjectTree::children_of`.
- `ObjectExt` is implemented for every type implementing `AsObject`; default method bodies are appropriate since they only need `self.object_id()`.
- The process-global must be safe to set/clear from single-threaded `Application` code; a `Cell<Option<…>>` or similar thread-local-free global with `#[cfg(not(feature = "std"))]` consideration is a design concern.
- `#[inline]` required on every simple generated accessor.
- All new public items must satisfy the workspace doc convention (`# Examples`, `# Parameters` when ≥ 1 non-receiver arg, `# Returns` where helpful).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `obj.parent()` returns `None` when `obj` is a root (no parent registered in the tree) |
| AC2 | `obj.parent()` returns `Some(parent_id)` when `obj` has a registered parent |
| AC3 | `obj.parent()` returns `None` when called outside an `Application` scope (no global tree set) |
| AC4 | `obj.children()` returns children in insertion order |
| AC5 | `obj.children()` returns an empty `Vec` for a leaf node (no children) |
| AC6 | `obj.children()` returns an empty `Vec` when called outside an `Application` scope |
| AC7 | `obj.parent_in(tree)` returns the same value as `obj.parent()` when the same tree is active |
| AC8 | `obj.children_in(tree)` returns a `&[ObjectId]` slice with the same contents and order as `obj.children()` |
| AC9 | `Application` registers the global tree accessor on construction and deregisters (sets to none) on drop |
| AC10 | All new public items carry doc comments with at least `# Examples` and `# Parameters` (where applicable) |

## Open questions

_None at spec time._
