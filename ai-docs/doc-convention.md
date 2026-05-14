# Documentation Conventions

This document is the canonical reference for `///` and `//!` doc-comment style
across the workspace. Every public item in every workspace crate
(`quartzite`, `quartzite-core`, `quartzite-events`, `quartzite-geometry`,
`quartzite-macros`, `quartzite-runtime`) must conform.

## Scope

- **Applies to:** every public item — `pub fn`, `pub struct`, `pub enum`,
  `pub trait`, `pub union`, `pub macro_rules!`, exported proc-macros, and
  every method declared inside a `pub trait`.
- **Does NOT apply to:**
    - `#[doc(hidden)]` items.
    - Private items (`fn`, `struct`, etc. without `pub`).
    - Methods inside `impl Trait for Type { … }` blocks (trait-impl methods —
      see *Trait-impl exemption* below).

## References

The convention is the union of the four sources cited in issue #80:

- [Rust by Example — Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)
- [RFC 1574 — More API documentation conventions](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text)
- [The Rust Book — Making useful documentation comments](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments)
- [Pascal Hertleif — Machine-readable inline Markdown code documentation](https://deterministic.space/machine-readable-inline-markdown-code-cocumentation.html)

## Summary line

- One sentence on the first line of the doc comment.
- **Third person singular present indicative.** Write `Returns the…`,
  `Creates a new…`, `Constructs a…`, `Sets the…`. Not imperative
  (`Return…`), not progressive (`Returning…`), not future (`Will return…`).
- Period at the end.
- American English spelling: `behavior`, `color`, `serialize`,
  `initialize`, not `behaviour`, `colour`, `serialise`, `initialise`.

## Section order (strict)

When multiple `#`-headed sections are present, they must appear in this order:

1. Summary line (no heading).
2. Free-form prose paragraphs (optional).
3. `_Simple._` marker line (optional — see *Simple-fn tag* below).
4. `# Parameters`
5. `# Returns`
6. `# Type parameters`
7. `# Lifetimes`
8. `# Errors`
9. `# Panics`
10. `# Safety`
11. `# Examples`
12. `# See also`

Missing sections that are not required-by-the-rules-below is fine. Reordering
is **not** fine: agents and reviewers check this mechanically.

## Always-present sections

- **`# Examples`** — every public item. Doctests must compile.
    - Use `no_run` for proc-macro examples and items that need an event loop
      to run (per existing AGENTS.md rule).
    - Pure library types use compiling doctests.
    - Plural form (`# Examples`) even if only one example is shown
      (RFC 1574).
- **`# Parameters`** — every fn or method with **≥1 argument** other than the
  receiver (`self` / `&self` / `&mut self`). Format:

    ```
    /// # Parameters
    ///
    /// - `name`: brief description (units, ranges, ownership semantics).
    /// - `other`: …
    ```

  The argument identifier is in backticks; description follows after a colon
  and a single space.

## Conditional sections

- **`# Returns`** — when the return type is non-trivial and not already
  obvious from the summary. Skip for simple getters whose summary already
  names the returned value.
- **`# Errors`** — required when the fn returns `Result`. List each error
  variant and the precondition that produces it. Enforced by
  `clippy::missing_errors_doc`.
- **`# Panics`** — required when the fn can panic on a precondition the caller
  controls (`unwrap` / `expect`, indexing, arithmetic overflow, asserted
  invariants). Enforced by `clippy::missing_panics_doc`.
- **`# Safety`** — required for every `unsafe fn` and every `_unchecked`
  variant. List the invariants the caller must uphold to avoid undefined
  behaviour. Enforced by `clippy::missing_safety_doc`.
- **`# Type parameters`** — when generic bounds are non-obvious (e.g. why
  `Args: Clone + Send + Sync + 'static` is required for queued-connection
  signal forwarding).
- **`# Lifetimes`** — when lifetime relationships between parameters and
  return values are non-obvious. Rare in this codebase.

## Optional section

- **`# See also`** — bulleted list of related items as intra-doc links. Use
  to point readers at sibling APIs that expand on the topic.

## Simple-fn tag

- **`_Simple._`** — italic marker line that publishes the recursive "simple"
  property defined in `AGENTS.md` *Code Style → `#[inline]` and the
  `_Simple._` doc tag*. Calls into a tagged fn count as "free" in the
  recursive `#[inline]`-budget rule.
- **Form:** a single line `/// _Simple._` placed **immediately after the
  summary line (and any free-form prose paragraphs), before any `#`
  heading.** Use exactly the underscore-italic, capitalised form shown — it
  must be searchable as `rg '_Simple\._'` for audit.
- **When required:**
    - Generic functions (own `<T>` / `<F: ...>` / `impl Trait` parameter)
      that satisfy the recursive simple definition. They get the tag
      *instead of* `#[inline]` — monomorphization makes the body available
      cross-crate, so the attribute would be redundant; the tag is the
      human-visible signal.
    - **Methods inside `impl<T> ...` or `impl<T> Trait for Foo<T>` blocks
      where `Self` is parametrised by the impl block's generics**, even
      when the method itself declares no extra type parameters. The body
      is monomorphized per concrete `T`, so it is generic-shaped from the
      compiler's point of view.
        - **Inherent impl** (`impl<T> Foo<T> { fn ... }`): place
          `/// _Simple._` directly under the summary line, as for any
          generic free fn — the method has its own docs.
        - **Trait impl** (`impl<T> Trait for Foo<T> { fn ... }`): place
          `// _Simple._` (regular line comment, **not** `///`) directly
          above the `fn` keyword inside the impl block. Rustdoc treats
          a `///` line on a trait-impl method as *overriding* the trait's
          inherited docstring — `/// _Simple._` on `ObjectRef<T>::clone`
          would replace `Clone::clone`'s actual docstring with just
          `_Simple._` on the rustdoc page. A regular `//` comment is
          invisible to rustdoc, stays a human-only marker visible in
          source, and preserves the trait-inherited docs. Audit grep
          `rg '_Simple\._'` matches both forms. Examples:
          `Signal<Args>::default`, `ObjectRef<T>::clone`,
          `WeakRef<T>::eq`.
    - Trait method declarations whose **every conforming impl is required to
      be simple** — typically because the impl is generated by a
      derive / proc-macro in this workspace (e.g. `AsObject::object_base`
      generated by `#[derive(Extend)]`), or because the trait's contract
      makes only simple impls valid. A hand-written impl whose body is
      non-simple violates the trait contract.
- **When forbidden:**
    - Concrete simple fns (no own type parameters AND `Self` is concrete —
      i.e. the surrounding `impl` block introduces no generics) — they
      use the `#[inline]` attribute, which is itself the visible marker.
      **This includes methods inside `impl Trait for ConcreteFoo` blocks**
      (concrete impl on a concrete struct): they take `#[inline]` per the
      concrete row, **not** `// _Simple._`. The comment form has no
      codegen effect, so a concrete trait-impl method marked with only
      `// _Simple._` would lose cross-crate inlining without LTO. The
      `// _Simple._` form is reserved for `impl<T> Trait for Foo<T>`
      (generic-impl) — there it avoids overriding rustdoc inheritance,
      and `#[inline]` would be redundant because monomorphization
      already exports the body per concrete `T`.
    - Default trait methods inside a `pub trait` body whose own bodies
      declare no type parameters (e.g. `ObjectExt::id`,
      `ObjectExt::is_on_current_thread`) — carve-out keeps them in the
      concrete row; use `#[inline]`. See `AGENTS.md` Code Style →
      `#[inline]` and the `_Simple._` doc tag for the rationale.
    - Trait method declarations whose contract admits non-simple conforming
      impls (e.g. `Object::read_property` contains `match` branches in the
      generated impl). Tagging here would overclaim simplicity and mislead
      callers.
- **Maintenance.** Strip `_Simple._` (and the parallel `#[inline]` attribute
  on concrete fns) in the same edit that makes a previously-simple fn
  non-simple. A stale tag is actively misleading because callers count
  calls into the fn as "free" in the recursive budget rule. After
  de-tagging, `rg` for callers and re-evaluate each — cascade until
  quiescent. See `AGENTS.md` Code Style → "Marker maintenance" for the
  full rule.
- **Conforming example:**

  ```rust
  /// Renames the object `id` to `new_name`, updating the name index.
  ///
  /// _Simple._
  ///
  /// # Parameters
  ///
  /// - `id`: identifier of the object to rename.
  /// - `new_name`: new name; replaces any existing name and updates the
  ///   by-name index.
  ///
  /// # Examples
  /// // …
  pub fn rename(&mut self, id: ObjectId, new_name: impl Into<String>) {
      fn inner(this: &mut ObjectTree, id: ObjectId, new_name: String) {
          // body
      }
      inner(self, id, new_name.into())
  }
  ```

## Feature flags rendering (`document_features`)

Crates that invoke [`document_features`](https://crates.io/crates/document_features)
to render their `Cargo.toml` `[features]` table into rustdoc must obey two
conventions — one in the source file, one in `Cargo.toml`. Both are
load-bearing for the public docs.html that ships to GitHub Pages.

### Macro placement

Place the `#![doc = document_features::document_features!()]` invocation
**inline within the `//!` crate doc**, immediately after a
`## Feature flags` heading (or `# Feature flags` — pick the heading level
that matches sibling sections like `# Examples` already used in the same
crate doc). The remaining inner attributes (lints, `cfg_attr`, etc.) follow
the macro line. Canonical shape:

```rust
//! Crate overview…
//!
//! # Examples
//! …
//!
//! # Feature flags
#![doc = document_features::document_features!()]
#![other lint attributes…]
```

**Forbidden positions:**

- **Before** the `//!` block — the rendered feature list appears first,
  ahead of the human-curated overview (inverts reading priority).
- **After** the entire attribute block with no preceding `## Feature flags`
  heading — the features render as an unlabelled appendix with no TOC anchor
  (no deep-link target, no sidebar entry, no search match).

### Cargo.toml feature sectioning

Group entries in `[features]` by audience using `#! ### <Section>`
section headings. `document_features` parses these into rustdoc subsections.

- **Main features** (default-on, commonly toggled, affects build target /
  API surface): listed first under no extra heading, just `## per-feature`
  doc strings.
- **Diagnostic features** (purely additive observability — tracing spans,
  debug instrumentation, profiling hooks): under `#! ### Diagnostic features`
  with a one-paragraph `#!` intro stating "Off by default. Enabling these is
  purely additive and only affects observability, never correctness or
  behaviour."
- Other categories (`#! ### Experimental features`,
  `#! ### Optional dependencies`, …) follow the canonical
  `document_features` example as needed.

Crates that **don't** invoke `document_features!()` (e.g. a crate with only
a `verbose-tracing` flag) don't need section headings — the comments would
be inert decoration.

### Why

- Reader priority: the human-curated overview is what readers want first.
  An auto-generated feature list before it forces every visitor to scroll
  past low-value content.
- Anchorability: a section without an `## Feature flags` heading has no TOC
  anchor; readers can't link to it, can't search for it in the sidebar,
  can't deep-link from external docs.
- Decision fatigue: mixing diagnostic features with main features makes
  every reader evaluate "do I need this?" on every line. Section headings
  communicate "main features matter; diagnostic features are additive
  observability" at a glance.

### How to apply

- When adding `document_features` to a new crate, write the source file in
  the canonical shape above before any other lint attributes are added —
  easier than retrofitting later.
- When adding a feature to an existing `[features]` table:
  1. Decide its audience: build-target / API-surface (main) vs.
     observability-only (diagnostic) vs. experimental / in-development.
  2. If it's diagnostic and the table doesn't already have a
     `#! ### Diagnostic features` section, add one (with the standard
     intro paragraph) before adding the feature.
  3. Place the `## per-feature` docstring immediately above the feature
     line.
- When reviewing a PR that touches `document_features`-using crates, check
  both: (a) macro is inline within `//!` after a heading; (b) any new
  feature lands under the right `#! ###` section per its audience.

## Feature-gated documentation

Documentation that references items behind `#[cfg(feature = "X")]` re-exports
or modules requires two distinct precautions. Both have failed silently in
this workspace and broken `RUSTDOCFLAGS="-D warnings"` builds.

### Doctests on feature-gated items

When a doctest exercises items that exist only behind a cargo feature
(`#[cfg(feature = "serde")]` re-export, `serde::Serialize` derive, etc.),
the doctest must be **`cfg_attr`-gated**, not `no_run`-gated.

`no_run` only suppresses doctest *execution*. It still runs `rustc` against
the doctest body, so a missing type behind a disabled feature fails the
build with `error[E0432]: unresolved import`.

The correct guard is to gate the entire doctest behind the feature so it
is omitted from compilation when the feature is off:

```rust
/// # Examples
///
#[cfg_attr(feature = "serde", doc = "```")]
#[cfg_attr(not(feature = "serde"), doc = "```ignore")]
/// use my_crate::serde_gated::Foo;
/// let f: Foo = serde_json::from_str("…").unwrap();
/// # assert!(true);
/// ```
pub use serde_gated::Foo;
```

When the feature is on, the fence is the runnable form (` ``` `); when off,
it is `ignore`d so rustdoc skips the body entirely. **Never** use `no_run`
on a doctest that *imports* a feature-gated item — `no_run` is for
"compiles but cannot run" (event loops, GUI apps), not for "should not
even compile under this configuration."

