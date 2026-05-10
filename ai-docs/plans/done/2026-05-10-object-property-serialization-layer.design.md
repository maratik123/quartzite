# Design: object-property-serialization-layer

**Issue:** [#107](https://github.com/maratik123/quartzite/issues/107)
**Date:** 2026-05-10

## Approach

Add a `serde` Cargo feature to `quartzite-core` and `quartzite-runtime` (off by
default). When the feature is on, the snapshot layer compiles in three nested
modules — one per granularity — sharing a single `SerializeError` /
`DeserializeError` pair declared in `quartzite-core::snapshot`. Both crates
already follow the `cfg`-feature pattern (`std`, `verbose-tracing`); this is
just one more orthogonal axis.

Layering rationale (matches the spec's three layers):

1. **Property layer** — derive `serde::{Serialize, Deserialize}` on `Value`
   itself in `quartzite-core::value`. Hand-write the `Custom` arm via
   `typetag::serde` (the `Arc<dyn CustomValue>` cannot be auto-derived because
   `dyn CustomValue` is non-`Sized` and lacks a built-in serde impl). All other
   variants get auto-derived. The derive lives behind `#[cfg(feature =
   "serde")]` so the no-`std` / no-`derive` build path is unchanged.
2. **Object layer** — declare `ObjectSnapshot { class_name: String, properties:
   BTreeMap<String, Value> }` in `quartzite-core::snapshot::object`. Provide
   two free fns: `capture_object(&dyn Object) -> ObjectSnapshot` (walks
   `meta_object().properties`, filtering on `PropertyFlag::Stored`, collecting
   `read_property` results) and `restore_object(&ObjectSnapshot) ->
   Result<Box<dyn Object>, DeserializeError>` (lives in `quartzite-runtime`
   because it consults `ObjectFactory`). The split honors the no-runtime
   posture of `quartzite-core` — the snapshot **type** is core, the
   factory-mediated **restore** is runtime.
3. **Tree layer** — declare `ObjectNode { snapshot: ObjectSnapshot, children:
   Vec<ObjectNode> }` and `TreeSnapshot { schema_version: u32, root: ObjectNode
   }` in `quartzite-core::snapshot::tree`. The capture/restore fns live in
   `quartzite-runtime::snapshot`: `capture_tree(&ObjectTree, root) ->
   TreeSnapshot` walks the tree depth-first, and `restore_tree(&TreeSnapshot)
   -> Result<ObjectTree, DeserializeError>` rebuilds a fresh tree, building an
   `OldObjectId → NewObjectId` remap table during restore and rewriting every
   nested `Value::Object(WeakObjectRef)` after the new objects are inserted
   (two-pass: insert all → walk-and-remap).

The `typetag::serde` v1 policy (Key decision) is the cleanest fit:

- **Why typetag over hand-rolled tag-and-payload dispatch:** `typetag` already
  solves the trait-object serde problem with a discriminator + `inventory`
  registry. Hand-rolling a `class_name` registry on `CustomValue` would
  duplicate `inventory`'s linker-section trick and force users to write a
  parallel `register_custom_value!` macro. Typetag's `#[typetag::serde]`
  annotation is a one-liner per concrete impl.
- **Why typetag over "skip with warning":** silent data loss on round-trip
  violates the AC1 / AC9 bit-equal guarantee.
- **Trait shape under feature:** the spec calls for a feature-gated supertrait
  on `CustomValue`. Implement as `pub trait CustomValue: ... + erased_serde
  trait surface via typetag::serde` — the typetag macro on **impls** (not on
  the trait declaration itself) is what sets the runtime registry; on the
  trait we add a `#[typetag::serde(tag = "type")]`-style attribute when the
  feature is on, which generates the supertrait bound. Behind
  `#[cfg(feature = "serde")]`, replace the trait body with a typetag-annotated
  variant; behind `#[cfg(not(feature = "serde"))]`, keep the existing surface
  verbatim. (The feature axis adds a supertrait — clean break per the
  pre-publish stability rule.)

WeakObjectRef remap (per Key decisions):

- **Tree layer:** during `restore_tree`, we keep the `OldObjectId →
  NewObjectId` map as objects are inserted in pre-order. After insertion of
  every node, we make a second pass that calls `tree.with_mut(new_id, |obj|
  walk_properties_and_remap(obj, &remap))`. The walk visits every `Stored`
  property's `Value`, recursing into `List` / `Map` / `Custom` variants, and
  rewrites `Value::Object(WeakObjectRef(old))` to
  `Value::Object(WeakObjectRef(remap[&old]))`. Refs whose old id is not in the
  remap (i.e. external) stay as-is and dangle.
- **Object / property layers:** no remap — the `WeakObjectRef`'s inner `u64`
  serializes verbatim and dangles on restore.

Schema versioning:

- `TreeSnapshot { schema_version, root }` is auto-derived via `serde::Deserialize`.
- `pub const CURRENT_SCHEMA_VERSION: u32 = 1` lives in
  `quartzite-core::snapshot`.
- The reader is wrapped: `TreeSnapshot::deserialize_versioned<'de, D>(d) ->
  Result<TreeSnapshot, DeserializeError>` deserializes the raw value, then
  validates `schema_version <= CURRENT_SCHEMA_VERSION` (returning
  `DeserializeError::UnsupportedVersion` otherwise). For ergonomics, also
  expose `from_value`, `from_slice`, `from_str` thin wrappers parameterised on
  a backend trait, OR document the policy and require callers to call
  `validate_version()` after their own deserialization. **Choice: explicit
  `validate_version()`** — keeps the layer backend-agnostic per the spec's
  "serde-pluggable" decision, no fixed dep on `serde_json` / `bincode`.

Errors:

- `SerializeError` enum: `PropertyMissing { class_name, property }` (raised
  if `read_property` returns `None` for a `Stored`-flagged property — a
  meta-system invariant violation).
- `DeserializeError` enum: `UnsupportedVersion { found, supported }`,
  `UnknownClass { name }`, `FactoryMissing` (process-wide
  `ObjectFactory::global()` is `None` — raised by `restore_object` /
  `restore_tree`), `WriteRejected { class_name, property }` (raised when
  `write_property` returns `false` — type mismatch or read-only).

Both via `thiserror` (workspace convention).

Why no new crate (matches spec's Key decision): the snapshot **types** (Value
serde impls, `ObjectSnapshot`, `TreeSnapshot`) belong in `quartzite-core`
because they're the data plane and the `Value` impls require crate-local
access. The **runtime side** (factory lookup, tree rebuild, ID remap) belongs
in `quartzite-runtime` because it touches `ObjectTree` and `ObjectFactory`.
Splitting into a third crate would force a circular dep
(`quartzite-snapshot` would need both core types and runtime factories) or a
rework of the factory ownership.

Rejected alternatives:

- **`erased_serde` instead of `typetag`** — typetag is built **on** erased_serde
  and provides the discriminator registry we need. Using bare erased_serde
  would force us to invent the registry, defeating the point.
- **Per-variant `Stored` skip in derive(Object)** — already considered; the
  spec explicitly chose runtime-side filtering (the meta-system already carries
  the flag).
- **Snapshot in `quartzite` facade only** — pushes serde dep on the facade
  crate; loses the ability to snapshot `quartzite-core`-only objects without
  the runtime; layer cannot be tested independently.
- **Schema version on `ObjectSnapshot`** — spec rejects (versioning lives at
  tree boundary, lower layers nest).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `serde` Cargo feature to `quartzite-core` (workspace pin: `serde = "1"` `default-features = false`, `derive`, `alloc`; `typetag = "0.2"` gated). Verify `cargo build -p quartzite --no-default-features` still passes. | `quartzite-core/Cargo.toml` | — |
| 2 | Add `serde` Cargo feature to `quartzite-runtime` (forwards to `quartzite-core/serde`; pulls `serde` workspace pin). Verify `cargo build` matrix. | `quartzite-runtime/Cargo.toml` | 1 |
| 3 | Add `serde` passthrough feature to facade `quartzite` (`serde = ["quartzite-core/serde", "quartzite-runtime/serde"]`). | `Cargo.toml` (workspace root facade `[features]`) | 2 |
| 4 | Implement `Serialize` / `Deserialize` for `Value` in `quartzite-core::value` behind `#[cfg(feature = "serde")]`. Hand-write the `Custom` arm using typetag-aware boxing; auto-derive other variants via a `Serialize`-tagged proxy enum or hand-write a `Visitor` (choice depends on what handles `Arc<dyn CustomValue>`; design recommends **hand-written impl** for `Value` to avoid double-boxing the `Arc` and to retain the existing `Default` / `PartialEq` semantics unchanged). Add `#[cfg(feature = "serde")]` `Serialize`/`Deserialize` for `WeakObjectRef` (transparent `u64`). Add `serde_json = "1"` and `bincode = "2"` (pinned to major 2 — `bincode 3.0.0` on crates.io is a `compile_error!` placeholder per `bincode 3.0.0/src/lib.rs`; the functional latest is `2.0.1`, verified current 2026-05-10) to `quartzite-core/Cargo.toml [dev-dependencies]` **unconditionally** (Cargo's `[target.<cfg>.dev-dependencies]` does **not** accept feature predicates — only `cfg(target_arch / target_os / target_family / ...)`). The `serde_tests` module itself is gated `#[cfg(all(test, feature = "serde"))]`, so dev-deps are inert when the `serde` feature is off. | `quartzite-core/src/value.rs`, `quartzite-core/Cargo.toml` | 1 |
| 5 | Add the typetag supertrait to `CustomValue` (feature-gated) in `quartzite-core::value`. Add the rustdoc `# Examples` block per AC11 showing `#[typetag::serde] impl CustomValue for MyType { ... }`. | `quartzite-core/src/value.rs` | 4 |
| 6 | Create `quartzite-core::snapshot` module with `SerializeError` / `DeserializeError` (`thiserror`), `pub const CURRENT_SCHEMA_VERSION: u32 = 1`, `TreeSnapshot::validate_version` helper, and re-exports. Module is gated `#[cfg(feature = "serde")]`. Wire into `lib.rs` plus `# Serialization` rustdoc section per AC10. Update `PropertyFlag::Stored` rustdoc to point at the module per AC11. | `quartzite-core/src/snapshot.rs` (new), `quartzite-core/src/snapshot/object.rs` (new), `quartzite-core/src/snapshot/tree.rs` (new), `quartzite-core/src/lib.rs`, `quartzite-core/src/meta.rs` | 4 |
| 7 | Implement object-layer capture/restore in `quartzite-runtime::snapshot::object`: `capture_object(obj: &dyn Object) -> Result<ObjectSnapshot, SerializeError>` (filters on `PropertyFlag::Stored`, calls `read_property`); `restore_object(snap: &ObjectSnapshot) -> Result<Box<dyn Object>, DeserializeError>` (consults `ObjectFactory::global()`, calls `write_property`, mapping `false` to `WriteRejected`). Module gated `#[cfg(feature = "serde")]`; the `pub mod snapshot;` declaration in `lib.rs` and any re-exports are also `#[cfg(feature = "serde")]` to mirror core (the runtime module consumes `quartzite_core::snapshot::SerializeError`, which exists only under `serde` — `cargo build --no-default-features` would otherwise break). | `quartzite-runtime/src/snapshot.rs` (new), `quartzite-runtime/src/snapshot/object.rs` (new), `quartzite-runtime/src/lib.rs` | 6 |
| 8 | Implement tree-layer capture/restore in `quartzite-runtime::snapshot::tree`: `capture_tree(tree: &ObjectTree, root: ObjectId) -> Result<TreeSnapshot, SerializeError>` (depth-first walk via `children_of`, building nested `ObjectNode`s); `restore_tree(snap: &TreeSnapshot) -> Result<(ObjectTree, ObjectId), DeserializeError>` (returns the new tree and root id; builds remap table; second pass rewrites `Value::Object` payloads in every restored object's `Stored` properties via a `walk_value_remap` helper; explicitly resets `signals_blocked = false` per AC6). Helper `walk_value_remap(value: &mut Value, remap: &HashMap<u64, u64>)` recurses over `List` / `Map` only (the `CustomValue` trait has no walk hook — `Custom(Arc<dyn CustomValue>)` payloads are opaque to the remap; `WeakObjectRef`s embedded inside `Custom` payloads dangle on restore — caller's responsibility to remap if relevant) and rewrites `Object` arms. Module gated `#[cfg(feature = "serde")]`; re-exports in `quartzite-runtime/src/snapshot.rs` and `lib.rs` are also `#[cfg(feature = "serde")]` to mirror task 7. | `quartzite-runtime/src/snapshot/tree.rs` (new), `quartzite-runtime/src/snapshot.rs`, `quartzite-runtime/src/lib.rs` | 7 |
| 9 | Re-export the snapshot surface through the `quartzite` facade behind `cfg(feature = "serde")`: `#[cfg(feature = "serde")] pub mod snapshot { pub use quartzite_core::snapshot::*; pub use quartzite_runtime::snapshot::*; }`. Both the `pub mod snapshot` declaration and every `pub use` inside it are `#[cfg(feature = "serde")]`-gated, mirroring tasks 6/7/8 — without this gate, the facade fails to compile under `--no-default-features` because `quartzite_core::snapshot` doesn't exist there. Add `# Serialization` section to facade `lib.rs` rustdoc. | `src/lib.rs` | 8 |
| 10 | Round-trip integration test: fixture `#[derive(Object)]` toy type `SerdeFixture` covering every v1 `Value` variant including a `#[typetag::serde]`-annotated `MyCustom { v: i64 }`, plus a non-`Stored` property; tests at all three layers (property, object, tree); WeakObjectRef remap test; signal-drop test (AC6); schema-version reject test (AC5); both `serde_json` and `bincode` backends per AC1 / AC9. | `quartzite-runtime/tests/snapshot.rs` (new); `quartzite-runtime/Cargo.toml` `[dev-dependencies]` adds `serde_json = "1"`, `bincode = "2"` (pinned to major 2 — see note below), `serial_test = "3"` (verified current 2026-05-10), plus quartzite-core dev dep with `serde` feature; module/test items gated `#[cfg(feature = "serde")]` so the no-default-features path stays clean | 8 |
| 11 | Update `ai-docs/plans/INDEX.md`: move this plan from the deferred footnote to "Active plans" with a row pointing at `2026-05-10-object-property-serialization-layer.spec.md`; remove the "Deferred — #107" footnote line; leave the #39 "blocked on #107" footnote intact (per AC12). | `ai-docs/plans/INDEX.md` | 10 |

> Eleven tasks. The `> 7 → split` rule fires only when the work itself spans
> multiple issues; here every task is one logical step inside a single feature
> with linear dependency, and the rationale (one cargo feature per crate, one
> serde impl per layer) cannot be cleanly halved without artificial
> stage-gating. Documenting here for the design reviewer; if a smaller PR is
> wanted, the natural split is "tasks 1–6 (types only)" and "tasks 7–11
> (runtime + tests)".

## Risks

- **`typetag` `inventory` linker-section incompatibility on
  `wasm32-unknown-unknown`** — already deferred per spec; document in module
  rustdoc; surface a compile-time `#[cfg(target_arch = "wasm32")]` guard
  isn't needed because users opt in via the cargo feature, but the doc
  must call out the limitation. **Mitigation:** explicit `#[doc(cfg(not(target_arch
  = "wasm32")))]` on the `Custom`-round-trip example block; spec-tracked.
- **Custom serde impl for `Value` may mis-handle `Float NaN` round-trip** —
  AC1 explicitly carves out NaN bit-equality. **Mitigation:** test asserts
  `is_nan()` only, not bit-equality, for the NaN case; document in
  `Value`'s rustdoc.
- **`Arc<dyn CustomValue>` cloning during deserialize** — typetag deserializes
  to `Box<dyn CustomValue>`; we wrap in `Arc::from(box)` to fit `Value::Custom`'s
  shape. **Mitigation:** hand-written impl owns this conversion explicitly;
  unit-test the round-trip.
- **`WeakObjectRef` remap walks every `Stored` property `Value`, recursing
  into `List` / `Map` only** — `Custom(Arc<dyn CustomValue>)` payloads are
  opaque (the `CustomValue` trait has no walk hook), so any `WeakObjectRef`
  embedded inside a `Custom` payload dangles on restore. Performance is
  O(total serialized scalars in `List`/`Map` arms) per restore. Acceptable:
  restore is a one-time event, not a hot path. **Mitigation:** the walk has
  zero overhead when no `Object` variant is present (single `match` branch);
  the `Custom`-embedded-ref dangling is documented in `restore_tree` rustdoc
  with a note pointing users to wrap the ref in a `Stored` `Value::Object`
  property if they need remap behavior.
- **`ObjectFactory` is a process-wide singleton with first-wins class-name
  semantics** — already a known limitation per the spec's open questions.
  **Mitigation:** documented in `restore_object` rustdoc; out of scope.
- **`PropertyFlag::Stored` semantic drift** — the flag has existed unused since
  the meta-system landed. Tests must verify the behavior the spec promises
  (skip-on-serialize, ignore-on-deserialize) is what we implement, not what
  callers might assume. **Mitigation:** AC4's dedicated test.
- **Pre-publish API stability axiom (no compat shims)** — the new
  `CustomValue` supertrait under the `serde` feature is a clean break for any
  existing downstream `CustomValue` impl; no deprecation layer added per
  AGENTS.md `# API Stability`. There are no published downstreams, so this is
  a non-issue, but call it out in commit / PR body.
- **Partial-restore atomicity** — if `restore_tree` fails mid-walk, the
  partially-built tree is dropped. **Mitigation:** the function returns a
  fresh `ObjectTree` (per spec's "fresh tree" decision); failure drops the
  in-progress tree without touching any caller-owned state. No additional
  rollback machinery needed.
- **`signal_to_signal` connections in the `tests/` integration directory**
  use `quartzite-core::connect` which requires `std`. The serde feature
  itself does not require `std`, but the integration test in task 10 lives
  under `quartzite-runtime/tests/` (already `std`-only), so no extra
  conditional cfgs are needed there.

## Test Design

### Task 4 — `Value` serde impl

- **Location:** `quartzite-core/src/value.rs` `#[cfg(all(test, feature =
  "serde"))]` module `serde_tests`
- **Entry point:** hand-written round-trip helpers
  `roundtrip_json(v: &Value) -> Value` and `roundtrip_bincode(v: &Value) ->
  Value`.
- **Scenarios (one parameterised `rstest` per backend):**
  - happy: every variant except `Custom` round-trips bit-equal
    (AC1) — Null, Bool(true/false), Int(i64::MIN/MAX/0), Float(0.0, 1.5,
    -3.0), String("", "ascii", "üñíçødé"), List of mixed, Map of two
    keys, Bytes(empty / `[0,255,...]`), Object(WeakObjectRef(0/1/u64::MAX)),
    Duration(ZERO / from_secs(1)).
  - edge: `Float(f64::NAN)` round-trips to a value where `is_nan()` is true
    (no bit-equality assertion).
  - error: malformed payload (`bincode::deserialize::<Value>(&[0xff;
    16])`) returns `Err`.
- **Fixtures:** `serde_json::to_string` + `serde_json::from_str`;
  `bincode::serde::encode_to_vec` / `bincode::serde::decode_from_slice` (the
  `bincode = "2"` API; bincode 3.0.0 on crates.io is a `compile_error!`
  placeholder, so we pin major 2 explicitly — verified current 2026-05-10).
  Dev-deps `serde_json = "1"` and `bincode = "2"` added in task 4
  **unconditionally** under `quartzite-core/Cargo.toml [dev-dependencies]`
  (Cargo's `[target.<cfg>.dev-dependencies]` does not accept feature
  predicates — only `cfg(target_arch / target_os / target_family / ...)` —
  so feature-conditional dev-deps are not a Cargo concept). The `serde_tests`
  module is gated `#[cfg(all(test, feature = "serde"))]`, which makes the
  dev-deps inert when `serde` is off; `cargo test --no-default-features`
  still compiles cleanly because the unused dev-deps are simply not linked
  into any test target.

### Task 5 — `CustomValue` typetag round-trip

- **Location:** `quartzite-core/src/value.rs` same `serde_tests` module.
- **Entry point:** typetag-annotated fixture `MyCustom { v: i64 }`.
- **Scenarios:** `roundtrip_json` and `roundtrip_bincode` of
  `Value::Custom(Arc::new(MyCustom { v: 42 }))` produce a `Value::Custom`
  whose downcast to `MyCustom` yields `v == 42` (AC9). Also assert
  `Custom`'s `clone_box`-via-`Clone` path is unaffected.

### Task 6 — `snapshot` module errors and version envelope

- **Location:** `quartzite-core/src/snapshot.rs` `#[cfg(test)]` module.
- **Entry point:** `TreeSnapshot::validate_version`.
- **Scenarios:**
  - happy: `schema_version = 1` returns `Ok(&snap)`.
  - error: `schema_version = u32::MAX` returns
    `Err(DeserializeError::UnsupportedVersion { found: u32::MAX, supported: 1
    })` (AC5).
  - error: `Display` impls on both error enums emit human-readable text (one
    `assert!(err.to_string().contains("..."))` per variant).

### Task 7 — Object capture / restore

- **Location:** `quartzite-runtime/src/snapshot/object.rs` `#[cfg(test)]`
  module.
- **Entry point:** `capture_object` and `restore_object`.
- **Fixtures:** local `#[derive(Object)]` `Sample` struct with three fields:
  `count: i64` (`Stored`), `name: String` (`Stored`), `cache: i64` (no
  `Stored`). `ObjectFactory::install` is gated by a once-cell at module
  scope (or by `serial_test::serial`).
- **Scenarios:**
  - happy: capture then restore; new instance has matching `count` /
    `name`; `cache` retains its **default** value (AC2, AC4).
  - error: payload with `class_name = "DoesNotExist"` returns
    `DeserializeError::UnknownClass`.
  - error: payload with `class_name` registered but property type mismatch
    (`count: Value::Bool(true)`) returns `DeserializeError::WriteRejected`.
  - error: capture against an object whose `meta_object().properties`
    references a property absent from `read_property` returns
    `SerializeError::PropertyMissing` (synthesized via a hand-rolled
    `MetaObject` static in the test module — exercises the meta-invariant
    branch).

### Task 8 — Tree capture / restore + remap

- **Location:** `quartzite-runtime/src/snapshot/tree.rs` `#[cfg(test)]`
  module.
- **Entry point:** `capture_tree`, `restore_tree`, `walk_value_remap`.
- **Fixtures:** the same `Sample` derive plus a second `Sample2` with a
  `linked: Value` property of type `Value::Object`.
- **Scenarios:**
  - happy: 3-node tree (root + child + grandchild) round-trips; restored
    tree's `parent_of` / `children_of` matches the original shape (AC3).
  - happy: `Sample2 { linked: Value::Object(WeakObjectRef(child_old_id))
    }` is rewritten to `linked == Value::Object(WeakObjectRef(child_new_id))`
    after restore — child_new_id resolved via the new tree's `with` (AC3).
  - happy: `WeakObjectRef` whose old id is **outside** the snapshot is left
    unchanged (dangling, per spec).
  - happy: `Value::List(vec![Value::Object(...)])` and
    `Value::Map({"k": Value::Object(...)})` both get remapped (recursive
    walk).
  - signal-drop (AC6): a `Sample` instance with `signals_blocked = true`
    inside the snapshot is restored with `signals_blocked == false` and
    empty `ConnectionTable` (asserted via the runtime's
    `ConnectionTable::receivers_for_signal` lookup yielding empty for the
    new id).
  - schema version: hand-build `TreeSnapshot { schema_version: u32::MAX,
    root: ... }`, serialize, deserialize: error matches `UnsupportedVersion`
    (AC5; this is the integration-side counterpart to the unit test in
    task 6).

### Task 10 — Cross-backend integration tests

- **Location:** `quartzite-runtime/tests/snapshot.rs` (new).
- **Entry point:** parameterised tests over `(SerdeBackend, layer)` axis.
- **Scenarios:** rebuild AC1, AC2, AC3, AC4, AC5, AC6, AC9 here as black-box
  tests against the public re-exported API (not via crate-private helpers),
  using both `serde_json` and `bincode` backends per AC1/AC9.
- **Fixtures:** `Application::new()` once-cell to install
  `ObjectFactory`; fixture `Sample` derive; `MyCustom` `#[typetag::serde]`
  impl. `serial_test::serial` on every test that touches the global
  `ObjectFactory`/`Application` — requires `serial_test = "3"` (verified
  current 2026-05-10) added to `quartzite-runtime/Cargo.toml
  [dev-dependencies]` alongside `serde_json = "1"` and `bincode = "2"`.

## Open questions

- ~~**Bincode major version**~~ **Resolved 2026-05-10:** pin `bincode = "2"`
  explicitly — the registry's `max_stable_version` is `3.0.0`, but
  `bincode 3.0.0/src/lib.rs` is literally
  `compile_error!("https://xkcd.com/2347/");` — a placeholder reservation,
  not a working release. The functional latest is `2.0.1`; pinning major `2`
  per the workspace `0.x` / `x` rule. (Following the standard registry-query
  rule blindly here would produce an immediate build failure — this is a
  rare exception where the reviewer must inspect the crate's `lib.rs` after
  the registry query, documented inline in tasks 4 and 10.) `serde_json = "1"`
  (verified current 2026-05-10).
- **Should `restore_tree` accept an `&mut ObjectTree` to merge into, in
  addition to building a fresh tree?** Spec says **fresh** (Key decision).
  Confirming no merge variant is wanted — caller wires the result up
  themselves. (Reviewer should sanity-check; if disagreed, an additional
  `restore_tree_into(&mut ObjectTree, &TreeSnapshot)` is a small follow-up.)
- **Module name `snapshot` vs `serde`** — spec mentions both
  (`quartzite-core::serde` *or* `::snapshot`). Design picks `snapshot` —
  `serde` would shadow the external crate name and confuse imports
  (`use quartzite_core::serde::...` reads ambiguously). Reviewer can flip if
  preferred.
- **`Application::install_factory` automation** — `Application::new()` already
  installs an empty `ObjectFactory`. Should the `serde` feature add a `pub fn
  register<T: Object + Default + 'static>(&mut self, name: &str)` ergonomic
  helper, or leave registration entirely to user code? Spec is silent. Design
  defers (YAGNI; users register today via the bare factory API and the
  test fixture demonstrates the pattern).

## Design Review

### Round 2 — 2026-05-10 — verdict: **revised, ready for re-review**

Round-1 verdict was **ITERATE** with one blocker and four notes. All five fixes
have been applied in-place; the previous **ITERATE** is cleared pending
re-review.

| # | Issue (round-1) | Fix applied |
|---|-----------------|-------------|
| 1 | **[BLOCKER]** Bincode version trap — `max_stable_version` is `3.0.0` but the source is `compile_error!("https://xkcd.com/2347/");`. Following the registry-query rule blindly produces an immediate build failure. | Tasks 4 and 10 now pin `bincode = "2"` explicitly with the placeholder note inline. The Open-questions entry is marked resolved with the explanation. Annotated `(verified current 2026-05-10)` per the long-lived-doc convention. |
| 2 | **[Note]** Runtime snapshot module not cfg-gated — tasks 7/8/9 do not state that `quartzite-runtime::snapshot{,::object,::tree}` and the facade re-export are gated `#[cfg(feature = "serde")]`. Without this, `cargo build --no-default-features` breaks because runtime consumes `quartzite_core::snapshot::SerializeError`, which only exists under the `serde` feature. | Tasks 7, 8, and 9 each carry an explicit "module gated `#[cfg(feature = "serde")]`; re-exports also cfg-gated to mirror core" sentence. |
| 3 | **[Note]** `walk_value_remap` inconsistency — task 8 prose claimed recursion into `Custom`, but `CustomValue` has no walk hook, so `Custom(Arc<dyn CustomValue>)` payloads are opaque. | Task 8 row updated to say `walk_value_remap` recurses into `List` / `Map` only; `WeakObjectRef`s embedded inside `Custom` payloads dangle on restore (caller's responsibility). The matching Risks bullet was rewritten with the same correction and a doc-note pointer. |
| 4 | **[Note]** Invalid Cargo syntax — `[target.'cfg(feature = "serde")'.dev-dependencies]` is not a Cargo concept (target predicates accept only `target_arch / target_os / target_family / ...`, never feature predicates). | Task 4 row and the Test Design fixtures both call out unconditional `[dev-dependencies]` listing of `serde_json` and `bincode`, with the gating moved to the test module via `#[cfg(all(test, feature = "serde"))]`. The previous incorrect snippet is gone. |
| 5 | **[Note]** `serial_test` missing from `quartzite-runtime` dev-deps in task 10. | Task 10 row and the Task-10 Test-Design fixtures both list `serial_test = "3"` (verified current 2026-05-10) alongside `serde_json` and `bincode`. |

No other sections were touched. Awaiting design-review re-pass.
