#![cfg_attr(docsrs, feature(doc_cfg))]
//! A GUI and object framework for Rust drawing on Qt's signals/slots and
//! property/reflection model — implemented in idiomatic Rust with no native
//! dependencies, no foreign ABI, and no codegen outside proc-macros.
//!
//! quartzite organises application state as a tree of [`Object`](core::Object)s with
//! reflection metadata, communicates between objects via typed signals/slots,
//! exposes typed fields as introspectable properties, and dispatches work
//! through per-thread event loops. The [`prelude`] is a single glob-import
//! that brings the most-used types into scope.
//!
// Quickstart is gated on the `derive` feature: the example uses the
// `#[derive(Extend, Object)]` and `#[object_impl]` macros from
// `quartzite-macros`, which the prelude only re-exports when `derive` is on.
// Under `cargo test --doc --no-default-features --features std` the doctest
// would otherwise fail to compile-check. `cfg_attr(..., doc = ...)` injects
// the section only when the macros are actually available.
#![cfg_attr(
    feature = "derive",
    doc = r#"# Quickstart

```no_run
use quartzite::prelude::*;

#[derive(Extend, Object)]
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
    fn reset(&mut self) {
        self.count = 0;
    }
}

// Explicit `fn main` keeps the type definitions at module level — the
// `#[derive]` codegen emits paths that resolve relative to the containing
// module, which would not resolve under rustdoc's implicit-main wrapper.
fn main() {
    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::new(),
    };

    // Connect a slot to the count_changed signal.
    c.count_changed.connect(|args| println!("count is now {}", args.0));

    // Writing the property emits count_changed automatically.
    c.write_property("count", Value::Int(42));

    // Invoke the slot dynamically through the reflection layer.
    c.invoke_method("reset", &[]);
}
```

`Application::run` is intentionally not shown above — it would block the
example. The [`runtime`](crate::runtime) section below covers it."#
)]
//!
//! # Signals
//!
//! [`Signal`](core::signal::Signal) is a typed event source; a slot is any
//! closure receiving the signal's argument tuple. Connecting a slot is
//! allocation-free at the call site; emitting invokes every connected slot
//! synchronously on the emitting thread by default. Use the
//! [`emit!`](crate::core::emit) macro when the signal lives behind a guard
//! or when conditional emission is desired.
//!
//! ```ignore
//! use quartzite::prelude::*;
//!
//! let mut sig: Signal<(i32,)> = Signal::new();
//! sig.connect(|args| println!("got {}", args.0));
//! emit!(sig, (42,));
//! ```
//!
//! Cross-thread delivery uses [`ConnectionType::Queued`](core::signal::ConnectionType);
//! the slot then runs on the receiver's home thread via the
//! [`QueuedDispatcher`](core::QueuedDispatcher) instead of the emitter's.
//! See [`core::signal`] for the signal type and [`core`] for the connection-type
//! enum.
//!
//! # Properties
//!
//! Properties are typed fields registered with the reflection layer, accessible
//! by name through [`Object::read_property`](core::Object::read_property) and
//! [`Object::write_property`](core::Object::write_property). Mark a field with
//! `#[prop]`; pair it with a notify signal so writes auto-emit:
//!
//! ```ignore
//! #[prop(notify = age_changed)]
//! pub age: u32,
//! #[signal]
//! pub age_changed: Signal<(u32,)>,
//! ```
//!
//! Read and write through the reflection layer:
//!
//! ```ignore
//! use quartzite::prelude::*;
//!
//! let v = obj.read_property("age");
//! obj.write_property("age", Value::Int(30));
//! ```
//!
//! See [`core`] for the [`Value`](core::Value) enum and the
//! [`Object`](core::Object) trait that exposes the reflection layer.
//!
//! # Object tree
//!
//! [`ObjectTree`](runtime::ObjectTree) is a registry of named objects with
//! parent/child relationships. The runtime uses it to dispatch events,
//! resolve queued-connection targets, and answer name lookups:
//!
//! ```ignore
//! use quartzite::prelude::*;
//!
//! // Given a tree populated by the runtime:
//! let matches: &[ObjectId] = tree.find_by_name("save_button");
//! let scoped: Vec<ObjectId> = tree.find_by_name_in(window_id, "save_button");
//! ```
//!
//! See [`runtime::ObjectTree`].
//!
//! # Event loop
//!
//! [`Application`](runtime::Application) is the singleton-per-process entry
//! point; [`EventLoop`](runtime::EventLoop) is per-thread. Posting a closure
//! to an event loop runs it on that thread's next tick:
//!
//! ```ignore
//! use quartzite::prelude::*;
//!
//! let app = Application::main();
//! EventLoop::main().post(|| println!("ran on the main thread"));
//! app.run(); // blocks until quit
//! ```
//!
//! Per-thread event loops + [`ThreadDriver`](runtime::ThreadDriver) /
//! [`PoolDriver`](runtime::PoolDriver) / [`AppDriver`](runtime::AppDriver)
//! together provide the threading model. See [`runtime`].
//!
//! # Timers
//!
//! [`Timer`](runtime::Timer) schedules deferred or periodic work against an
//! execution context ([`AppDriver`](runtime::AppDriver), [`PoolDriver`](runtime::PoolDriver), [`ThreadDriver`](runtime::ThreadDriver)):
//!
//! ```ignore
//! use quartzite::prelude::*;
//!
//! let cfg = TimerConfig {
//!     interval_ms: 100,
//!     ..TimerConfig::default()
//! };
//! ```
//!
//! See [`runtime::Timer`] and [`runtime::TimerConfig`]. A complete example
//! lives at [`examples/timer.rs`](https://github.com/maratik123/quartzite/tree/master/examples/timer.rs).
//!
//! # Ecosystem
//!
//! quartzite ships as a workspace; the facade re-exports the most useful
//! pieces. Depend on the facade unless you have a specific reason not to:
//!
//! - [`core`] (`quartzite-core`) — object trait, signals, properties,
//!   reflection, value types. `no_std`-compatible when the `std` feature is
//!   disabled (requires `alloc`).
//! - [`runtime`] (`quartzite-runtime`) — [`Application`](runtime::Application), [`EventLoop`](runtime::EventLoop),
//!   [`ObjectTree`](runtime::ObjectTree), [`Timer`](runtime::Timer). Requires `std`.
//! - [`macros`] (`quartzite-macros`) — [`Extend`](macros::Extend), [`Object`](macros::Object),
//!   [`object_impl`](macros::object_impl), [`object_part`](macros::object_part), [`MetaEnum`](macros::MetaEnum). Requires the `derive` feature
//!   (on by default).
//! - [`geometry`] (`quartzite-geometry`) — [`Point`](geometry::Point), [`Size`](geometry::Size), [`Rect`](geometry::Rect),
//!   [`Margins`](geometry::Margins) and their floating-point `*F` variants.
//! - [`events`] (`quartzite-events`) — event-dispatch types built on top of
//!   the event-types primitives in `quartzite-event-types`.
//! - [`paint`] (`quartzite-paint-api`) — `no_std`-compatible shared paint vocabulary:
//!   [`Color`](paint::Color), [`Pen`](paint::Pen), [`Brush`](paint::Brush), [`BrushKind`](paint::BrushKind), [`Painter`](paint::Painter) trait, [`PaintError`](paint::PaintError).
//!
//! Add quartzite to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! quartzite = { git = "https://github.com/maratik123/quartzite" }
//! ```
//!
//! `quartzite-macros` is pulled in transitively when the `derive` feature is
//! enabled — there is no need to add it as a direct dependency.
//!
//! # Design notes
//!
//! - **`no_std` posture.** [`core`] compiles `no_std + alloc`; [`runtime`]
//!   requires `std`. The facade picks up `std` by default; disable the `std`
//!   feature for the `no_std + derive`-free path.
//! - **Threading model.** Each object is owned by a specific thread
//!   (`ObjectBase::thread_id`). Cross-thread signal delivery uses
//!   [`ConnectionType::Queued`](core::signal::ConnectionType); the slot runs
//!   on the receiver's home thread, routed through the per-thread
//!   [`EventLoop`](runtime::EventLoop) via the
//!   [`QueuedDispatcher`](core::QueuedDispatcher).
//! - **Tracing.** Structured spans via the [`tracing`](https://docs.rs/tracing)
//!   ecosystem; high-frequency paths (signal emit, event-loop post) are
//!   gated behind the `verbose-tracing` cargo feature so the cost stays out
//!   of release builds by default.
//!
//! # Feature flags
#![doc = document_features::document_features!()]

