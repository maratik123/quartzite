# object-property-serialization-layer: snapshot/restore for Object/property state

**Source:** GitHub issue #107
**Date:** 2026-05-10
**Tracked in:** #107

## Scope

The v1 layer is **serde-pluggable**, **layered at three granularities** (property → object → tree), **drops signal connections on restore**, and round-trips `Value::Custom` payloads via **`typetag`-based registration**.

1. **Cargo feature** `serde` added to `quartzite-core` (and downstream crates that re-export from it — `quartzite-runtime`, `quartzite`). When enabled, derives or hand-writes `serde::Serialize` / `serde::Deserialize` for the snapshot-relevant types and pulls in `typetag` for `Value::Custom` round-trip. When disabled, the no-`std` / no-`derive` paths continue to compile unchanged.
2. **Three snapshot layers**, each with its own round-trip API:
   - **Property layer** — bare `Value: Serialize + Deserialize` (the smallest unit; lets a caller persist a single property's value via any serde backend).
   - **Object layer** — a serializable `ObjectSnapshot { class_name: String, properties: BTreeMap<String, Value> }` payload covering all `PropertyFlag::Stored`-flagged properties of a single `Object`. Restoring constructs a fresh `Object` of the named class via the runtime factory and writes properties back.
   - **Tree layer** — a serializable `TreeSnapshot { schema_version: u32, root: ObjectNode }` where each `ObjectNode` carries an `ObjectSnapshot` plus a `Vec<ObjectNode>` of children. Restoring rebuilds the entire `ObjectTree` from scratch, allocating fresh `ObjectId`s and constructing a remap table for any in-snapshot `WeakObjectRef`s so they retarget the new IDs.
3. **`Stored` flag is the gating mechanism** — properties without `PropertyFlag::Stored` are skipped on serialize and ignored on deserialize. (The flag already exists in `quartzite-core::meta`; this layer is its first consumer.)
4. **Schema-version envelope** — `TreeSnapshot` carries `schema_version: u32`; deserializing an unknown major returns `Err(DeserializeError::UnsupportedVersion { found, supported })`. The `ObjectSnapshot` and bare-`Value` layers do not carry their own version (they nest inside `TreeSnapshot` for versioning, or the caller wraps them).
5. **Signal connections are dropped on restore** — restored objects start with empty `ConnectionTable`s; `signals_blocked` resets to `false`. Re-establishing connections after restore is a v2 follow-up (separate issue once the reflection lookup is robust).
6. **`Value::Custom` round-trip via `typetag`** — when the `serde` feature is on, the `CustomValue` trait additionally requires `#[typetag::serde]` annotation on every concrete impl. Each opted-in user type is then transparently round-tripped (tag string + payload) by serde without manual dispatch on the call site. Types that do not opt in (i.e. don't carry the annotation) fail to compile a `Value::Custom(Arc::new(my_type))` insertion only after the user adds them to a snapshot — the trait is feature-gated so non-`serde` builds are unaffected. See *Key decisions* for why typetag was picked over the "skip with warning" alternative.
7. **Round-trip integration tests** at all three layers, parameterised over at least one toy `#[derive(Object)]` type covering every `Value` variant the v1 format supports — including a small `#[typetag::serde]`-annotated `CustomValue` impl that exercises the `Custom` round-trip.
8. **Documentation** — `Stored` flag's existing rustdoc updated to point at this layer; a top-level `## Serialization` rustdoc section in `quartzite-core::lib.rs` (gated under `cfg(feature = "serde")`) describing the round-trip contract, the `Stored` gating, the transient-state list, and the `#[typetag::serde]` requirement on `CustomValue` impls. `CustomValue`'s rustdoc gains a feature-gated `# Examples` block showing the `#[typetag::serde]` opt-in.

## Out of scope

- **`signals_blocked` persistence** — tracked in #39, which explicitly blocks on this issue. v1 documents `signals_blocked` as transient (resets to `false` on load); #39 picks up to revisit once this layer ships.
- **`ConnectionTable` entries** — never serialized (they hold runtime `Fn`-trait closures with no portable representation). Documented as transient.
- **Reconnecting signals after restore** — explicitly v2 per the round-1 answer; tracked separately when this lands.
- **Dynamic / non-schema properties** — tracked separately in #35.
- **Computed properties / bindings** — tracked separately in #56.
- **Python interop snapshot bridge** — tracked separately in #58.
- **QML-style declarative load** — out of scope; this layer is serde-driven binary/textual round-trip only.
- **Migration / upgrade tooling** — schema-version field exists; conversion between major versions is a future task once a v2 ships.
- **Atomic write helpers** — `to_writer` / `from_reader` operate on `serde::Serializer` / `serde::Deserializer` (or `io::Write` / `io::Read` for the `bincode`-style backends); partial-write atomicity is the caller's responsibility.

## Deferred

| What | Why | Separate issue needed? |
|---|---|---|
| Reconnecting signals on restore (v2) | Round-1 answer: drop now, re-establish in v2 once reflection lookup is robust | yes — open after this lands |
| `signals_blocked` persistence policy | Tracked in #39 — already exists | no, #39 already exists |
| Backwards-translation between schema versions (v1 → v2 reader) | Only meaningful once a v2 ships | yes, when v2 lands |
| Object-graph snapshots that span multiple `ObjectTree`s | Multi-window / multi-tree story still being designed | yes, after multi-window (#53) lands |
| `wasm32-unknown-unknown` support for the `serde` feature | `typetag` depends on `inventory`, which uses linker-section tricks unavailable on `wasm32-unknown-unknown`. Quartzite's runtime is currently desktop-targeted (wgpu/winit), so this is not a v1 blocker. | yes — open if/when wasm becomes a target |

## Key decisions

| Question | Decision |
|---|---|
| **Format approach** (round-1 Q1) | **serde-pluggable** — implement `serde::Serialize` / `Deserialize` on `Value` / `ObjectSnapshot` / `TreeSnapshot`. Backend (JSON, bincode, postcard, etc.) is the caller's choice. Adds `serde` (1) dep on `quartzite-core` / `quartzite-runtime` behind a `serde` cargo feature so the no-`derive` path keeps compiling. |
| **Snapshot granularity** (round-1 Q2) | **All three layered** — property primitive (`Value`) → object (`ObjectSnapshot`) → tree (`TreeSnapshot`). Each level has its own round-trip integration test. |
| **Signal-connection round-trip** (round-1 Q3) | **Drop on restore.** Restored objects have empty `ConnectionTable`s; reconnection deferred to v2. |
| **`Value::Custom` v1 policy** (round-2 Q1) | **`typetag` v1.** Add `typetag = "0.2"` to `quartzite-core` (gated on the `serde` feature). When the `serde` feature is on, the `CustomValue` trait additionally requires a `#[typetag::serde]` impl annotation on every concrete user type that wants to round-trip; serde then transparently encodes a `tag: String` discriminator + payload at every `Value::Custom` site. |
| **`CustomValue` trait shape under `serde` feature** | The `CustomValue` trait gains a `#[typetag::serde(name = "...")]`-style supertrait requirement *only* when `cfg(feature = "serde")` is active — non-`serde` builds keep the existing trait surface verbatim. User opts in by adding `#[typetag::serde] impl CustomValue for MyType { ... }` to their existing impl; serde tag defaults to the type's bare name unless overridden. |
| **`CustomValue` `tag` namespace vs. `ObjectFactory::class_name`** | Independent namespaces. `typetag` maintains its own discriminator registry via the `inventory` crate; it does **not** consult `ObjectFactory`. A user could choose to use the same string for both, but the v1 layer enforces no relationship. |
| Property gating | Use existing `PropertyFlag::Stored` bit — no new flag introduced. |
| Versioning envelope | `TreeSnapshot { schema_version: u32, root }` — `schema_version` starts at `1`; reader rejects unknown majors with `DeserializeError::UnsupportedVersion { found, supported }`. Lower layers (`Value`, `ObjectSnapshot`) carry no version — they're nested inside `TreeSnapshot` or the caller wraps them. |
| `ConnectionTable` persistence | Never serialized — runtime-only `Fn`-trait closures have no portable representation. |
| `signals_blocked` persistence (this issue) | Default: not persisted (resets to `false` on load); final word in #39. |
| `WeakObjectRef` round-trip semantics | **Tree layer:** the deserializer builds an `OldObjectId → NewObjectId` remap table while restoring, then rewrites every nested `WeakObjectRef` to point at the new ID. **Object layer:** `WeakObjectRef`s in serialized properties are persisted as the inner `u64` and dangle on restore (caller's responsibility to remap if relevant). **Property layer:** same as object layer. |
| Tree-restore semantics | Restore builds a **fresh** `ObjectTree` (new root, new IDs) — never merges into or mutates an existing tree. Caller wires the result up if they want it to replace a previous tree. |
| Object-restore semantics | Restore constructs a **fresh** `Object` via the runtime factory keyed on `class_name`; an unknown class returns `DeserializeError::UnknownClass { name }`. |
| Property-restore semantics | A bare `Value` payload is just `Value::deserialize` — caller decides which property it overwrites. |
| Cargo-feature naming | `serde` (workspace convention; `serde-pluggable` would be unusual). The feature is opt-in (off by default) on `quartzite-core` and `quartzite-runtime`. |
| Crate placement | **No new crate.** The layer lives behind `cfg(feature = "serde")` in `quartzite-core` (snapshot types + `Serialize`/`Deserialize` for `Value`) and `quartzite-runtime` (object/tree restore via factory + `ObjectId` remap). Re-exported through the `quartzite` facade. |
| Error type | New `SerializeError` / `DeserializeError` enums in `quartzite-core::serde` (or `::snapshot`) via `thiserror` (workspace convention — see `AGENTS.md` § *Error types*). |

## Technical constraints

- Workspace edition 2024, MSRV 1.95 (per workspace `Cargo.toml`).
- `#![deny(missing_docs)]` and the workspace doc-convention apply to every new public item — `# Examples`, `# Errors`, `# Panics` sections per `ai-docs/doc-convention.md`.
- New deps respect the pinning rule in `AGENTS.md` § *Dependency Versions*: `serde = "1"` (verified `1.0.228` current 2026-05-10) with `default-features = false` and `derive` opt-in via the cargo-feature, plus `alloc` for the no_std path. `typetag = "0.2"` (verified `0.2.21` current 2026-05-10) added under the same `serde` cargo feature for `Value::Custom` round-trip; transitively pulls `inventory` (verified `0.3.24` current 2026-05-10) for the discriminator registry.
- The `serde` cargo feature on `quartzite-core` does **not** force `std`. `typetag` and `inventory` both compile under `no_std + alloc` on every target quartzite currently builds for (verified by AC7). The known incompatibility is `wasm32-unknown-unknown` (no linker-section support for `inventory`); deferred per the *Deferred* table.
- `cargo build -p quartzite --no-default-features` MUST continue to compile (no_std + alloc path); the new layer is gated behind the `serde` cargo feature, off by default.
- `cargo clippy --workspace -- -D warnings` clean; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` clean.
- Errors via `thiserror` (workspace convention).
- New simple fns carry `#[inline]` or `_Simple._` per `ai-docs/code-style.md` § *`#[inline]` and the `_Simple._` doc tag*.
- Pre-publish API stability axiom (`AGENTS.md` § *API Stability*): no compat shims, no deprecation wrappers — clean breaks.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | **Property layer:** `Value::serialize` followed by `Value::deserialize` (any backend; tests pick one — likely `serde_json` for readability + `bincode` for binary correctness) round-trips bit-equal payloads for every concrete `Value` variant the v1 format supports (modulo `Float` NaN, where `f64::is_nan` is preserved but bit-exact equality is not asserted; `Value::Custom` round-trip is exercised separately by AC9). |
| AC2 | **Object layer:** an `ObjectSnapshot` produced from a `#[derive(Object)]` instance round-trips through `serialize` → `deserialize` → factory-construct → property-write, yielding an object whose `Stored` properties match the original by value. |
| AC3 | **Tree layer:** a `TreeSnapshot` of a populated `ObjectTree` round-trips through `serialize` → `deserialize` → tree-rebuild, yielding a tree whose root and every descendant carries the same `Stored` properties as the original; nested `WeakObjectRef`s are remapped to point at the corresponding new objects. |
| AC4 | Properties **without** the `Stored` bit are silently skipped on serialize and absent from the payload; on read they retain their default-constructed value. Verified by a unit test using a `#[derive(Object)]` type with at least one non-`Stored` property. |
| AC5 | Schema-version envelope: `TreeSnapshot` carries `schema_version: u32`; deserializing a payload with an unknown version returns `Err(DeserializeError::UnsupportedVersion { found, supported })` rather than panicking or silently mis-parsing. Verified by a unit test that hand-crafts a `schema_version = u32::MAX` payload. |
| AC6 | **Signal-connection drop:** an integration test connects two `#[derive(Object)]` instances inside a tree, snapshots the tree, restores it, and asserts the restored objects have empty `ConnectionTable`s and `signals_blocked == false`. |
| AC7 | `cargo build -p quartzite --no-default-features` continues to pass with the `serde` cargo feature off (no_std + alloc path stays compatible). |
| AC8 | `cargo clippy --workspace -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` both pass; new public items carry `# Examples` per `ai-docs/doc-convention.md`. |
| AC9 | `Value::Custom` round-trips bit-equal under the `typetag` policy: a `#[typetag::serde]`-annotated user type embedded in a property serializes to a tag-discriminated payload and deserializes back to a value comparing equal to the original. Verified by a unit test that exercises both serde_json (textual) and bincode (binary) backends on a hand-rolled `MyCustom { v: i64 }` impl. |
| AC10 | The serialization module's rustdoc carries a top-level `## Serialization` section describing the round-trip contract, the `Stored` gating, the `signals_blocked` / `ConnectionTable` non-persistence policy, the `#[typetag::serde]` requirement on `CustomValue` impls, and the v1 → v2 signal-reconnection deferral. |
| AC11 | `quartzite-core::meta::PropertyFlag::Stored` rustdoc is updated to point at the serialization module / feature. `CustomValue`'s rustdoc gains a `#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]`-gated `# Examples` block showing the `#[typetag::serde]` opt-in on a concrete impl. |
| AC12 | `INDEX.md` updated: this plan moves from the deferred footnote to "Active plans" / appropriate row; the "Deferred — #107" footnote is removed; #39's "blocked on #107" footnote is left intact (it stays blocked until #39 ships its own work on top of this layer). |

## Open questions

- **Atomicity of writes** — pushed to the caller for v1; if a real use-case needs a "write to temp + atomic rename" helper, file a follow-up.
- **Same-class-name collisions across crates** — the runtime factory currently keys on a `class_name: &'static str`. If two crates register the same name, restore behavior is first-wins. Out of scope for this layer; tracked separately if it becomes a problem in practice.
- **`typetag` tag collisions across user crates** — `typetag` defaults to the bare type name as the discriminator; two crates defining `Foo` (in different modules) collide on restore. Documented as a known limitation; users override via `#[typetag::serde(name = "my_crate::Foo")]`. Tracked separately if it becomes a real problem.