### Intra-doc links to feature-gated modules — `--all-features` everywhere

When a `pub use` or intra-doc link in a crate's prose points into a
feature-gated module, every site that builds rustdoc for that crate must
enable the feature, or `rustdoc::broken_intra_doc_links` fires under
`-D warnings`. The convention here is **`--all-features` everywhere** so
that adding a new gated module never requires editing a flag list — and
no gated module ever silently slips out of the doc build:

1. **`.github/workflows/ci.yml`** — the `cargo doc` invocation in the
   docs job runs with `--all-features`.
2. **`.github/workflows/docs.yml`** (the GitHub Pages publish workflow) —
   `cargo doc` runs with `--all-features`. Drift between CI and Pages
   produced one bug; using the same flag in both forecloses it.
3. **`Cargo.toml`** — `[package.metadata.docs.rs]` uses
   `all-features = true` in every crate whose docs.rs page may reference
   feature-gated items. Hand-picked `features = […]` lists are the failure
   mode — they go stale the moment a new gated module is added.

The local `AGENTS.md` *Build & Test* doc-gate command (and the matching
copies in `.claude/skills/task/SKILL.md`, `.claude/skills/code-review/SKILL.md`,
and `.claude/agents/self-review.md`) are **the local mirror** of site 1 —
they exist so an agent reproduces the CI doc gate before pushing. They use
the same `--all-features` flag.