/// Re-exports the core object model, signals, and reflection types from [`quartzite_core`].
///
/// Prefer `use quartzite::core::Foo` over adding a direct `quartzite-core` dependency.
pub mod core {
    pub use quartzite_core::*;
}

/// Re-exports derive macros and attribute macros from [`quartzite_macros`].
///
/// Adding `quartzite` (with the `derive` feature, which is on by default) is sufficient — there
/// is no need for a direct dependency on `quartzite-macros` or `quartzite-core`. The macros
/// resolve all generated paths through `::quartzite::core` automatically.
///
/// Most macros ([`Extend`](macros::Extend), [`Object`](macros::Object), [`object_impl`](macros::object_impl), [`object_part`](macros::object_part)) are already included in [`prelude`].
///
/// Requires the `derive` feature (enabled by default).
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub mod macros {
    pub use quartzite_macros::*;
}

/// Re-exports the application runtime, event loop, and object tree from [`quartzite_runtime`].
///
/// Prefer `use quartzite::runtime::Foo` over adding a direct `quartzite-runtime` dependency.
pub mod runtime {
    pub use quartzite_runtime::*;
}

/// Re-exports geometry primitives from [`quartzite_geometry`].
///
/// Provides integer ([`Point`](geometry::Point), [`Size`](geometry::Size), [`Rect`](geometry::Rect), [`Margins`](geometry::Margins)) and float ([`PointF`](geometry::PointF), [`SizeF`](geometry::SizeF), [`RectF`](geometry::RectF)) types.
pub mod geometry {
    pub use quartzite_geometry::*;
}

