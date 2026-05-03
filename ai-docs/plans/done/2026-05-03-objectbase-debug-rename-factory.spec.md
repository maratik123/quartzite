# ObjectBase Debug, rename no-op, ObjectFactory singleton

**Source:** issue #61
**Date:** 2026-05-03
**Tracked in:** #61

## Scope

1. Add `#[derive(Debug)]` to `ObjectBase`
2. Make `ObjectTree::rename` a no-op (return `Ok(())`, no event) when the new name equals the current name
3. Add tests covering the no-op `rename` behaviour
4. Pin `ObjectFactory` as a process-global singleton (same pattern as `ConnectionTable`); document the decision

## Out of scope

- `dynamic_properties` — tracked by a separate issue
- Any other `ObjectTree` changes beyond `rename` semantics

## Deferred

- Manual `Debug` impl with field filtering | auto-derive is sufficient now; can be revisited if fields are added that should be hidden | no separate issue needed

## Key decisions

| Question | Decision |
|---|---|
| `ObjectBase: Debug` | `#[derive(Debug)]` — auto-derive; may be replaced with a manual impl later if needed |
| `ObjectTree::rename` when name unchanged | No-op: return `Ok(())` immediately, fire no rename event |
| `ObjectFactory` scope | Process-global singleton, same pattern as `ConnectionTable` |

## Technical constraints

- `ObjectBase` fields must all implement `Debug` for the derive to compile; verify before adding the attribute
- The no-op path in `rename` must short-circuit before any event dispatch or tree mutation
- `ObjectFactory` global access must be safe from any thread (consistent with `ConnectionTable`)

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ObjectBase` implements `Debug` (via derive or equivalent); `format!("{:?}", base)` compiles and does not panic |
| AC2 | `ObjectTree::rename(id, same_name)` returns `Ok(())` without emitting a rename event |
| AC3 | `ObjectTree::rename(id, same_name)` leaves the tree state unchanged (name lookup still returns the same id) |
| AC4 | `ObjectFactory` is accessible as a process-global singleton; no `Application` reference required to obtain it |
| AC5 | All existing tests pass unchanged |

## Open questions

_(none)_
