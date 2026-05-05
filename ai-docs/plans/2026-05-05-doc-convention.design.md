# Design: Workspace-wide doc convention

**Issue:** #80
**Spec:** `ai-docs/plans/2026-05-05-doc-convention.md`
**Date:** 2026-05-05

## Approach

Adopt a single canonical convention based on RFC 1574 + Rust Book ch14-02 +
deterministic.space machine-readable inline-markdown convention. The
convention lives in `ai-docs/doc-convention.md` (new file, full text + worked
examples). `AGENTS.md` keeps a short pointer paragraph in *Code Style* so
agents see it on every session start.

Enforcement is layered:

1. **Compiler & clippy lints** in every `lib.rs` make the rule mechanical
   wherever the toolchain can check it (`broken_intra_doc_links`,
   `missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`,
   `doc_markdown`).
2. **Workspace `clippy.toml`** suppresses `doc_markdown` false positives via
   `doc-valid-idents`. The initial list is seeded from a one-pass grep of all
   `///` comments and grown during the audit.
3. **Section-order rule** is regex-checkable and goes into the review skill +
   both review agents. Lints cannot enforce ordering.
4. **Audit** every public item in all 6 crates in dependency order. Trait-impl
   methods are exempt (AC4); inherent items, free fns, and trait *definition*
   methods carry the full convention.
5. **Proc-macro codegen** is updated to emit conforming docs on every public
   inherent item it generates. Trait-impl items are exempt and may keep the
   minimal docs they have today.

The five-phase plan below produces a single PR. CI fails after Phase 1 and
stays red until the audit (Phase 2) and codegen update (Phase 3) finish; this
is intentional and documented in the progress file.

### Rejected alternatives

- **Per-crate convention files.** Six near-identical copies with drift risk.
  Rejected — the convention is uniform; one file is the source of truth.
- **Custom doc-lint script.** A fmt-style markdown linter for section order.
  Rejected — `clippy::doc_markdown` plus the review-agent regex check covers
  the realistic failure modes; a custom binary is YAGNI.
- **Per-crate `clippy.toml`.** Cargo merges only the *closest* `clippy.toml`
  per package; per-crate files mean drift on `doc-valid-idents`. One workspace
  `clippy.toml` at the root is the standard pattern.
- **Bring trait-impl methods into scope.** Would force documenting `From::from`,
  `Drop::drop`, `Display::fmt`, etc. across ~60 sites. Rejected per spec key
  decision and per ergonomics — the trait definition already documents the
  contract.

## Audit-scope sizing

Counts from `rg` over each crate. *Trait-impl pub fns* are exempt per AC4 and
inherent + trait-definition + free fns are subject to the full convention.
"Inherent + trait-def fns" is `total pub fn lines` minus the body of trait-impl
blocks; the figures below are the upper bound — the actual subject set is the
non-trait-impl subset.

| Crate | `pub fn` total | `pub struct` | `pub enum` | `pub trait` | trait-impl blocks | inherent impl blocks | Subject pub fns (est.) |
|---|---:|---:|---:|---:|---:|---:|---:|
| `quartzite-core` | 48 | 15 | 3 | 7 | 28 | 14 | ~28 |
| `quartzite-events` | 21 | 5 | 6 | 2 | 6 | 5 | ~17 |
| `quartzite-geometry` | 46 | 7 | 0 | 0 | 6 | 7 | ~40 |
| `quartzite-macros` | 13 | 0 | 0 | 1 | 1 | 4 | ~13 |
| `quartzite-runtime` | 51 | 12 | 1 | 0 | 25 | 12 | ~30 |
| `quartzite` (facade `src/`) | 0 | 0 | 0 | 0 | 0 | 0 | 0 (re-exports only) |

`pub use` lines are not counted as items — re-exports inherit doc from the
upstream item. `pub macro_rules!` is zero across the workspace.

`quartzite-geometry` is the largest absolute audit target (~40 inherent fns,
mostly accessors); `quartzite-runtime` and `quartzite-core` carry the most
*conditional-section* burden (`# Errors`, `# Panics` for fallible / panicking
APIs in `Application`, `ConnectionTable`, `signal::set_queued_dispatcher`).

## Existing doc-style baseline

Sampled 10 representative items per crate. Findings:

- **Summary tense.** Already overwhelmingly third-person present indicative
  ("Returns the…", "Creates a new…"). A handful of imperatives remain (e.g.
  `signal::set_queued_dispatcher` — "Register the process-wide queued
  dispatcher.", `Signal::new` — "Create a new signal…", `EventLoop::new`,
  `ObjectFactory::new`, `ObjectTree::new`). Estimate ~10–15 sites per crate.
- **`# Examples` blocks.** Present on virtually every public item — the
  `public-api-docs` plan already enforced this rule. Some `unsafe`-style or
  `no_run`-needing items use `no_run` correctly. Spot check: ~95% coverage on
  `quartzite-events`, `quartzite-geometry`, `quartzite-core`. `quartzite-macros`
  uses `no_run` per spec.