/// Re-exports the event model from [`quartzite_events`].
///
/// Provides [`MouseEvent`](events::MouseEvent), [`KeyEvent`](events::KeyEvent), [`ResizeEvent`](events::ResizeEvent), [`CloseEvent`](events::CloseEvent), [`TimerEvent`](events::TimerEvent), [`EventFilter`](events::EventFilter), and [`EventType`](events::EventType).
pub mod events {
    pub use quartzite_events::*;
}

/// Snapshot capture and restore for objects and object trees.
///
/// Provides three levels of granularity:
///
/// - **Property level** — [`Value`](crate::core::Value) is `serde::Serialize` /
///   `serde::Deserialize`; any backend works.
/// - **Object level** — [`capture_object`](quartzite_runtime::snapshot::capture_object) /
///   [`restore_object`](quartzite_runtime::snapshot::restore_object) snapshot a single
///   object's [`Stored`](quartzite_core::meta::PropertyFlag::Stored) properties.
/// - **Tree level** — [`capture_tree`](quartzite_runtime::snapshot::capture_tree) /
///   [`restore_tree`](quartzite_runtime::snapshot::restore_tree) snapshot an entire
///   [`ObjectTree`](crate::runtime::ObjectTree) including parent/child structure.
///
/// Signal connections and `signals_blocked` are **not** preserved; re-establish
/// connections after restore.
///
/// Requires the `serde` feature.
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod snapshot {
    pub use quartzite_core::snapshot::{
        CURRENT_SCHEMA_VERSION, DeserializeError, ObjectNode, ObjectSnapshot, SerializeError,
        TreeSnapshot,
    };
    pub use quartzite_runtime::snapshot::{
        capture_object, capture_tree, restore_object, restore_tree,
    };
}

/// Re-exports the full backend-agnostic paint vocabulary from [`quartzite_paint`].
///
/// This module exposes [`Brush`](quartzite_paint_api::Brush) /
/// [`BrushKind`](quartzite_paint_api::BrushKind),
/// [`Color`](quartzite_paint_api::Color), [`Font`](quartzite_paint_api::Font) /
/// [`FontWeight`](quartzite_paint_api::FontWeight),
/// [`Image`](quartzite_paint_api::Image) /
/// [`ImageError`](quartzite_paint_api::ImageError),
/// [`Path`](quartzite_paint_api::Path) / [`Segment`](quartzite_paint_api::Segment),
/// [`PaintError`](quartzite_paint_api::PaintError),
/// [`Painter`](quartzite_paint_api::Painter), and [`Pen`](quartzite_paint_api::Pen),
/// plus [`Alignment`](quartzite_geometry::Alignment) (re-exported through
/// `quartzite-paint` for vocabulary completeness).
///
/// The concrete backend implementation lives in `quartzite-renderer`
/// (vello + wgpu + winit).
pub mod paint {
    pub use quartzite_paint::*;
}

