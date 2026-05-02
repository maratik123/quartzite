# Design: Examples Crate

**Issue:** user description
**Date:** 2026-05-02

## Approach

Create a new workspace member `quartzite-examples` with four binary examples under an
`examples/` directory. Each example is a self-contained runnable program demonstrating a
distinct part of the public API.

**Macro dep requirement:** `quartzite-macros` codegen emits `::quartzite_core::` absolute
paths in all generated `impl` blocks. These resolve against the *user's* extern prelude —
so `quartzite-core` must be a direct dependency of any crate that uses the macros. This is
not a bug; it is the standard Rust proc-macro pattern. Future work with `proc_macro_crate`
could detect the right path at expansion time and lift this requirement, but that is deferred.

**Facade re-exports:** `quartzite/src/lib.rs` adds `pub use quartzite_core;` and
`pub use quartzite_runtime;`. Rationale:
- Users can reference types as `quartzite::quartzite_core::X` without an extra dep.
- Type identity is guaranteed when mixing facade and direct sub-crate deps in the same
  program — `quartzite::quartzite_core::ObjectBase` is the same type as `quartzite_core::ObjectBase`.
- Migration from facade to direct sub-crate requires only a `Cargo.toml` change.

## Facade public re-export (`quartzite/src/lib.rs`)

Add immediately after the `pub mod prelude` block. `#![deny(missing_docs)]` is active,
so both re-exports must carry a `///` doc comment:

```rust
/// Re-export of [`quartzite_core`]. Provides direct access to core types and ensures
/// type identity when mixing facade and direct sub-crate dependencies. Use
/// `quartzite_core` directly in your `Cargo.toml` when you need the full sub-crate API.
pub use quartzite_core;

/// Re-export of [`quartzite_runtime`]. Provides direct access to runtime types and
/// ensures type identity when mixing facade and direct sub-crate dependencies.
pub use quartzite_runtime;
```

## `quartzite-examples` Cargo.toml

```toml
[package]
name = "quartzite-examples"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Runnable examples for the quartzite object framework"
publish = false

[dependencies]
quartzite = { path = "../quartzite" }
quartzite-core = { path = "../quartzite-core" }
```

- `quartzite` — prelude, runtime access (`ObjectTree`, `Application`, `Timer`, …), derive macros
- `quartzite-core` — required for `::quartzite_core::` paths in macro-generated code
- `quartzite-runtime` NOT needed as a direct dep — no `::quartzite_runtime::` paths appear in macro output
- `publish = false` — not intended for crates.io

Root `Cargo.toml` workspace `members` gains `"quartzite-examples"`.

A minimal `src/lib.rs` satisfies Cargo's "at least one target" requirement:

```rust
//! Runnable examples for the quartzite object framework.
//! Run with: cargo run --example <name> -p quartzite-examples
```

No public items → `missing_docs` gate satisfied.

## Concrete API calls per example

### `hello_object.rs`

Shows `#[derive(Extend, DeriveObject)]` + `#[object_impl]`, property read/write, and
slot invocation. Also demonstrates using `quartzite_core::ObjectBase` directly (the
explicit import shows users they need the sub-crate dep).

```rust
use quartzite::prelude::*;
use quartzite_core::ObjectBase;

#[derive(Extend, DeriveObject)]
#[root]
struct Counter {
    #[base]
    object_base: ObjectBase,
    #[prop(notify = count_changed)]
    pub count: i32,
    #[signal]
    pub count_changed: Signal<(i32,)>,
}

#[object_impl]
impl Counter {
    #[slot]
    fn reset(&mut self) { self.count = 0; }
}

fn main() {
    let mut c = Counter { object_base: ObjectBase::new(), count: 0, count_changed: Signal::new() };
    println!("initial count: {:?}", c.read_property("count")); // Some(Int(0))
    c.write_property("count", Value::Int(42));
    println!("after write:   {:?}", c.read_property("count")); // Some(Int(42))
    c.invoke_method("reset", &[]);
    println!("after reset:   {:?}", c.read_property("count")); // Some(Int(0))
}
```

AC2 evidence: three `println!` lines with `Int(0)`, `Int(42)`, `Int(0)`.

### `signals_slots.rs`

Shows typed `Signal::connect` / `Signal::emit` and dynamic `Object::connect_signal`.
`SignalCallback = Box<dyn Fn(&[Value]) + Send + Sync>` — bare capturing-nothing closure
satisfies both bounds.