The `--all-features` convention assumes features are additive (no two
features mutually exclusive). When a crate carries **mutually-exclusive**
or **conditionally-paired** features (e.g. `std` vs `libm` float backends,
backend A vs backend B), `all-features = true` and `--all-features` are
**forbidden** for that crate — they activate both alternatives
simultaneously, which compiles only by accident and documents a
combination no real downstream consumer would ever pick. Use this carve-out
in place of the default rule:

- **`[package.metadata.docs.rs]`** — set `no-default-features = true` and
  list an explicit `features = [...]` selecting the combination that yields
  the richest, most representative public-API documentation. Typically
  this is the `std` (or main) path plus every purely-additive feature
  (`serde`, `derive`, `style`, …), explicitly excluding the no-std-only
  alternative. Re-audit this block whenever a new feature lands in the
  crate.
- **CI / Pages workflows** — keep `--all-features` only if it still
  compiles cleanly. If two mutually-exclusive features now force a choice,
  narrow the flag set to a coherent superset that matches the
  `[package.metadata.docs.rs]` selection. Drift between workflow flags and
  docs.rs metadata is the failure mode (PR #339 reviewer flagged
  `quartzite-paint-api` and root `quartzite` for `all-features = true`
  activating both `std` and `libm`).
- **Self-review / `code-review`** — when the diff introduces a mutually-
  exclusive feature pair, the `all-features = true` REJECT does NOT
  fire for the affected crate; the reviewer instead verifies the explicit
  `features = [...]` list is present and representative.

