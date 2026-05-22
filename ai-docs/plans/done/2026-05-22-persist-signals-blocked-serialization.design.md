# Design: persist-signals-blocked-serialization

**Issue:** #39
**Date:** 2026-05-22

## Approach

Persist `ObjectBase::signals_blocked` across the snapshot/restore round-trip by adding a single `bool` field to `ObjectSnapshot`, annotated `#[serde(default)]` so that v1-shaped payloads predating the field continue to deserialize and yield `false`. The capture side reads the flag via `obj.object_base().signals_blocked()`; the restore side calls `obj.object_base_mut().block_signals()` only when the snapshot says `true` (a fresh factory-constructed object is already in the default-`false` state, so no unblock path is needed).

The Tree layer inherits automatically because `ObjectNode::snapshot: ObjectSnapshot` is the carrier — no edits in `quartzite-runtime/src/snapshot/tree.rs` capture/restore code, only its `signals_blocked_resets_on_restore` test (inverted in scope item 4).

Schema version stays at `1`. `#[serde(default)]` is the entire forward-compatibility mechanism for this kind of additive evolution within v1, per spec Q2.

**Rejected alternatives:**

- **Bump `CURRENT_SCHEMA_VERSION` to `2` + bidirectional migration code.** Settled by spec Q2 — heavyweight when additive `#[serde(default)]` covers the v1-payload-missing-field case cleanly. Reserves the bump for true incompatible changes (rename, type change, removal).
- **Custom `Deserialize` impl on `ObjectSnapshot`.** Unnecessary; `#[serde(default)]` is purpose-built for this exact case.
- **Round-trip a richer signal-state struct (e.g., bundle with `ConnectionTable` shape).** Out of scope — connection round-trip is tracked separately at #107 *Deferred*. #39 is exclusively about the blocked flag.
- **Reset path in `restore_object` for `signals_blocked == false`.** Asymmetric but correct: factory output is already unblocked, so a redundant `unblock_signals()` call would just be no-op churn. Tests still cover both `true` and `false` cases (AC5).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `pub signals_blocked: bool` field with `#[serde(default)]` + one-line `///` doc to `ObjectSnapshot`; update **all struct-literal sites** that construct an `ObjectSnapshot` to add `signals_blocked: false` (or `signals_blocked: <bool>` as appropriate). The exhaustive list of sites (11 total) verified by reviewer:<br>1. `quartzite-core/src/snapshot/object.rs:24` (struct-level rustdoc doctest)<br>2. `quartzite-core/src/snapshot/tree.rs:14` (`ObjectNode` rustdoc doctest)<br>3. `quartzite-core/src/snapshot/tree.rs:52` (`TreeSnapshot` rustdoc doctest)<br>4. `quartzite-core/src/snapshot/tree.rs:84` (`validate_version` rustdoc doctest)<br>5. `quartzite-core/src/snapshot.rs:181` (`validate_version_ok` test)<br>6. `quartzite-core/src/snapshot.rs:197` (`validate_version_rejects_future` test)<br>7. `quartzite-runtime/src/snapshot/object.rs:305` (`restore_object_unknown_class_returns_error`)<br>8. `quartzite-runtime/src/snapshot/object.rs:320` (`restore_object_type_mismatch_returns_write_rejected`)<br>9. `quartzite-runtime/src/snapshot/tree.rs:441` (`schema_version_rejected` unit test)<br>10. `quartzite-runtime/tests/snapshot.rs:442` (`schema_version_rejected` integration test)<br>11. `quartzite-runtime/src/snapshot/object.rs:48` (`Ok(ObjectSnapshot { … })` return in `capture_object` — covered by task 2's prose but must be updated in concert with task 1). | `quartzite-core/src/snapshot/object.rs`, `quartzite-core/src/snapshot/tree.rs`, `quartzite-core/src/snapshot.rs`, `quartzite-runtime/src/snapshot/object.rs`, `quartzite-runtime/src/snapshot/tree.rs`, `quartzite-runtime/tests/snapshot.rs` | — |
| 2 | Update `capture_object` to populate `signals_blocked: obj.object_base().signals_blocked()` on the constructed `ObjectSnapshot`. Keep the function shape (still `Result<ObjectSnapshot, SerializeError>`, no new error variants). Re-evaluate `#[inline]` / `_Simple._` marker: the function has a `for` loop already so it's not simple — no marker change. | `quartzite-runtime/src/snapshot/object.rs` | 1 |
| 3 | Update `restore_object` to call `obj.object_base_mut().block_signals()` after the property-write loop when `snap.signals_blocked == true`. Update the rustdoc paragraph that currently states "empty connection table and `signals_blocked = false`" to drop the `signals_blocked` clause (the connection-table clause stays). Re-evaluate `#[inline]` / `_Simple._` — already non-simple (loop + branches), no marker change. | `quartzite-runtime/src/snapshot/object.rs` | 1 |
| 4 | Flip both inverted tests + add positive and v1-default-deserialize coverage in the runtime tests:<br>- Rename + invert `signals_blocked_resets_on_restore` → `signals_blocked_persists_across_restore` in `quartzite-runtime/src/snapshot/tree.rs` (asserts `true` round-trips through `capture_tree` + `restore_tree`).<br>- Rename + invert `signals_blocked_resets_after_restore` → `signals_blocked_persists_across_restore` in `quartzite-runtime/tests/snapshot.rs` (integration; same invariant via the public surface).<br>- Add a `signals_blocked_false_round_trips` test (sibling in either file — design's call: place in `quartzite-runtime/src/snapshot/object.rs` `#[cfg(test)] mod tests` alongside the existing `restore_object_round_trips_stored_props` since the fixture (`Sample` + `install_factory()`) already exists there) covering the explicit `signals_blocked == false` round-trip (AC5).<br>- Add a `v1_payload_without_signals_blocked_deserializes_to_false` test in `quartzite-runtime/src/snapshot/object.rs` `#[cfg(test)] mod tests` (no `serde` cargo feature needed — runtime tests already pull serde transitively via fixtures, but since the test depends on the serde derive, gate the test module-block with `#[cfg(feature = "serde")]` if necessary; see Open question 1). Uses `serde_json::from_str` over a hand-written v1-shaped JSON `{"class_name":"SnapshotSample","properties":{}}` and asserts the resulting `ObjectSnapshot.signals_blocked == false`. Pins the `#[serde(default)]` contract (AC7). | `quartzite-runtime/src/snapshot/tree.rs`, `quartzite-runtime/tests/snapshot.rs`, `quartzite-runtime/src/snapshot/object.rs` | 1, 2, 3 |
| 5 | Update the three primary documentation sites + the inner `tree.rs:75` connection-tables rustdoc to remove `signals_blocked` from the *transient / NOT preserved* lists and document its preservation positively. Per spec scope item 5:<br>- `quartzite-core/src/snapshot.rs:27` — drop the `signals_blocked` row from the *Transient state — what is NOT serialized* table; add a new sentence (or a separate row above the table) noting that `signals_blocked` is preserved across restore.<br>- `quartzite-runtime/src/snapshot.rs:20` — drop the `signals_blocked` row from the *What is NOT preserved* table; add a positive sentence/row above noting preservation.<br>- `quartzite-runtime/src/snapshot/object.rs::restore_object` — already covered by task #3 (just retain the connection-table clause).<br>- `quartzite-runtime/src/snapshot/tree.rs:75` — in `restore_tree`'s rustdoc, change the bullet `"…empty connection tables and signals_blocked = false"` to `"…empty connection tables"` (drop the `signals_blocked` clause; the tables clause stays). | `quartzite-core/src/snapshot.rs`, `quartzite-runtime/src/snapshot.rs`, `quartzite-runtime/src/snapshot/tree.rs` | 1, 2, 3 |
| 6 | Update `ai-docs/plans/INDEX.md` row for #39 (line 146): replace `**#39** signals_blocked serde (persist across serialization) — unblocked by #107 ✅` with `**#39** signals_blocked serde — ✅ implemented in PR #<N>` (drop the historical-blocker qualifier per AC11). Verify the post-edit `wc -c` of INDEX.md remains under the 35k cap (it currently isn't an instruction file, but the propagation-check habit applies — INDEX.md is not on the 35k list, so no cap concern, just a sanity check). | `ai-docs/plans/INDEX.md` | 1–5 |

## Handoff plan

`M = 6` (two groups, 3 + 3):

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Group A:** subtasks 1–3 — type + capture + restore (the core code change; field add, capture write-through, restore re-apply).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — tests + docs + INDEX.md (terminal group; 3 subtasks; within the 1..=3 range).

## Risks

- **Round-trip flake risk — meta-property collision.** The new field rides the `ObjectSnapshot` struct derive; if any consumer (today: only the in-tree serde fixtures) has hard-coded `ObjectSnapshot { class_name, properties }` literals omitting `signals_blocked`, they will fail to compile. Mitigation: task 1 covers all 11 enumerated literal sites. The `#[serde(default)]` annotation only helps wire-format compat, not Rust source-level compat — struct literals must list every field.
- **No-default-features build regression.** Spec calls out `cargo build -p quartzite --no-default-features --features libm` must continue to pass. The snapshot module is `feature = "snapshot"`-gated in `quartzite-core` and the runtime snapshot module is `feature = "serde"`-gated in `quartzite-runtime`. Mitigation: no edits land outside those gated paths.
- **Doctest in `ObjectSnapshot`'s rustdoc.** Rustdoc doctest at `quartzite-core/src/snapshot/object.rs:20-29` builds an `ObjectSnapshot` literal with `class_name` + `properties`. Adding a third required field (without default Trait impl on `ObjectSnapshot`) breaks the doctest unless the literal is updated. Mitigation: task 1 explicitly updates that doctest literal. Alternative: derive `Default` on `ObjectSnapshot` and rewrite the doctest with `..Default::default()` — rejected as out-of-scope mission creep.
- **Backward-compat test claim accuracy.** AC7 requires deserializing a v1-shaped payload **without** the `signals_blocked` key and asserting `false`. The test must use a hand-written serialized form (JSON or bincode) — not a previously-captured `ObjectSnapshot` Rust value (because `ObjectSnapshot` now mandates the field at the Rust level). Mitigation: hardcode a JSON string literal `{"class_name":"X","properties":{}}` and round-trip via `serde_json::from_str::<ObjectSnapshot>`. Verified `serde_json = "1"` is already a dev-dep in `quartzite-runtime/Cargo.toml`. The test module inherits the `#[cfg(feature = "serde")]` gate from the parent `crate::snapshot` module (gated at `quartzite-runtime/src/lib.rs:29`); no per-test `#[cfg]` annotation is needed.
- **API-stability axiom (clean breaks, pre-publish).** Adding a required field on a `pub` struct is a clean break by the project's pre-publish posture. No `#[deprecated]` shim, no compat re-export — call sites update directly.

## Test Design

### Task 4a — `signals_blocked_persists_across_restore` (unit, `quartzite-runtime/src/snapshot/tree.rs`)

- **Location:** `quartzite-runtime/src/snapshot/tree.rs` `#[cfg(test)] mod tests` (line ~501; replaces `signals_blocked_resets_on_restore`).
- **Entry point:** `capture_tree` + `restore_tree` (the Tree-layer round-trip).
- **Scenarios:**
  - Happy path — `block_signals()` on the root pre-capture; assert `restored.with(new_root, |o| o.object_base().signals_blocked()) == Some(true)`.
- **Fixtures:** Existing `TreeSample` + `install_factory()` in the same module.

### Task 4b — `signals_blocked_persists_across_restore` (integration, `quartzite-runtime/tests/snapshot.rs`)

- **Location:** `quartzite-runtime/tests/snapshot.rs` (line ~460; replaces `signals_blocked_resets_after_restore`).
- **Entry point:** `capture_tree` + `restore_tree` exercised through the public surface (mirrors the unit test but in the integration crate, where it can also implicitly cross the serde/bincode boundary if extended later).
- **Scenarios:**
  - Happy path — block on the root, capture, restore, assert preserved.
- **Fixtures:** Existing `SerdeFixture` + `install_factory()` in the same file.

### Task 4c — `signals_blocked_false_round_trips` (unit, `quartzite-runtime/src/snapshot/object.rs`)

- **Location:** `quartzite-runtime/src/snapshot/object.rs` `#[cfg(test)] mod tests`.
- **Entry point:** `capture_object` + `restore_object` (Object-layer round-trip).
- **Scenarios:**
  - Construct a Sample, leave `signals_blocked` at default `false`, capture, restore, assert `restored.object_base().signals_blocked() == false`. Pins AC5's `false`-case requirement.
- **Fixtures:** Existing `Sample` + `install_factory()` in the same module.

### Task 4d — `v1_payload_without_signals_blocked_deserializes_to_false` (unit, `quartzite-runtime/src/snapshot/object.rs`)

- **Location:** `quartzite-runtime/src/snapshot/object.rs` `#[cfg(test)] mod tests`.
- **Entry point:** `serde_json::from_str::<ObjectSnapshot>`.
- **Scenarios:**
  - Deserialize the literal JSON string `r#"{"class_name":"SnapshotSample","properties":{}}"#` (v1-shaped, no `signals_blocked` key); assert `result.signals_blocked == false`. Pins the `#[serde(default)]` contract (AC7).
- **Fixtures:** None beyond `serde_json` (already a dev-dep in `quartzite-runtime/Cargo.toml`).

### Task 4 housekeeping

- **Carry forward unchanged:** `restore_object_round_trips_stored_props`, `capture_includes_stored_excludes_non_stored`, `restore_object_unknown_class_returns_error`, `restore_object_type_mismatch_returns_write_rejected`, `capture_object_returns_property_missing_when_read_returns_none`, `schema_version_rejected` (in both `tree.rs` and integration tests), `validate_version_*` — these don't touch `signals_blocked` and must continue to pass.

### Lint / build gates

- `cargo clippy --workspace --all-targets -- -D warnings` (AC9)
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` (AC9; covers the `# Examples` literal update from task 1 + the rustdoc tweaks from tasks 3 + 5)
- `cargo build -p quartzite --no-default-features --features libm` (AC10)
- `cargo test --workspace` (AC4 + AC5 + AC7 + non-regression on the carry-forward tests)
- `cargo fmt -- --check`

## Open questions

_None._ The parent `quartzite-runtime::snapshot` module is `#[cfg(feature = "serde")]`-gated at `quartzite-runtime/src/lib.rs:29`. The task 4d test in `quartzite-runtime/src/snapshot/object.rs` inherits this gate automatically; no per-test `#[cfg]` annotation is needed.
