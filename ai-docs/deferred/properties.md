# Properties

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Computed properties (stored = false + getter closure) \| use methods for now | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | | #56 |
| Property bindings (two-way sync) \| BindingEngine is future work | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | | #56 |
| `#[prop(stored = false)]` with custom getter/setter closure \| needs design decision on how closures are stored in static context | [macros spec](../plans/done/2026-05-01-macros.spec.md) | | #56 |
| `from_bits` / integer round-trip API — not needed today; can be added later | [enumflags2-property-flags spec](../plans/done/2026-05-03-enumflags2-property-flags.spec.md) |  | — |
| Reconnecting signals on restore (v2) — Round-1 answer: drop now, re-establish in v2 once reflection lookup is robust | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| Backwards-translation between schema versions (v1 → v2 reader) — Only meaningful once a v2 ships | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| Object-graph snapshots that span multiple `ObjectTree`s — Multi-window / multi-tree story still being designed | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| `wasm32-unknown-unknown` support for the `serde` feature — `typetag` depends on `inventory`, which uses linker-section tricks unavailable on `wasm32-unknown-unknown`. Quartzite's runtime is currently desktop-targeted (wgpu/winit), so this is not a v1 blocker. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Serialization / deserialization (serde) support | [enumflags2-property-flags spec](../plans/done/2026-05-03-enumflags2-property-flags.spec.md) |  | — |
| FFI / integer wire format | [enumflags2-property-flags spec](../plans/done/2026-05-03-enumflags2-property-flags.spec.md) |  | — |
| Any changes to other metadata types (`SignalMeta`, `MethodMeta`, etc.) | [enumflags2-property-flags spec](../plans/done/2026-05-03-enumflags2-property-flags.spec.md) |  | — |
| **`ConnectionTable` entries** — never serialized (they hold runtime `Fn`-trait closures with no portable representation). Documented as transient. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| **Reconnecting signals after restore** — explicitly v2 per the round-1 answer; tracked separately when this lands. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| **QML-style declarative load** — out of scope; this layer is serde-driven binary/textual round-trip only. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| **Migration / upgrade tooling** — schema-version field exists; conversion between major versions is a future task once a v2 ships. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| **Atomic write helpers** — `to_writer` / `from_reader` operate on `serde::Serializer` / `serde::Deserializer` (or `io::Write` / `io::Read` for the `bincode`-style backends); partial-write atomicity is the caller's responsibility. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| ~~**Bincode major version**~~ **Resolved 2026-05-10:** pin `bincode = "2"` explicitly — the registry's `max_stable_version` is `3.0.0`, but `bincode 3.0.0/src/lib.rs` is literally `compile_error!("https://xkcd.com/2347/");` — a placeholder reservation, not a working release. The functional latest is `2.0.1`; pinning major `2` per the workspace `0.x` / `x` rule. (Following the standard registry-query rule blindly here would produce an immediate build failure — this is a rare exception where the reviewer must inspect the crate's `lib.rs` after the registry query, documented inline in tasks 4 and 10.) `serde_json = "1"` (verified current 2026-05-10). | [object-property-serialization-layer design](../plans/done/2026-05-10-object-property-serialization-layer.design.md) |  | — |
| **Should `restore_tree` accept an `&mut ObjectTree` to merge into, in addition to building a fresh tree?** Spec says **fresh** (Key decision). Confirming no merge variant is wanted — caller wires the result up themselves. (Reviewer should sanity-check; if disagreed, an additional `restore_tree_into(&mut ObjectTree, &TreeSnapshot)` is a small follow-up.) | [object-property-serialization-layer design](../plans/done/2026-05-10-object-property-serialization-layer.design.md) |  | — |
| **Module name `snapshot` vs `serde`** — spec mentions both (`quartzite-core::serde` *or* `::snapshot`). Design picks `snapshot` — `serde` would shadow the external crate name and confuse imports (`use quartzite_core::serde::...` reads ambiguously). Reviewer can flip if preferred. | [object-property-serialization-layer design](../plans/done/2026-05-10-object-property-serialization-layer.design.md) |  | — |
| **`Application::install_factory` automation** — `Application::new()` already installs an empty `ObjectFactory`. Should the `serde` feature add a `pub fn register<T: Object + Default + 'static>(&mut self, name: &str)` ergonomic helper, or leave registration entirely to user code? Spec is silent. Design defers (YAGNI; users register today via the bare factory API and the test fixture demonstrates the pattern). | [object-property-serialization-layer design](../plans/done/2026-05-10-object-property-serialization-layer.design.md) |  | — |
| **Atomicity of writes** — pushed to the caller for v1; if a real use-case needs a "write to temp + atomic rename" helper, file a follow-up. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| **Same-class-name collisions across crates** — the runtime factory currently keys on a `class_name: &'static str`. If two crates register the same name, restore behavior is first-wins. Out of scope for this layer; tracked separately if it becomes a problem in practice. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
| **`typetag` tag collisions across user crates** — `typetag` defaults to the bare type name as the discriminator; two crates defining `Foo` (in different modules) collide on restore. Documented as a known limitation; users override via `#[typetag::serde(name = "my_crate::Foo")]`. Tracked separately if it becomes a real problem. | [object-property-serialization-layer spec](../plans/done/2026-05-10-object-property-serialization-layer.spec.md) |  | — |