The default `--all-features` / `all-features = true` rule still applies to
every crate whose features remain additive.

Reviewer check: when the diff touches `#[cfg(feature)]`-gated public
modules / re-exports, or modifies any `[features]` table, confirm the
workflow `cargo doc` invocations and every crate's docs.rs metadata still
use `--all-features` / `all-features = true`. A diff that swaps either of
those for a hand-picked subset is a regression.

## Linking and code references

- **Backtick every Rust identifier in prose.** Type names, function names,
  module names, build-config tokens like `no_std`, third-party crate
  types — all must be wrapped in backticks (`` `MouseEvent` ``,
  `` `RwLock` ``, `` `no_std` ``). The `clippy::doc_markdown` lint
  enforces this — see the *Lints* section below.
- Prefer **intra-doc links** over plain backticks when the reference is a
  navigation target: `` [`Type`] ``, `` [`Type::method`] ``,
  `` [`crate_name::Type`] ``. Rustdoc resolves them at build time, and
  `#![deny(rustdoc::broken_intra_doc_links)]` catches stale links.
- **Use the inline form `` [`Foo`](path::Foo) `` (text + `(target)`) for
  intra-doc links — NOT the reference form `` [`Foo`][path::Foo] ``.** Both
  render to identical HTML, but the workspace convention is inline so
  readers and reviewers see one consistent form. Inline also matches the
  `std` / `tokio` / `serde` ecosystem default, reducing cognitive load for
  newcomers. Reference form is permitted only when the target genuinely
  contains characters that break the inline form (rare for Rust paths).
  When touching a file with reference-form links, convert them in the
  same edit.