```rust
use quartzite::prelude::*;
use quartzite_core::ObjectBase;

#[derive(Extend, DeriveObject)]
#[root]
struct Greeter {
    #[base]
    object_base: ObjectBase,
    #[signal]
    pub greeted: Signal<(String,)>,
}

#[object_impl]
impl Greeter {}

fn main() {
    let mut g = Greeter { object_base: ObjectBase::new(), greeted: Signal::new() };
    g.greeted.connect(|args| println!("typed slot: hello, {}", args.0));
    g.connect_signal("greeted", Box::new(|vals| {
        println!("dynamic slot received {} value(s)", vals.len());
    }));
    g.greeted.emit(&(String::from("world"),));
}
```

AC3 evidence: both `println!` lines fire on the single `emit`.

### `object_tree.rs`

Shows `ObjectTree::insert` with parent, `parent_of`, `children_of`, `find_by_name`, `with`.
Uses a minimal `Node` type with no properties or signals.

```rust
use quartzite::prelude::*;
use quartzite_core::ObjectBase;

#[derive(Extend, DeriveObject)]
#[root]
struct Node {
    #[base]
    object_base: ObjectBase,
}

#[object_impl]
impl Node {}

impl Node {
    fn named(name: &str) -> Self { Self { object_base: ObjectBase::named(name) } }
}

fn main() {
    let mut tree = ObjectTree::new();
    let root_id  = tree.insert(Box::new(Node::named("root")),       None);
    let child_id = tree.insert(Box::new(Node::named("child")),      Some(root_id));
    let _gid     = tree.insert(Box::new(Node::named("grandchild")), Some(child_id));
    println!("parent of child:   {:?}", tree.parent_of(child_id));
    println!("children of root:  {:?}", tree.children_of(root_id));
    println!("find 'grandchild': {:?}", tree.find_by_name("grandchild"));
    tree.with(root_id, |obj| println!("root name: {:?}", obj.object_base().name()));
}
```

AC4 evidence: four `println!` lines showing parent ID, children slice, found-by-name, name.

### `timer.rs`

Shows `Application::new`, `Timer::new`, `Timer::connect_timeout`, `Timer::start`,
`Application::exec`, `Application::quit`. Stops after 3 ticks (~150 ms) — CI safe.
Does not use `quartzite_core` types in user code directly (no `use quartzite_core::` needed
in this file); only the crate-level dep is required for macro codegen in the other files.

```rust
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Duration;
use quartzite::prelude::*;

fn main() {
    let app = Application::new().expect("only one Application per process");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);
    let mut timer = Timer::new(Duration::from_millis(50));
    timer.connect_timeout(move |_| {
        let n = counter2.fetch_add(1, Ordering::SeqCst) + 1;
        println!("tick {n}");
        if n >= 3 { Application::global().expect("app").quit(); }
    });
    timer.start(app.event_loop().sender());
    app.exec();
    println!("done after {} ticks", counter.load(Ordering::SeqCst));
}
```

AC5 evidence: "tick 1"–"tick 3", then "done after 3 ticks".

## Decomposition

| # | Task | Files |
|---|------|-------|
| 1 | Add `quartzite-examples` to workspace `Cargo.toml` | `Cargo.toml` |
| 2 | Create `quartzite-examples/Cargo.toml` + `src/lib.rs` | `quartzite-examples/` |
| 3 | Add public sub-crate re-exports to `quartzite/src/lib.rs` | `quartzite/src/lib.rs` |
| 4 | `hello_object.rs` | `quartzite-examples/examples/hello_object.rs` |
| 5 | `signals_slots.rs` | `quartzite-examples/examples/signals_slots.rs` |
| 6 | `object_tree.rs` | `quartzite-examples/examples/object_tree.rs` |
| 7 | `timer.rs` | `quartzite-examples/examples/timer.rs` |
| 8 | Update `AGENTS.md` | `AGENTS.md` |
| 9 | Update `README.md` | `README.md` |
| 10 | Update `ai-docs/context.md` | `ai-docs/context.md` |

## Risks

- **`missing_docs` on re-exports:** mitigated by `///` doc comments on both `pub use` lines.
- **`Application` singleton in timer:** example is a separate binary — naturally isolated.
- **Timer in CI:** bounded at 3 ticks (~150 ms), will not block CI.
- **`quartzite-macros` tests unaffected:** codegen unchanged; no new dev-deps needed.

## Test Design

AC2–AC5: run each binary, verify exit code 0 and expected stdout:

| Example | Expected stdout |
|---------|-----------------|
| `hello_object` | `Int(0)` → `Int(42)` → `Int(0)` via `read_property` |
| `signals_slots` | Typed slot line + dynamic slot line, both from single `emit` |
| `object_tree` | Parent ID, children slice, found-by-name IDs, root name |
| `timer` | "tick 1"/"tick 2"/"tick 3", then "done after 3 ticks" |

AC1: `cargo build`
AC6: `cargo clippy -- -D warnings`
AC7: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`
AC8/AC9/AC10/AC11: manual file review

## Open questions

(none)