- **`# Parameters` sections.** **None present.** AC13's MouseEvent::new is the
  flagship example — it has six parameters with no `# Parameters` block.
  Almost every multi-arg fn (`MouseEvent::new`, `KeyEvent::new`,
  `Application::post_event`, `ObjectTree::insert`, `Signal::connect_queued`,
  `Signal::connect_auto`, `ConnectionTable::register`, `Margins::new`,
  `Rect::new`, `Size::new`, etc.) is missing it.
- **`# Errors` / `# Panics` / `# Safety`.** **None present** in `///` blocks.
  Several APIs need them:
  - `# Errors`: `Application::new` (`Result<_, ApplicationError>`),
    `signal::set_queued_dispatcher`, `ObjectFactory::install`,
    `ConnectionTable::install_as_dispatcher`, `value::FromValue::from_value`
    impls (trait def has no `# Errors`; inherent `pub fn` returning `Result`
    doesn't either).
  - `# Panics`: `EventLoop::run` panics on internal channel poisoning,
    `Application::exec` re-asserts main-thread invariants, `Timer::start` /
    `Timer::stop` use `expect("…")` patterns. Each call site needs to be
    audited for panic-on-precondition behaviour.
  - `# Safety`: no `unsafe fn` in the public API today. Lint stays as a
    safety net for future additions.
- **Section ordering.** Where sections do appear (only `# Examples` mostly),
  ordering is consistent. The new convention adds the strict order; nothing
  to *reorder* on initial pass — only items to *insert*.

## Proc-macro emitted-doc audit

Every `quote!` site in `quartzite-macros/src/` that emits a `pub` item:

| File | Line | Site | Item kind | Subject to convention? |
|---|---:|---|---|---|
| `extend/codegen.rs` | 65–77 | `emit_root_trait_and_impl` — `pub trait As{Self}` with two trait-def methods (`#acc`, `#acc_mut`) | trait definition (user-facing) | **Yes** — trait def methods need docs |
| `extend/codegen.rs` | 71–76 | `impl As{Self} for {Self}` — self-ref accessor pair | trait impl | exempt (AC4) |
| `extend/codegen.rs` | 139–153 | `impl AsObject for {Self}` — 4 trait methods | trait impl | exempt |
| `extend/codegen.rs` | 178–189 | `impl As{Parent} for {Self}` — delegation pair | trait impl | exempt |
| `extend/codegen.rs` | 209–216 | `impl As{Mixin} for {Self}` — mixin pair | trait impl | exempt |
| `object/codegen.rs` | 95–98 | `pub const __PROPS__{T}` | inherent const inside `#[doc(hidden)]` mod | exempt (`#[doc(hidden)]`) |
| `object/codegen.rs` | 121–124 | `pub const __SIGNALS__{T}` | inherent const inside `#[doc(hidden)]` mod | exempt |
| `object/codegen.rs` | 142–149 | `pub fn __lookup_property_{T}` | fn inside `#[doc(hidden)]` mod | exempt |
| `object/codegen.rs` | 163–170 | `pub fn __lookup_signal_{T}` | fn inside `#[doc(hidden)]` mod | exempt |
| `object/codegen.rs` | 186–195 | `pub fn __read_property_{T}` | fn inside `#[doc(hidden)]` mod | exempt |
| `object/codegen.rs` | 230–241 | `pub fn __write_property_{T}` | fn inside `#[doc(hidden)]` mod | exempt |
| `object/codegen.rs` | 273–287 | `pub fn __connect_signal_dynamic_{T}` | fn inside `#[doc(hidden)]` mod | exempt |
| `object/codegen.rs` | 310–323 | `pub fn emit_{signal}` | inherent method on user struct | **Yes** — has 1-line doc, needs `# Parameters` (when ≥1 sig arg) and `# Examples` |
| `object/codegen.rs` | 341–363 | `pub fn connect_{signal}_auto` | inherent method on user struct | **Yes** — has 3-line doc, needs `# Parameters`, `# Examples` |
| `object/codegen.rs` | 388–411 | `pub fn connect_{signal}_queued` | inherent method on user struct | **Yes** — already has `# Examples` (no_run); needs `# Parameters` |
| `object_impl/codegen.rs` | 33–43 | `emit_impl_block` re-emits user's impl block verbatim | user-authored | not generated docs — user's responsibility |
| `object_impl/codegen.rs` | 70–73 | `const __METHODS__{T}` (no `pub`) | private const | n/a |
| `object_impl/codegen.rs` | 124–134 | `fn __invoke_method_{T}` (no `pub`) | private fn | n/a |
| `object_impl/codegen.rs` | 154–166 | `fn __lookup_method_{T}`, `fn __lookup_enum_{T}` (no `pub`) | private fn | n/a |
| `object_impl/codegen.rs` | 187–205 | `static META_{T}` (no `pub`), `fn __meta_init_{T}` (no `pub`) | private | n/a |
| `object_impl/codegen.rs` | 224–253 | `impl Object for {Self}` — 5 trait methods | trait impl | exempt (AC4) |
| `object_part/codegen.rs` | 8–10 | re-emits user's `impl` block verbatim | user-authored | not generated docs |
| `meta_enum/codegen.rs` | 56–82 | private statics + private lookup fns | n/a | n/a |
| `meta_enum/codegen.rs` | 85–111 | `impl IntoValue for {T}`, `impl FromValue for {T}` | trait impl | exempt |

**Generated user-facing public items needing convention-conforming docs:**

1. `pub fn emit_{signal}` (`object/codegen.rs:295–323`)
2. `pub fn connect_{signal}_auto` (`object/codegen.rs:332–363`)
3. `pub fn connect_{signal}_queued` (`object/codegen.rs:376–412`)
4. `pub trait As{Self}` + the two trait-definition methods `#acc`/`#acc_mut`
   (`extend/codegen.rs:65–69`)

Phase 3 updates these four `quote!` sites to interpolate doc comments that
include `# Parameters` (where args > 0) and `# Examples`. The
`#[doc(hidden)]`-mod-wrapped helpers and trait-impl methods stay untouched.

## Lint-enabling strategy

Each crate's `lib.rs` gets the same five lint attributes. Insertion point: the
existing `#![deny(missing_docs)]` attribute block. Order rule: keep
`#![cfg_attr(...)]`, `#![no_std]`, and `#![cfg_attr(docsrs, ...)]` first; then
the new lints; then `#![deny(missing_docs)]`; then `#![doc = ...]`.

Concrete insertions:

```rust
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
```

Per-crate `lib.rs` placement:

| Crate | Insert after this existing attribute |
|---|---|
| `quartzite-core/src/lib.rs` | line 7 (`#![cfg_attr(docsrs, feature(doc_cfg))]`) — before `#![deny(missing_docs)]` (line 8) |
| `quartzite-events/src/lib.rs` | line 1 (`#![no_std]`) — before `#![deny(missing_docs)]` (line 2) |
| `quartzite-geometry/src/lib.rs` | line 1 (`#![no_std]`) — before `#![deny(missing_docs)]` (line 2) |
| `quartzite-macros/src/lib.rs` | line 10 (last `///` doc) — before `#![deny(missing_docs)]` (line 11) |
| `quartzite-runtime/src/lib.rs` | line 5 (last `//!` line) — before `#![deny(missing_docs)]` (line 6) |
| `src/lib.rs` (facade) | line 2 (`#![deny(missing_docs)]`) — insert before it (after line 1's `#![cfg_attr(docsrs, feature(doc_cfg))]`) |

Because CI runs `cargo clippy -- -D warnings`, the four `warn`-level lints
become hard errors. This is intentional — the audit is the path to clean.

### Initial `clippy.toml` `doc-valid-idents` allowlist

Workspace root `clippy.toml`. The allowlist supplements clippy's built-in list
(which already includes `OpenGL`, `WebGL`, `JSON`, `HTTP`, etc., so most of the
acronyms found are already exempt). Project-specific seed list, derived from
grep across `///` doc comments:

```toml
doc-valid-idents = [
    # quartzite types — exact CamelCase names that appear in prose
    "AsObject",
    "AsButton",
    "AsWidget",
    "AsLayout",
    "BitFlags",
    "CloseEvent",
    "ConnectionId",
    "ConnectionTable",
    "ConnectionType",
    "CustomValue",
    "DispatcherAlreadySet",
    "EnumEntry",
    "EnumMeta",
    "EventFilter",
    "EventLoop",
    "EventType",
    "FactoryAlreadySet",
    "FromValue",
    "IntoValue",
    "KeyEvent",
    "KeyEventKind",
    "KeyModifier",
    "KeyModifiers",
    "LayoutBase",
    "MetaEnum",
    "MetaObject",
    "MethodMeta",
    "MixinTypeName",
    "MouseButton",
    "MouseButtons",
    "MouseEvent",
    "MouseEventKind",
    "ObjectBase",
    "ObjectExt",
    "ObjectFactory",
    "ObjectId",
    "ObjectRef",
    "ObjectTree",
    "ParamMeta",
    "ParentTypeName",
    "PointF",
    "PropertyFlag",
    "PropertyFlags",
    "PropertyMeta",
    "QueuedDispatcher",
    "ReceiverGuard",
    "RectF",
    "ResizeEvent",
    "SignalCallback",
    "SignalMeta",
    "SingleShot",
    "SizeF",
    "SlotKey",
    "SlotMap",
    "StyleBase",
    "ThreadId",
    "ThreadPool",
    "TimerEvent",
    "TokenStream",
    "TypeError",
    "TypeName",
    "WeakObjectRef",
    "WeakRef",
    "WidgetBase",
    # third-party crate types in prose
    "IndexMap",
    "RwLock",
    # short acronyms used in prose
    "GPU",
    # Rust-/build-config tokens
    "no_std",
]
```

Clippy's defaults already cover `OpenGL`, `WebGL`, `JSON`, `HTML`, `CSS`,
`HTTP`, `HTTPS`, `URL`, `XML`, `YAML`, `TLS`, `TCP`, `UDP`, `IP`, `RAM`,
`CPU`, `CamelCase`, etc., so we do not list them.

The list is **seed only**. The Phase 2 per-crate audit appends entries when
new `doc_markdown` warnings appear — for example identifiers used only inside
prose like `MyClass`, `MyCustom`, `MyEvent`, `TestDispatcher` (these are
example-block identifiers; if they appear in `///` outside fenced blocks they
need adding, otherwise they don't).

## Section-ordering enforcement design

The convention mandates this order:

```
Summary
# Parameters
# Returns
# Type parameters
# Lifetimes
# Errors
# Panics
# Safety
# Examples
# See also
```

Mechanical check (regex over `///` heading lines on a single public item):

```
^/// # (Parameters|Returns|Type parameters|Lifetimes|Errors|Panics|Safety|Examples|See also)\s*$
```

Collect the matched headings in document order, then assert the resulting
sequence is a subsequence of the canonical order above. Any other heading
under a `///` block (e.g. a stray `# Notes`) is also a finding: the convention
forbids ad-hoc sections.

**Review-agent checklist item language** (added to both
`.claude/agents/review-findings.md` and `.claude/agents/self-review.md` under
the existing **Documentation** section):

- *Doc convention conformance.* For every `pub` item touched by the diff
  (skip trait-impl methods — exempt per `ai-docs/doc-convention.md`),
  verify:
  - **Summary line.** First `///` line is third-person singular present
    indicative ("Returns…", "Creates…"). Imperative ("Create…", "Return…")
    or progressive ("Creating…") → REJECT.
  - **`# Parameters`.** Required when the fn has ≥1 argument other than
    `self`/`&self`/`&mut self`. Missing → REJECT.
  - **`# Returns`** — required when the return type is non-`()` and not
    obvious from the summary; reviewer judgement.
  - **`# Errors`** — required when the fn returns `Result`. Missing →
    REJECT (also flagged by `clippy::missing_errors_doc`).
  - **`# Panics`** — required when the fn has any `panic!`, `unreachable!`,
    `unwrap()`, `expect("…")`, `[index]`, slicing, or arithmetic that can
    overflow on plausible inputs. Missing → REJECT.
  - **`# Safety`** — required when the fn is `unsafe`. Missing → REJECT.
  - **Section order.** Headings must appear in canonical order
    (Parameters → Returns → Type parameters → Lifetimes → Errors → Panics
    → Safety → Examples → See also). Out-of-order → REJECT.
  - **No ad-hoc sections.** Only the canonical `#` headings above plus
    `# See also` are allowed. Stray sections → REJECT.
  - **Intra-doc links.** Type / fn references in prose use `` [`Type`] ``
    rather than backtick-only or fully qualified paths where intra-doc
    resolves. Reviewer judgement.

## `ai-docs/doc-convention.md` content outline

Standalone reference doc — agents read it without conversation context.

```
# Documentation Conventions

## Scope
- Applies to every public item in every workspace crate.
- Does NOT apply to `#[doc(hidden)]` items, private items, or trait-impl
  methods (the trait definition documents the contract).

## References
- Rust by Example — Documentation: <URL>
- RFC 1574 — More API documentation conventions: <URL>
- The Rust Book ch14-02 — Making useful documentation comments: <URL>
- Pascal Hertleif — Machine-readable inline Markdown code documentation: <URL>

## Summary line
- One sentence.
- Third person singular present indicative ("Returns…", "Creates…",
  "Constructs…", "Sets…"). Not imperative, not progressive, not future.
- Period at end. American English spelling.

## Section order (strict)
Summary
# Parameters
# Returns
# Type parameters
# Lifetimes
# Errors
# Panics
# Safety
# Examples
# See also

Sections must appear in this order. Missing optional sections is fine; out-of-order is not.

## Always-present sections
- `# Examples` — every public item (existing rule). Doctests must compile.
  Use `no_run` for proc-macro examples and runtime items needing an event
  loop; pure library types use compiling doctests.
- `# Parameters` — every fn with ≥1 argument other than the receiver
  (`self`/`&self`/`&mut self`). One bullet per argument:
    - `name`: brief description (units, ranges, ownership semantics).

## Conditional sections
- `# Returns` — when the return type is non-trivial and not obvious from the
  summary. Skip for simple getters where the summary already names the
  returned value.
- `# Errors` — required when the fn returns `Result`. List each error variant
  and the condition that produces it.
- `# Panics` — required when the fn can panic on a precondition the caller
  controls (`unwrap`/`expect`, indexing, arithmetic overflow, asserted
  invariants).
- `# Safety` — required for every `unsafe fn`. List the invariants the caller
  must uphold.
- `# Type parameters` — when generic bounds are non-obvious (e.g. why
  `Args: Clone + Send` is required for `connect_queued`).
- `# Lifetimes` — when lifetime relationships are non-obvious (rare in this
  codebase).

## Optional section
- `# See also` — bulleted list of related items, each as an intra-doc link.

## Linking
- Prefer intra-doc links (`` [`Type`] ``, `` [`Type::method`] ``).
- Use the full generic name in prose (`Option<T>`, not `Option`) where the
  generic parameter is meaningful.
- Cross-crate links: `` [`quartzite_core::ObjectBase`] ``; the
  workspace-local relative form (`super::Type`) only inside the same crate.

## Language
- American English. Spell check: "behavior" not "behaviour", "color" not
  "colour", "serialize" not "serialise".

## Trait-impl exemption
- Methods inside `impl Trait for Type { … }` blocks inherit docs from the
  trait definition. Do not duplicate.
- Generated trait impls (e.g. `From`, `Into`, `Display`, `Drop`) are
  exempt for the same reason.
- The trait *definition* (`pub trait Foo { fn bar(...); }`) is **not**
  exempt — every method declared in the trait carries the convention.

## Conforming example

  /// Creates a new mouse event.
  ///
  /// # Parameters
  ///
  /// - `position`: cursor position in widget-local coordinates.
  /// - `global_position`: cursor position in screen coordinates.
  /// - `event_button`: the button that triggered the event (empty for moves).
  /// - `buttons_state`: bitmask of all currently pressed buttons.
  /// - `modifiers`: active keyboard modifiers at event time.
  /// - `kind`: which kind of mouse event this is.
  ///
  /// # Examples
  ///
  /// ```
  /// use quartzite_events::{MouseButton, MouseButtons, MouseEvent, MouseEventKind};
  /// use quartzite_geometry::Point;
  ///
  /// let e = MouseEvent::new(
  ///     Point::new(0, 0),
  ///     Point::new(0, 0),
  ///     MouseButton::Left.into(),
  ///     MouseButton::Left.into(),
  ///     Default::default(),
  ///     MouseEventKind::Press,
  /// );
  /// assert!(e.event_button().contains(MouseButton::Left));
  /// ```
  #[inline]
  pub fn new(
      position: Point,
      global_position: Point,
      event_button: MouseButtons,
      buttons_state: MouseButtons,
      modifiers: KeyModifiers,
      kind: MouseEventKind,
  ) -> Self { … }

## Non-conforming example (with annotations)

  /// Create a new mouse event.            // ← imperative ("Create"); should be "Creates"
  ///
  /// # Examples                            // ← Examples appears before Parameters; wrong order
  /// …
  ///
  /// # Parameters
  /// - position: …                         // ← bullets must put `name` in backticks: `position`

## Lints that mechanically enforce parts of this convention
- `#![deny(missing_docs)]` — every public item has at least a one-line doc.
- `#![deny(rustdoc::broken_intra_doc_links)]` — intra-doc links must resolve.
- `#![warn(clippy::missing_errors_doc)]` — `# Errors` on `Result`-returning fns.
- `#![warn(clippy::missing_panics_doc)]` — `# Panics` on panicking fns.
- `#![warn(clippy::missing_safety_doc)]` — `# Safety` on `unsafe` fns.
- `#![warn(clippy::doc_markdown)]` — flags un-backticked CamelCase / acronyms.

## Behavioural enforcement
- `code-review` skill (`.claude/skills/code-review/SKILL.md`) and the
  `review-findings` / `self-review` agents check tense and section order
  (mechanical lints cannot).
```

## Implementation breakdown

| # | Subtask | Files touched | Tests added/changed | Verify gate |
|---|---|---|---|---|
| 1 | Write `ai-docs/doc-convention.md` (canonical reference). Add `## Documentation Conventions` paragraph to `AGENTS.md` *Code Style* section linking to the doc. Create workspace-root `clippy.toml` with the seed `doc-valid-idents` list. | `ai-docs/doc-convention.md` (new), `AGENTS.md`, `clippy.toml` (new) | none | files exist; markdown links resolve via `realpath` per AGENTS.md rule |
| 2 | Add the five new lint attributes to every crate's `lib.rs` (per the table above). | `quartzite-core/src/lib.rs`, `quartzite-events/src/lib.rs`, `quartzite-geometry/src/lib.rs`, `quartzite-macros/src/lib.rs`, `quartzite-runtime/src/lib.rs`, `src/lib.rs` | none | `cargo build --workspace` still compiles; `cargo clippy` is now expected red — record the failure list to seed Phase 2 |
| 3 | **Audit & fix `quartzite-core`.** Walk every public item; bring summary lines to third-person present; add `# Parameters` for every fn with ≥1 arg; add `# Errors` to every `Result`-returning fn; add `# Panics` to every fn with `expect`/`unwrap`/index; reorder sections per convention. | `quartzite-core/src/{id,meta,object_base,receiver_guard,signal,traits,value}.rs` | doctests already exist on most items; convert any imperative-summary doctest preamble; add doctests to any items that gain `# Examples` for the first time | `cargo clippy -p quartzite-core --all-targets -- -D warnings` clean; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc -p quartzite-core --no-deps`; `cargo test -p quartzite-core` |
| 4 | **Audit & fix `quartzite-geometry`.** Largest crate by accessor count. Tense pass + add `# Parameters` to `Point*::new`, `Size*::new`, `Rect*::new`, `Margins::new`, `Margins::apply`. Most accessors stay one-liner + `# Examples`. | `quartzite-geometry/src/{lib,point,rect,size,margins}.rs` | none new (accessors are already covered) | `cargo clippy -p quartzite-geometry --all-targets -- -D warnings` clean; doc gate; `cargo test -p quartzite-geometry`; `cargo build -p quartzite-geometry --no-default-features` |
| 5 | **Audit & fix `quartzite-events`.** Add `# Parameters` to `MouseEvent::new` (AC13 — flagship), `KeyEvent::new`, `TimerEvent::new`, `ResizeEvent::new`. Tense check on enums and `Event` / `EventFilter` traits. | `quartzite-events/src/{lib,event,keyboard,mouse,timer,window}.rs` | extend the `MouseEvent` test module with `mouse_event_new_parameters_doctest` — a doctest in the `# Examples` of `MouseEvent::new` exercising `event_button` and `buttons_state` separately. Equivalent doctest fields under `KeyEvent::new` for symmetry. | `cargo clippy -p quartzite-events --all-targets -- -D warnings` clean; doc gate; `cargo test -p quartzite-events` (doctests must be green); `cargo build -p quartzite-events --no-default-features` |
| 6 | **Audit & fix `quartzite-macros`** (the four exported proc-macros and any exported helper). Tense pass on the macro doc comments themselves; add `# Parameters` for the macros that document attribute syntax (formal "Attributes" subsection — keep as-is, but ensure no convention-breaking sections appear). | `quartzite-macros/src/lib.rs` | extend existing tests in `quartzite-macros/src/{object,object_part,extend}/codegen.rs` `#[cfg(test)]` modules to assert: emitted `pub fn emit_<sig>` body contains `"# Parameters"` and `"# Examples"`; emitted `pub fn connect_<sig>_auto` body contains `"# Parameters"` and `"# Examples"`; emitted `pub trait As<Self>` accessor methods have non-empty `///` lines. Use string-contains assertions in the same style as existing tests (`out.contains("...")`). | `cargo clippy -p quartzite-macros --all-targets -- -D warnings` clean; doc gate; `cargo test -p quartzite-macros` |
| 7 | **Update `quartzite-macros` codegen.** Change four `quote!` sites to interpolate full convention-conforming doc comments on emitted user-facing items. (a) `emit_signal_wrappers` — add `# Parameters` listing `arg0..argN` with their tuple-element types; keep existing `# Examples` if absent add a `no_run` example. (b) `emit_connect_auto_wrappers` — add `# Parameters` (`receiver`, `f`); add `# Examples` no_run. (c) `emit_connect_queued_wrappers` — already has `# Examples`; add `# Parameters`. (d) `emit_root_trait_and_impl` — add doc comment per trait method (`#acc`, `#acc_mut`) summarising "Returns a shared/mutable reference to this object." Trait-impl methods generated by these blocks remain exempt (no docs needed). | `quartzite-macros/src/{object,extend}/codegen.rs` | the new tests added in subtask 6 lock the contract | `cargo clippy -p quartzite-macros --all-targets -- -D warnings`; doc gate against `quartzite` workspace pulls in macro-expanded tests — also run the integration tests `cargo test -p quartzite-macros --test object_impl --test object --test extend --test meta_enum`; for spot-check, run `cargo expand -p quartzite --example hello_object` (if `cargo-expand` is installed) and confirm emitted docs |
| 8 | **Audit & fix `quartzite-runtime`.** Largest conditional-section burden. Add `# Errors` on `Application::new`, `ObjectFactory::install`, `ConnectionTable::install_as_dispatcher`. Add `# Panics` on `Application::exec`, `EventLoop::run`, `Timer::start`, `Timer::stop`. Tense pass throughout; reorder sections. | `quartzite-runtime/src/{lib,application,connection_table,event_loop,factory,object_ref,object_tree,thread_pool,timer}.rs` | none structurally new — existing tests cover behaviour; new doctests on items where `# Errors`/`# Panics` blocks gain `no_run` snippets | `cargo clippy -p quartzite-runtime --all-targets -- -D warnings`; doc gate; `cargo test -p quartzite-runtime` |
| 9 | **Audit & fix `quartzite` facade (`src/lib.rs`).** Module-level docs and module re-export docs only. Tense pass; reorder if any heading exists; ensure module docs do not raise `doc_markdown` warnings. | `src/lib.rs` | the existing `prelude_compiles` test is enough | `cargo clippy -p quartzite --all-targets -- -D warnings`; doc gate; `cargo test -p quartzite`; `cargo build -p quartzite --no-default-features` (AC10) |
| 10 | **Update review skill + agents** (Propagation Rule). Add the *Doc convention conformance* checklist item shown above to `.claude/agents/review-findings.md` (under §5 Style or new §6 Documentation conformance) and `.claude/agents/self-review.md` (under §6 Documentation). In `.claude/skills/code-review/SKILL.md` add a sentence in the verify step: "Reviewers must check `ai-docs/doc-convention.md` conformance for every changed `pub` item." Run the Propagation grep procedure (`grep -rn "doc-convention" .claude/agents/ .claude/skills/ AGENTS.md`) to confirm no other instruction file references the convention. | `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md` | none | grep cross-check returns only the three files above |
| 11 | **Final workspace verification.** | none | none | run, in order, and require all clean: `cargo fmt -- --check`; `cargo clippy --workspace --all-targets -- -D warnings` (AC7); `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` (AC8); `cargo test --workspace` (AC9); `cargo build -p quartzite --no-default-features` (AC10). Update `ai-docs/plans/INDEX.md` to flip the entry to ✅ implemented and move the spec + design files into `ai-docs/plans/done/`. |

### Mapping subtasks → acceptance criteria

| AC | Covered by subtask(s) |
|---|---|
| AC1 (`ai-docs/doc-convention.md`) | 1 |
| AC2 (`AGENTS.md` pointer) | 1 |
| AC3 (every public item conforms) | 3, 4, 5, 6, 8, 9 |
| AC4 (trait-impl exemption) | doc-convention.md text (subtask 1); enforced in 3, 4, 5, 6, 8, 9 by skipping; verified in 11 |
| AC5 (lints in every `lib.rs`) | 2 |
| AC6 (`clippy.toml` allowlist) | 1, grown during 3–9 |
| AC7 (`cargo clippy` clean) | 11 |
| AC8 (`cargo doc` clean) | 11 |
| AC9 (`cargo test` green) | 11 |
| AC10 (`no_std` build) | 4, 5, 9, 11 |
| AC11 (`quartzite-macros` codegen emits conforming docs) | 6 (tests), 7 (codegen) |
| AC12 (review skill + agents updated together) | 10 |
| AC13 (`MouseEvent::new` `# Parameters` for `event_button`/`buttons_state`) | 5 |

### Context-reset handoff

Subtask count is **11**, well above the `/task` skill's threshold of 5. The
`/context-reset` handoff after subtask 3 (`quartzite-core` audit complete) is
**mandatory**. A second handoff is recommended after subtask 7 (codegen
update complete) given the size of the remaining workspace audit + agent
changes. The progress file template lives in
`.claude/skills/context-reset/SKILL.md`.

## Risks

- **CI red between Phase 1 and end of Phase 2.** Lints land first, audit
  follows. Mitigated by sequencing all subtasks in a single PR; the PR is
  not merged until subtask 11 passes. The subtask 2 verify step explicitly
  *expects* clippy red and records the failure list as the audit worklist.
- **`clippy::doc_markdown` false positives explode mid-audit.** New
  identifiers will surface during 3–9. Mitigated by appending to
  `clippy.toml` `doc-valid-idents` as they appear; restart the per-crate
  verify gate after each addition.
- **Doctest churn.** Adding `# Parameters` does not change doctests, but
  rewording summary lines may break `assert_eq!` checks if they referenced
  doc text (none found in audit). Mitigated by running `cargo test` after
  each crate.
- **Proc-macro emitted docs interact with the destination crate's
  `#![deny(missing_docs)]`.** Confirmed already enforced — every emitted
  `pub fn` already carries at least a one-line doc. The Phase 3 update
  *adds* sections; it does not remove the existing one-liner. No new gate
  failures expected.
- **`cargo expand` not always installed.** Subtask 7's `cargo expand`
  spot-check is informational; the locking contract is the
  string-contains tests in subtask 6 (`out.contains("# Parameters")` on
  the `quote!` output). Tests are mandatory; expand is optional.
- **AGENTS.md rule on `# Examples` mandates doctests.** Some items in
  generated code use `no_run` (proc-macro per spec). Reaffirmed in
  `doc-convention.md` so reviewers don't flip-flop.
- **Trait-impl exemption ambiguity.** Spelled out in `doc-convention.md`:
  exempt = methods *inside* `impl Trait for Type {}` blocks (including
  derives like `From`, `Drop`, `Display`, `Default`). Trait *definition*
  methods (`pub trait Foo { fn bar(); }`) are **not** exempt.
- **`#[doc(hidden)]` items.** Out of scope (the convention only governs
  public API surface). The proc-macro hidden mod (`mod __quartzite_*`) is
  exempt; documented in `doc-convention.md` *Scope* section.

## Test design

For each subtask's tests:

- **Subtask 5 (events)** — extend `quartzite-events/src/mouse.rs`
  `#[cfg(test)] mod tests` with the existing `mouse_event_multi_button` test
  pattern. The new doctest for AC13 lives in the `# Examples` block of
  `MouseEvent::new` and exercises `event_button()` + `buttons_state()`
  returning the values passed in. Sketch:

  ```text
  /// # Parameters
  /// - `event_button`: the button that triggered the event …
  /// - `buttons_state`: bitmask of all currently pressed buttons …
  ///
  /// # Examples
  ///
  /// ```
  /// use quartzite_events::{MouseButton, MouseButtons, MouseEvent, MouseEventKind};
  /// use quartzite_geometry::Point;
  ///
  /// let pressed: MouseButtons = MouseButton::Left | MouseButton::Right;
  /// let e = MouseEvent::new(
  ///     Point::new(0, 0), Point::new(0, 0),
  ///     MouseButton::Left.into(), pressed,
  ///     Default::default(), MouseEventKind::Press,
  /// );
  /// assert!(e.event_button().contains(MouseButton::Left));
  /// assert!(e.buttons_state().contains(MouseButton::Right));
  /// ```
  ```

- **Subtask 6 (macros codegen tests)** — add three tests to the existing
  `#[cfg(test)] mod tests` blocks:

  - `quartzite-macros/src/object/codegen.rs::tests`:

    ```rust
    #[test]
    fn emit_wrapper_doc_contains_parameters_and_examples() {
        let out = emit(quote! {
            struct Foo { #[signal] pub moved: Signal<(i32, i32)> }
        });
        assert!(out.contains("# Parameters"), "missing # Parameters in emit wrapper doc: {out}");
        assert!(out.contains("# Examples"),  "missing # Examples in emit wrapper doc: {out}");
    }

    #[test]
    fn connect_auto_wrapper_doc_contains_parameters_and_examples() {
        let out = emit(quote! {
            struct Foo { #[signal] pub ticked: Signal<(i32,)> }
        });
        assert!(out.contains("# Parameters"), "missing # Parameters in connect_auto wrapper doc: {out}");
        // # Examples already verified by existing connect_queued_wrapper_generated_for_signal
        assert!(out.contains("# Examples"),  "missing # Examples in connect_auto wrapper doc: {out}");
    }
    ```

  - `quartzite-macros/src/extend/codegen.rs::tests`:

    ```rust
    #[test]
    fn root_trait_methods_carry_docs() {
        let out = emit(quote! { #[root] struct Widget { x: i32 } });
        // The trait-definition method docs must be emitted (trait DEFINITION,
        // not trait IMPL) — convention applies.
        assert!(
            out.contains("/// Returns") || out.contains("# [doc"),
            "missing doc on root-trait accessor: {out}"
        );
    }
    ```

  Pattern matches the existing `out.contains(...)` assertion style used by
  every other test in these modules, so no new test infrastructure is
  needed. Token spacing (e.g. `# Parameters` becomes `# Parameters` after
  `quote!` round-trip; verified empirically — `quote!` preserves space-
  delimited tokens in attribute strings).

- **No new integration tests in `tests/`.** The four existing
  `quartzite-macros/tests/{extend,meta_enum,object,object_impl}.rs`
  integration tests will silently start exercising the new emitted docs
  (because `#![deny(missing_docs)]` is asserted on the test crate via
  default Cargo behaviour for proc-macro outputs in the calling crate).
  The unit tests above are the explicit lock.

- **No markdown linter.** Out of scope. Validate `ai-docs/doc-convention.md`
  links manually (per AGENTS.md rule).

## Open questions

(none — all key decisions are in the spec; this design merely sequences and
sites the changes.)

## Notes for the implementer

- Always prefix the implementer's worklist with this file path; the
  agent reads the design first.
- The progress file lives at
  `ai-docs/plans/2026-05-05-doc-convention.progress.md` (created by the
  `/task` Implementation step, not by this design).
- After subtask 11 succeeds, move both `2026-05-05-doc-convention.md` (the
  spec) and `2026-05-05-doc-convention.design.md` to
  `ai-docs/plans/done/`. Update `ai-docs/plans/INDEX.md` to flip the row
  to ✅ implemented with a test count of "0 new structural tests; +3
  codegen contract tests; +1 doctest (AC13)".