- Use the full generic name in prose (`Option<T>`, not `Option`) where the
  generic parameter is meaningful.
- Cross-crate links use the workspace crate name:
  `` [`quartzite_core::ObjectBase`] ``. Inside the same crate, prefer the
  shortest unambiguous form (`` [`super::Type`] `` or just `` [`Type`] ``).
- **Cross-crate links must target direct dependencies.** A
  `` [Type](other_crate::path) `` link only resolves when `other_crate` is
  a direct entry in the linking crate's `[dependencies]`. If the target
  lives in a transitive dep, either re-route through a direct dep that
  re-exports it, or demote the link to plain backticked prose. The
  `--all-features` doc gate (PR #197) compiles every gated module, so a
  link that bypasses a direct dep fails
  `#![deny(rustdoc::broken_intra_doc_links)]` at build time.

  ```text
  // Before — broken: quartzite_style_types is a transitive dep of quartzite,
  //                  not a direct one.
  /// [`Palette`](quartzite_style_types::Palette)
  // After — works: quartzite_style is a direct dep and re-exports Palette.
  /// [`Palette`](quartzite_style::Palette)
  ```

- **Backticked identifiers in prose should be intra-doc links when in scope.**
  A bare `` `Foo` `` in prose renders as inert code-style text. When `Foo`
  resolves to an item in scope (own crate, or a direct dep), wrap it as
  `` [`Foo`] `` so the reader can navigate. Leave bare when the candidate is
  a generic placeholder (`T`, `Self`, `K`), a Cargo feature name
  (`` `serde` ``, `` `std` ``), or an English noun that happens to be
  capitalized (the concept *Object*, the tool *Cargo*, *GUI*).

  ```text
  // Before — inert text:
  /// quartzite organises application state as a tree of `Object`s.
  // After — clickable link:
  /// quartzite organises application state as a tree of [`Object`]s.
  ```

- **Bare qualified paths follow the same direct-dep rule.** A backticked
  qualified path in prose (`` `pkg::path::Item` ``) should be wrapped as an
  intra-doc link (`` [`pkg::path::Item`] ``) when `pkg` is a direct dep of
  the linking crate. When it isn't, the link is structurally impossible —
  leave bare. Don't introduce a broken link to "match the convention." The
  most common case in this workspace is a downstream crate referenced from
  an upstream one (e.g., `quartzite-core` mentioning `quartzite_runtime::*`):
  the dep edge runs the other way, so the link cannot exist.

  ```text
  // Inside quartzite-core (which does NOT depend on quartzite-runtime).
  // Before — would-be broken if linkified: quartzite_runtime is downstream.
  /// Capture and restore functions live in `quartzite_runtime::snapshot`.
  // After — unchanged: bare backticks are correct here. A
  // [`quartzite_runtime::snapshot`] would fail intra-doc-link resolution.
  /// Capture and restore functions live in `quartzite_runtime::snapshot`.
  ```

## Language

- American English throughout: `behavior`, `color`, `serialize`,
  `initialize`. The clippy default is `en-us`-aware.

## Trait-impl exemption

- Methods inside `impl Trait for Type { … }` blocks are **exempt** from the
  full convention. They inherit docs from the trait definition.
    - This includes both manually-written impls (`impl Display for Foo`) and
      compiler-generated derives (`#[derive(Debug, Default, Clone, …)]`).
    - It also covers std-lib trait method-impls (`From`, `Into`, `Drop`,
      `Display`, `Debug`, `Default`, etc.) and user-defined-trait impls.
- The trait *definition* (`pub trait Foo { fn bar(...); }`) is **NOT**
  exempt. Every method declared inside a `pub trait` body must carry the
  full convention (summary, `# Parameters`, conditional sections,
  `# Examples`).
- `#[doc(hidden)]` impls are out of scope for the convention as a whole.

## Conforming example

```rust
/// Creates a new mouse event.
///
/// # Parameters
///
/// - `position`: cursor position in widget-local coordinates.
/// - `global_position`: cursor position in screen coordinates.
/// - `event_button`: the button that triggered the event (empty for pure
///   moves with no button change).
/// - `buttons_state`: bitmask of all buttons currently pressed at event time.
/// - `modifiers`: keyboard modifiers active at event time.
/// - `kind`: which kind of mouse event this is (press / release / move / …).
///
/// # Examples
///
/// ```
/// use quartzite_events::{MouseButton, MouseEvent, MouseEventKind};
/// use quartzite_geometry::Point;
///
/// let event = MouseEvent::new(
///     Point::new(0, 0),
///     Point::new(0, 0),
///     MouseButton::Left.into(),
///     MouseButton::Left.into(),
///     Default::default(),
///     MouseEventKind::Press,
/// );
/// assert!(event.event_button().contains(MouseButton::Left));
/// ```
#[inline]
pub fn new(
    position: Point,
    global_position: Point,
    event_button: MouseButtons,
    buttons_state: MouseButtons,
    modifiers: KeyModifiers,
    kind: MouseEventKind,
) -> Self {
    // …
}
```

## Non-conforming example (with annotations)

```rust
/// Create a new mouse event.        // ← imperative; should be "Creates"
///
/// # Examples                       // ← Examples appears before Parameters
/// // …                                wrong order — Parameters must come first
///
/// # Parameters
/// - position: …                    // ← `position` must be in backticks
pub fn new(/* … */) -> Self { /* … */ }
```

The fixes are: change `Create` → `Creates` (third-person present); move
`# Parameters` above `# Examples`; wrap each parameter name in backticks.

## Lints that mechanically enforce parts of this convention

Each crate's `lib.rs` enables:

- `#![deny(missing_docs)]` — every public item has at least a one-line
  doc. (Already present in every crate.)
- `#![deny(rustdoc::broken_intra_doc_links)]` — intra-doc links must
  resolve.
- `#![warn(clippy::missing_errors_doc)]` — `# Errors` section on every
  `Result`-returning public fn.
- `#![warn(clippy::missing_panics_doc)]` — `# Panics` section on every
  fn that can panic.
- `#![warn(clippy::missing_safety_doc)]` — `# Safety` section on every
  `unsafe fn`.
- `#![warn(clippy::doc_markdown)]` — flags un-backticked `CamelCase`
  identifiers in prose. The lint's heuristic ignores pure all-caps
  acronyms (`GPU`, `JSON`, `URL` — even bare in prose), so they don't
  need allow-listing; only `CamelCase` brand names / proper nouns / type
  names that you genuinely want to write as plain prose would. **Project
  type names, third-party type names, and build-config tokens (like
  `no_std`) must be backticked inline at every prose mention** — never
  allow-listed. The workspace currently ships **no `clippy.toml`**: the
  default-allow behaviour is sufficient. If a future genuine
  non-code-token false-positive appears, add a workspace-root
  `clippy.toml` with the narrowest possible `doc-valid-idents` array.
  Clippy's default list covers a wide range (storage units, frequencies,
  Apple/Microsoft frameworks, the `OpenGL` family, JS langs, OSes,
  `IPv4`/`IPv6`, `OAuth`, `NaN`, `CamelCase`, etc.); see the [clippy
  lint config docs](https://doc.rust-lang.org/clippy/lint_configuration.html#doc-valid-idents)
  for the authoritative list.

CI runs `cargo clippy -- -D warnings`, so the four `warn`-level lints are
hard errors in practice.

## Behavioural enforcement (what lints cannot check)

Lints cannot verify:

- Summary-line tense (third-person present indicative).
- Section order.
- `# Parameters` content quality (the bullets exist, but do they describe
  units / ranges / ownership semantics?).
- Trait-impl exemption boundaries (clippy may flag a `Result`-returning
  trait-impl method for missing `# Errors` even though the trait
  definition documents it; reviewer judgement applies).

These are checked by the `code-review` skill
(`.claude/skills/code-review/SKILL.md`) and the `review-findings` /
`self-review` agents on every PR. Both agents read this file as their
reference.
