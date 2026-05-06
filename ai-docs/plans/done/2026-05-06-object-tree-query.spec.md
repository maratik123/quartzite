# ObjectTree query enhancements

**Source:** issue #54
**Date:** 2026-05-06
**Tracked in:** #54

## Scope

- `ObjectTree::find_by_name_in(root: ObjectId, name: &str) -> Vec<ObjectId>` — subtree search restricted to descendants of `root` (inclusive), results sorted by ascending depth (shallowest first)
- Built-in `name_changed` quartzite signal code-generated on every `#[object]` type, carrying `(old_name: Option<&str>, new_name: Option<&str>)`
- `ObjectTree::rename` and `ObjectTree::clear_name` emit `name_changed` after updating the by-name index
- Tests: scoped lookup ignores siblings and ancestors outside the subtree; `name_changed` fires on rename and on clear_name

## Out of scope

- Tree-level observer/watcher object (use per-object signal and connect on insert)
- Wiring through `quartzite-events` (UI events crate) for name-change notifications
- `destroy` does not emit `name_changed` — destruction is a separate concern

## Deferred

- Optional per-subtree `HashMap` index for O(1) scoped `find_by_name_in` | avoids DFS on large trees; requires subtree membership tracking | separate issue when needed

## Key decisions

| Question | Decision |
|---|---|
| Subtree depth for `find_by_name_in` | All descendants (inclusive of root), not direct children only |
| Result ordering for `find_by_name_in` | Sorted by ascending depth (shallowest match first); ties within same depth are insertion-order stable |
| Notification mechanism | Quartzite signal per object (`name_changed`), not raw callbacks or quartzite-events |
| Signal placement | Code-generated on every `#[object]` type (built-in, like Qt's `objectNameChanged`) |
| Signal payload | `(old_name: Option<&str>, new_name: Option<&str>)` — full before/after |
| `destroy` trigger | Does not emit `name_changed` |
| O(1) subtree index | Deferred to a separate issue |

## Technical constraints

- `quartzite-runtime` must not gain a dependency on `quartzite-events`
- `find_by_name_in` returns `Vec<ObjectId>` (cannot return `&[ObjectId]` cheaply from a DFS traversal)
- Signal emission in `rename`/`clear_name` must happen after the by-name index is updated, so observers see consistent state
- The `name_changed` signal codegen must be added to `quartzite-macros` (`#[object]` derive) and the emit call wired through `ObjectTree` — the tree holds the `&mut dyn Object` needed to call `emit_signal`
- `find_by_name_in` returns only objects within the subtree rooted at `root`; `root` itself is included if its name matches (at depth 0)
- Results are sorted by ascending depth; ties at the same depth preserve discovery order

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ObjectTree::find_by_name_in(root, name)` returns only `ObjectId`s that are descendants-or-self of `root` with the given name |
| AC2 | `find_by_name_in` with a name that exists outside the subtree returns an empty result |
| AC3 | `find_by_name_in` with an unknown `root` returns an empty result |
| AC3b | When multiple matches exist at different depths, the result is sorted shallowest first (lower depth index appears earlier in the `Vec`) |
| AC4 | Every type derived with `#[object]` exposes a `name_changed` signal (verified by the existing `#[object]` test types) |
| AC5 | `ObjectTree::rename(id, new_name)` emits `name_changed` on the object with `(Some(old), Some(new))` after updating the index |
| AC6 | `ObjectTree::clear_name(id)` emits `name_changed` on the object with `(Some(old), None)` after updating the index |
| AC7 | A no-op rename (same name) does not emit `name_changed` |
| AC8 | Renaming an anonymous object (name was `None`) emits `name_changed` with `(None, Some(new_name))` |
| AC9 | `ObjectTree::destroy` does not emit `name_changed` |

## Open questions

_(none)_
