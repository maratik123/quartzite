# Redesign multi-block object_impl API: #[object_part] + #[object_impl]

**Source:** user description
**Date:** 2026-05-03
**Tracked in:** #57

## Scope

- Add `#[object_part]` proc-macro attribute: accumulates slot/invokable methods into the
  thread-local store and emits only the cleaned impl block (no MetaObject, no `impl Object`)
- Change `#[object_impl]` to auto-detect mode from accumulator state: empty accumulator = sole
  (existing single-block behaviour); non-empty accumulator = terminal (drain + merge + full output)
- Remove `partial` and `final` attribute flags from `#[object_impl]`
- Remove `#[object_meta]` proc-macro and its module
- Update all doc comments and examples in `lib.rs`
- Update all unit and integration tests to use the new API

## Out of scope

- Changing the `thread_local!` accumulator mechanism or its key format
  (`CARGO_PKG_NAME::type_name`)
- Changing duplicate-detection behaviour or error messages
- Any other proc-macro (`#[derive(Object)]`, `#[derive(Extend)]`, `#[meta_enum]`)

## Deferred

- None

## Key decisions

| Question | Decision |
|---|---|
| How is sole vs terminal distinguished? | Inferred from accumulator state at `#[object_impl]` expansion time — no explicit flag needed |
| What replaces `#[object_impl(partial)]`? | `#[object_part]` — a separate attribute with no flags |
| What replaces `#[object_impl(final)]`? | Plain `#[object_impl]` (terminal mode, auto-detected) |
| What replaces `#[object_meta]`? | Plain `#[object_impl]` on an empty block (terminal mode, no methods) |
| Backward compatibility? | None — project not yet published to crates.io |
| Branch? | Reuse `feat/2026-05-03-macro-codegen-improvements`; redesign is a result of PR #79 review |

## Technical constraints

- `final` is a reserved Rust keyword; current code already uses string-based parsing to handle it.
  After this change, `#[object_impl]` accepts no flags at all, so keyword handling is no longer needed.
- `#[object_part]` must be registered as a `proc_macro_attribute` in `lib.rs`.
- The `object_meta` module can be deleted entirely; `object_part` module is added.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `#[object_part]` on an inherent impl block emits only the cleaned impl block; the output contains no MetaObject static and no `impl Object` |
| AC2 | `#[object_part]` on a trait impl block emits `impl Trait for Type { … }` with slot/invokable attributes stripped |
| AC3 | `#[object_impl]` with an empty accumulator (sole mode) emits the full output: cleaned impl block, MetaObject static, `impl Object` — behaviour identical to the former plain `#[object_impl]` |
| AC4 | `#[object_impl]` with a non-empty accumulator (terminal mode) drains accumulated methods, merges with the current block's methods, and emits the full output |
| AC5 | Duplicate slot/invokable names across two `#[object_part]` blocks produce a compile error |
| AC6 | Duplicate slot/invokable names between a `#[object_part]` block and the `#[object_impl]` terminal block produce a compile error |
| AC7 | `#[object_meta]` is removed; `#[object_impl(partial)]` and `#[object_impl(final)]` are no longer accepted; each produces a compile error when used |
| AC8 | `#[object_part]` accepts no attribute arguments; passing any argument produces a compile error |

## Open questions

- None
