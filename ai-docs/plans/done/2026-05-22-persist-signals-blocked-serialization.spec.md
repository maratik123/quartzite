# persist-signals-blocked-serialization: round-trip `signals_blocked` through snapshot/restore

**Source:** GitHub issue #39
**Date:** 2026-05-22
**Tracked in:** #39

## Scope

Issue #39 is the **final word** on whether `ObjectBase::signals_blocked` is part of the snapshot/restore round-trip. Issue #107 (object/property serialization layer) shipped with the conservative default — `signals_blocked` is documented as transient and resets to `false` on restore (AC6 in #107). #39 was deferred against the layer landing; the layer is now in.

**Decision (Q1, Round 1):** persist `signals_blocked` across snapshot/restore. **Decision (Q2, Round 2):** keep `CURRENT_SCHEMA_VERSION` at `1`; use `#[serde(default)]` on the new field for additive evolution.

In scope:

1. **Add `signals_blocked: bool` to `ObjectSnapshot`.** The struct lives at `quartzite-core/src/snapshot/object.rs`. The derive on it is `Serialize + Deserialize` (already present); the new field rides those derives. Annotate the field `#[serde(default)]` so that a v1-shaped payload missing the key deserializes cleanly and yields `signals_blocked = false` (Q2).
2. **`quartzite-runtime/src/snapshot/object.rs::capture_object`** reads `obj.object_base().signals_blocked()` and stores it on the produced `ObjectSnapshot`. **`restore_object`** calls `obj.object_base_mut().block_signals()` when the snapshot says `true`; otherwise the fresh-constructed object is already unblocked and no extra call is needed.
3. **Tree layer inherits via nesting.** `TreeSnapshot::nodes` carries `ObjectSnapshot` per node; no `quartzite-runtime/src/snapshot/tree.rs` code-path edits are needed for the capture/restore path itself, only its test (see AC4).
4. **Inverted tests.** The two existing tests that pin the v1 reset-to-`false` behaviour — `signals_blocked_resets_on_restore` (unit test, `quartzite-runtime/src/snapshot/tree.rs:501`) and `signals_blocked_resets_after_restore` (integration test, `quartzite-runtime/tests/snapshot.rs:460`) — are rewritten to assert preservation. Both test names invert to `signals_blocked_persists_across_restore` at their respective sites.
5. **Documentation flipped.** Three doc sites mention the reset-to-`false` behaviour today:
   - `quartzite-core/src/snapshot.rs:27` — h2 *Transient state — what is NOT serialized*.
   - `quartzite-runtime/src/snapshot.rs:20` — h2 *What is NOT preserved* table.
   - `quartzite-runtime/src/snapshot/object.rs:62` — `restore_object` rustdoc line citing "empty connection table and `signals_blocked = false`".

   The `signals_blocked` mention moves out of *NOT preserved* / *Transient state* at all three sites; its preservation is documented positively at the same spots (and the `restore_object` rustdoc keeps the connection-table line since that remains transient). `quartzite-runtime/src/snapshot/tree.rs:75` ("tables and `signals_blocked = false`") is the inner connection-tables rustdoc and similarly drops the `signals_blocked` clause while keeping the tables clause.
6. **Schema-version envelope unchanged.** `TreeSnapshot::schema_version` stays at `CURRENT_SCHEMA_VERSION == 1` (`quartzite-core/src/snapshot.rs:68`). The `#[serde(default)]` annotation on the new field is the entire forward-evolution mechanism for v1. No bump.
7. **`INDEX.md` row updated.** Today's row for #39 records the historical blocker (#107) as resolved. Replace with a "implemented in PR #<N>" line and drop the historical-blocker qualifier.

## Out of scope

- Re-establishing signal **connections** after restore (`ConnectionTable` entries). Tracked at #107's *Deferred* row ("Reconnecting signals on restore (v2)"); independent of #39.
- Migration tooling between snapshot schema versions. Not relevant under Q2 = no-bump.
- Behavioural change to `signals_blocked` itself (its mutators, its interaction with `emit_macro`, etc.). #39 is exclusively about its serialization round-trip.
- Python bridge / QML declarative load / wasm32 — all separately deferred at the #107 spec level.

## Deferred

| What | Why | Separate issue needed? |
|---|---|---|
| Round-trip of `ConnectionTable` entries | Tracked at #107's *Deferred* row ("Reconnecting signals on restore (v2)"); independent of #39's policy. | no — already tracked |
| Future `CURRENT_SCHEMA_VERSION` bump policy | This spec adopts `#[serde(default)]` for additive evolution within v1. A bump is reserved for a payload-shape change that cannot be expressed additively (rename, type change, removal). When that need arises it gets its own spec. | yes, only if and when needed |

## Key decisions