/// Re-exports the styling layer ([`Style`](quartzite_style::Style),
/// [`StyleRegistry`](quartzite_style::StyleRegistry), [`Palette`](quartzite_style::Palette),
/// [`ColorRole`](quartzite_style::ColorRole)).
///
/// [`Palette`](quartzite_style::Palette) and [`ColorRole`](quartzite_style::ColorRole) originate in `quartzite-style-types` (the leaf
/// crate) and reach this module via [`quartzite_style`]'s own re-export —
/// a single `use quartzite::style::*;` therefore covers the full vocabulary.
/// `quartzite-style-types` is not a direct dependency of this crate, so it
/// is named in prose only, not linked.
///
/// Requires the `style` feature.
#[cfg(feature = "style")]
#[cfg_attr(docsrs, doc(cfg(feature = "style")))]
pub mod style {
    pub use quartzite_style::*;
}

/// Re-exports the built-in widget system from [`quartzite_widgets`].
///
/// Provides [`WidgetBase`](widgets::WidgetBase), [`WidgetExt`](widgets::WidgetExt), layout types ([`BoxLayout`](widgets::BoxLayout), [`GridLayout`](widgets::GridLayout)), and concrete
/// widgets ([`Label`](widgets::Label), [`Button`](widgets::Button), [`LineEdit`](widgets::LineEdit), [`TextEdit`](widgets::TextEdit), [`ScrollArea`](widgets::ScrollArea), [`Container`](widgets::Container)).
///
/// Requires the `widgets` feature.
#[cfg(feature = "widgets")]
#[cfg_attr(docsrs, doc(cfg(feature = "widgets")))]
pub mod widgets {
    pub use quartzite_widgets::*;
}

/// Re-exports a curated set of types needed for typical usage — one glob covers a working import.
///
/// Use `use quartzite::prelude::*;` to get the object model, signal types, derive
/// macros, and runtime in one line. For production code that values legibility,
/// import from [`core`], [`macros`], or [`runtime`] explicitly instead.
pub mod prelude {
    // quartzite-core: object model
    pub use quartzite_core::{
        AsObject, ConnectionId, Object, ObjectBase, ObjectExt, ObjectId, SignalCallback, Value,
        WeakObjectRef,
    };
    // quartzite-core: signals
    pub use quartzite_core::signal::{ConnectionType, Signal};
    // quartzite-core: emit! macro
    pub use quartzite_core::emit;
    // quartzite-core: std-only types
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub use quartzite_core::Mutex;
    // quartzite-core: std-only dispatcher API
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub use quartzite_core::{
        DispatcherAlreadySet, QueuedDispatcher, queued_dispatcher, set_queued_dispatcher,
    };
    // quartzite-core: signal-to-signal connection API
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub use quartzite_core::{SignalConnectionError, connect_signal_to_signal, connect_signals};

    // quartzite-macros: derive macros (requires derive feature)
    #[cfg(feature = "derive")]
    #[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
    pub use quartzite_macros::{Extend, Object, object_impl, object_part};

    // quartzite-runtime
    pub use quartzite_runtime::{
        AppDriver, Application, ApplicationError, EventLoop, ObjectRef, ObjectTree, PoolDriver,
        ThreadDriver, Timer, TimerConfig, TimerDriver, WeakRef,
    };

    // quartzite-paint-api: shared paint vocabulary
    pub use quartzite_paint_api::{
        Brush, BrushKind, Color, Font, FontWeight, Image, ImageError, PaintError, Painter, Path,
        Pen, Segment,
    };

    // quartzite-geometry: alignment (lives next to the paint vocabulary)
    pub use quartzite_geometry::Alignment;
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_compiles() {
        let _: ObjectId = ObjectBase::new().id();
    }
}