| Question | Decision |
|---|---|
| **Q1 — Persist `signals_blocked` across snapshot/restore, or accept "reset to `false`" as the deliberate policy?** | **Persist** (Round 1). `signals_blocked` round-trips through `ObjectSnapshot`. |
| **Q2 — Bump `CURRENT_SCHEMA_VERSION` from `1` to `2`, or keep at `1` with `#[serde(default)]`?** | **Keep at `1`** (Round 2). The new field carries `#[serde(default)]`; v1-shaped payloads without the field deserialize and yield `signals_blocked = false`. This sets the project pattern for additive `ObjectSnapshot` evolution within a version. |
| Scope of the round-trip | Object layer carries the field; tree layer inherits via `ObjectSnapshot` nesting; property (bare `Value`) layer is unaffected (it doesn't carry an `ObjectBase`). |
| Implementation site | `quartzite-core/src/snapshot/object.rs` (struct field) + `quartzite-runtime/src/snapshot/object.rs` (capture + restore). No `tree.rs` code-path edits beyond the test rewrite. |
| Pre-publish API stability | AGENTS.md § *API Stability*: clean breaks fine; `ObjectSnapshot`'s serialized shape may be altered directly (pre-`cargo publish`, no downstream consumers). |
| Doc sites | The four reset-to-`false` mentions enumerated under *Scope* item 5 (three primary doc sites + the inner `tree.rs:75` connection-tables rustdoc) are the complete set to update. |

## Technical constraints

- Workspace edition 2024, MSRV per workspace `Cargo.toml`.
- Workspace lints (`[workspace.lints.*]`): `missing_docs = "deny"` + `rustdoc::broken_intra_doc_links = "deny"` apply. The new `pub signals_blocked: bool` field needs a one-line doc-comment.
- `cargo clippy --workspace --all-targets -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` both stay clean.
- `cargo build -p quartzite --no-default-features --features libm` continues to pass. `signals_blocked` exists unconditionally on `ObjectBase`; this change touches `quartzite-core/src/snapshot/` (gated behind the `snapshot` feature today) and `quartzite-runtime/src/snapshot/`. The no-default-features path is unaffected.
- Errors via `thiserror` (workspace convention). `CURRENT_SCHEMA_VERSION` and `DeserializeError::UnsupportedVersion` are untouched.
- `#[inline]` / `_Simple._` policy applies. The `capture_object` and `restore_object` deltas are 1–3 lines each — likely simple; the design agent makes the final marker call.
- `#[serde(default)]` on `signals_blocked` is the entire mechanism for backward-readability of v1 payloads that pre-date this field. No custom `Deserialize` impl, no migration shim.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ObjectSnapshot` in `quartzite-core/src/snapshot/object.rs` carries a new `pub signals_blocked: bool` field annotated `#[serde(default)]` with a doc-comment. Serde derive coverage is preserved. |
| AC2 | `quartzite-runtime/src/snapshot/object.rs::capture_object` reads `signals_blocked` from the source object via `ObjectBase` and stores it on the produced `ObjectSnapshot`. |
| AC3 | `quartzite-runtime/src/snapshot/object.rs::restore_object` re-applies `signals_blocked` to the fresh object (calls `block_signals()` when the snapshot value is `true`; otherwise leaves the freshly-constructed object's default-`false` state). |
| AC4 | Both former "resets to false" tests are inverted to assert preservation and renamed to `signals_blocked_persists_across_restore` at their respective sites (`quartzite-runtime/src/snapshot/tree.rs` and `quartzite-runtime/tests/snapshot.rs`). |
| AC5 | At least one test case covers `signals_blocked == true` round-tripping; at least one covers `signals_blocked == false`. |
| AC6 | The doc sites enumerated in *Scope* item 5 are updated to remove `signals_blocked` from the *transient / NOT preserved* lists and document its preservation positively. The `restore_object` rustdoc (and the `tree.rs:75` connection-tables rustdoc) retain their connection-table clauses. |
| AC7 | A unit test (sited in `quartzite-core/src/snapshot/object.rs` or alongside the runtime-side restore tests, design's call) deserializes a v1-shaped `ObjectSnapshot` payload that omits the `signals_blocked` key and asserts the resulting field is `false`. This pins the `#[serde(default)]` contract. |
| AC8 | `CURRENT_SCHEMA_VERSION` in `quartzite-core/src/snapshot.rs:68` remains at `1`; no change to the schema-version validation path or to the existing `schema_version_rejected` tests. |
| AC9 | `cargo clippy --workspace --all-targets -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` both pass. |
| AC10 | `cargo build -p quartzite --no-default-features --features libm` continues to pass. |
| AC11 | `ai-docs/plans/INDEX.md` row for #39 is updated to mark the issue implemented and drop the historical-blocker qualifier (#107 reference). |
| AC12 | Tracking issue #39 closed with a closure note referencing the PR. |

## Open questions

_None._ Both Q1 (persist) and Q2 (keep schema at 1 with `#[serde(default)]`) are settled. The spec is fully constrained for design.

```yaml
---
status: ready
round: 3
---
```
