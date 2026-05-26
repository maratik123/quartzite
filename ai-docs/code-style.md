# Code Style

This document is the canonical reference for Rust source-style rules
across the workspace. Every Rust source file in every workspace crate
(`quartzite`, `quartzite-core`, `quartzite-events`, `quartzite-event-types`,
`quartzite-geometry`, `quartzite-macros`, `quartzite-runtime`) must
conform.

[`AGENTS.md`](../AGENTS.md) carries a bulleted index summarising each
cluster below; this file is the canonical source of the rule prose.
Documentation-style rules (summary tense, section order, intra-doc
links, etc.) live in [`doc-convention.md`](doc-convention.md), not here.

## Scope

- **Applies to:** every Rust source file under `src/`, `tests/`, and
  `examples/` across all workspace crates.
- **Does NOT apply to:**
    - Build scripts (`build.rs`) — see crate-level conventions.
    - Generated / codegen output (proc-macro emission targets) — the
      generating code follows these rules; the generated output is
      whatever the generating code produces.
    - `Cargo.toml`, `clippy.toml`, `rustfmt.toml`, and other
      configuration — covered by AGENTS.md `## Dependency Versions` and
      tooling sections.

## Source files

- **All source files must be Rust (`.rs`).** No other languages in `src/`.
- Max line: 100 (rustfmt default).
- Let chains (`if let A = x && let B = y { ... }`) format via `cargo fmt`,
  never `rustfmt <file>` directly.

## Linter posture

- Strict clippy (enforced).
- **Workspace-wide lint policy** lives in the root `Cargo.toml`
  `[workspace.lints.rust]` + `[workspace.lints.rustdoc]` +
  `[workspace.lints.clippy]` tables; every member crate (13 leaves +
  the root `quartzite` package) opts in via `[lints] workspace = true`
  in its own `Cargo.toml`. The four group/lint enables in force are
  `clippy::pedantic` and `clippy::nursery` (both `warn`, `priority =
  -1` so specific `clippy::* = "allow"` entries override the group),
  plus `clippy::large_stack_frames` and `clippy::large_stack_arrays`
  (both `deny`, listed separately so each survives a future
  per-group rollback).
- **Size-aware thresholds** live in `clippy.toml` at the workspace
  root (`stack-size-threshold` / `array-size-threshold`). Clippy
  auto-discovers `clippy.toml` from the workspace root; per-crate
  `clippy.toml` is not used.
- **Allow-list discipline.** Every `clippy::* = "allow"` entry in
  `[workspace.lints.clippy]` MUST carry a one-line `#`-comment above
  it justifying the allow — same rule as the in-source no-blanket-
  `#[allow]`-without-justification axiom. Where the allow overlaps a
  project doc (e.g., `missing_panics_doc` vs `panic-index.md`), the
  comment cross-references the relevant doc.

## Rust idioms

- Prefer Rust idioms over literal ports from other languages. When in
  doubt, ask. **Never cite other GUI/UI frameworks (Qt, GTK, WinForms,
  SwiftUI, etc.) as justification for design choices** in specs,
  designs, commit messages, or PR bodies. Justify decisions from Rust
  idioms, the crate ecosystem, and explicit reasoning about the
  problem.
- Let chains (`if let A = x && let B = y { ... }`) are valid in this
  codebase (edition 2024). Do not avoid them.
- **Comparison helpers.** Use `.min()`, `.max()`, `.clamp()`,
  `Option::or` / `Option::filter`, etc. instead of explicit `if`/`match`
  for simple value selection. Applies to all numeric types and any
  `Ord`/`PartialOrd` types. Reach for the stdlib method first; fall back
  to branching only when the comparison logic is genuinely non-trivial.

## Magic numbers

Numeric literals that carry semantic meaning belong in named
`const` values, not inline at the use site. The name documents the
intent; the literal is an implementation detail.

- **Applies to:** colour-component values (`Color::new(0.94, 0.94,
  0.94, 1.0)` → `const NEUTRAL_GREY: Color = …`), sizes, timeouts,
  cache limits, magic offsets, retry counts — anything where the
  reader has to ask "why this number?".
- **Does not apply to:** self-evident constants (`0`, `1`, `-1`,
  `2` for "next/prev", powers-of-two array bounds in obvious
  doubling contexts), loop indices, and test fixtures where the
  exact value carries no meaning beyond "some valid input".
- **Placement:** module-private `const FOO: T = …;` at the top of
  the module (after `use` statements). Public surface only when the
  constant is genuinely part of the public API (e.g. a default
  exposed for callers to inspect).
- **Naming:** `SCREAMING_SNAKE_CASE` per Rust convention; the name
  must describe the *role* (`PALETTE_LIGHT_BACKGROUND`), not the
  shape (`COLOR_94`). If the role is unclear, the constant is
  premature — wait until the use site clarifies it.

History — this rule has fired twice on `Palette::default()` /
`palette.rs` (2026-05-08 in `Palette::default()`, again 2026-05-13
on `palette.rs:119`'s `Color::new(0.0, 0.5, 1.0, 1.0)` literal —
the second occurrence reached a human reviewer post-push because
`/bugfix` Step 6 lacked the self-review gate that would have caught
it). Both `self-review` and `review-findings` checklists list this
as a `minor` finding so the gate fires pre-push.

## Library safety idioms

Concrete forms of the "non-panicking APIs for libraries" rule (see
[`AGENTS.md` § *API Naming*](../AGENTS.md#api-naming)):

- **Mutex / RwLock — workspace default is `parking_lot`.** Use
  `parking_lot::Mutex` and `parking_lot::RwLock` for all in-tree
  locks. Their `.lock()` / `.read()` / `.write()` methods are
  infallible (no `PoisonError`, no `Result` wrapper) and return the
  guard directly — call sites are plain `let g = m.lock();` with no
  `.unwrap()` / `.expect()` / `.ok()?` / `.unwrap_or_else(...)`
  ceremony. `parking_lot` does not poison on panic, so the recovery
  idioms below are unnecessary in current code.
  - _Footnote — rare FFI-imposed `std::sync::*Lock` retainees._ If a
    `std::sync::Mutex` / `std::sync::RwLock` is unavoidable (e.g. it
    appears in a public API exported across an FFI boundary that
    requires the `std` type), prefer
    `mutex.lock().ok()?` in `Option`/`Result`-returning fns or
    `.unwrap_or_else(|e| e.into_inner())` to recover the inner value
    on poison. Reserve `.lock().unwrap()` for cases where poisoning
    genuinely indicates an unrecoverable broken global invariant —
    and even then prefer `.expect("reason")`. No such retainee
    currently exists in-tree.
- **Prefer safe primitives over raw pointers.** If a `OnceLock` / `Arc`
  / `Weak` already in scope holds the value, an `AtomicBool` flag is
  enough to track liveness — do not reach for `AtomicPtr` + `unsafe`.
  Reserve `unsafe` for cases where no safe construct expresses the
  semantic.

## Documentation

- The workspace declares `missing_docs = "deny"` and
  `clippy::undocumented_unsafe_blocks = "deny"` in
  `[workspace.lints.*]`; every crate opts in via
  `[lints] workspace = true` in its own `Cargo.toml`.
- Every public item must have at least a one-line `///` doc comment.
- Every new public item with only a single-line doc must include a
  `# Examples` block. Proc-macro examples use `no_run`; runtime items
  needing an event loop use `no_run`; pure library types use compiling
  doctests.
- **Documentation Conventions:** See
  [`doc-convention.md`](doc-convention.md) for the canonical workspace
  doc style. Mandatory rules in brief: third-person-present-indicative
  summary line; `# Parameters` on every public fn with ≥1 arg; strict
  section order (`Parameters → Returns → Type parameters → Lifetimes →
  Errors → Panics → Safety → Examples → See also`); `# Errors` /
  `# Panics` / `# Safety` whenever applicable (clippy-enforced);
  intra-doc links over inline URLs; American English. Methods inside
  `impl Trait for Type {}` blocks are exempt — the trait definition
  documents the contract.

## Error types

Use `thiserror` for any new error enum/struct in this workspace — it
eliminates boilerplate `Display` / `std::error::Error` impls and keeps
error definitions concise. Hand-rolled `Display` / `Error` impls are
reserved for cases where `thiserror`'s derive cannot express the
required behaviour.

## Enum repr

`#[repr(...)]` on enums is required in exactly two cases:

1. **`enumflags2::bitflags` contract** — the macro requires `#[repr(uN)]`
   on its target enum to guarantee the bitfield arithmetic is sound.
2. **External numeric spec carried in discriminants** — when an enum's
   discriminants are specified by an external standard and the raw integer
   type matters (e.g. OpenType `usWeightClass` encoded as `u16` for
   `FontWeight`).

In all other cases, `#[repr]` MUST NOT be added. Decorative annotations
(e.g. `#[repr(i64)]` to "match" a wire format that the macro or runtime
handles automatically) add noise without correctness value and are
forbidden.

## Tracing

Functions that make a meaningful, traceable change to application state
should open with a `*_span!` guard
(`let _span = …_span!(…).entered();`) wrapping the entire function
body. Threshold: "would someone debugging a live system want to see
this in a log?" Skip trivial getters/setters.

- Use **`debug_span!`** for significant mutations — object lifecycle
  (insert, destroy, reparent), timer lifecycle (start/stop), event
  loop/pool lifecycle.
- Use **`trace_span!`** for supplementary or lower-significance
  mutations — name changes, connection bookkeeping, posting closures.
- High-frequency paths (e.g. signal emit or event loops) must gate the
  span behind the `verbose-tracing` cargo feature rather than always-on.
- When adding a sibling to an existing traced function, apply the same
  treatment so the log story stays consistent.

## `#[inline]` and the `_Simple._` doc tag

Mark every **simple** function so callers and reviewers can recognise
it as inline-equivalent.

> **AXIOM — `#[inline]` and `_Simple._` are MUTUALLY EXCLUSIVE.**
> Never put both on the same `fn`. Pick exactly **one** marker based on
> the fn's shape (table below). Co-occurrence is always a bug — the
> tag form is "this fn would carry `#[inline]` if it could, but its
> shape makes `#[inline]` a no-op," so adding both says the opposite
> things at once.
>
> | If you see... | Action |
> |---|---|
> | `#[inline]` AND `_Simple._` (any form) on the same fn | **REMOVE** `_Simple._`, keep `#[inline]` |
> | `#[inline]` on a generic fn / `impl<T> Trait for Foo<T>` method | **REPLACE** with the appropriate `_Simple._` form |
> | `_Simple._` on a concrete fn / `impl Trait for ConcreteFoo` method | **REPLACE** with `#[inline]` |
>
> The `Carve-out` and `decision tree` below choose **which** of the two
> applies for a given shape — they never select **both**.

**"Simple" (recursive definition):**
- No branches or loops in the body, AND
- At most one call to a **non-simple** function.

Calls to other simple functions are "free" — apply the test
transitively to each callee. After inlining, the chain collapses, so
the binary cost is bounded regardless of how many simple-into-simple
delegations the source contains.

**Three visible markers, mutually exclusive by fn shape:**

| Function shape | Marker | Why this marker |
|---|---|---|
| Concrete (no own type parameters AND `Self` is concrete — i.e. the surrounding `impl` block introduces no generics) | **`#[inline]` attribute** above the `fn` line | Needed for cross-crate inlining; visible in source. |
| Generic — either the fn declares its own `<T>` / `<F: ...>` / `impl Trait` parameter, **or** it sits inside an `impl<T> ...` / `impl<T> Trait for Foo<T> { … }` block where `Self` is parametrised by the impl block's generics | **`_Simple._` doc tag** under the summary (see [`doc-convention.md`](doc-convention.md)) | Body is monomorphized per concrete `T`, so the compiler already has it cross-crate and `#[inline]` is redundant; the tag is the human signal that calls into this fn count as "free" in the recursive budget rule. |
| Trait method declaration whose every conforming impl is required to be simple (typically because the impl is generated by a derive / proc-macro in this workspace, or because the trait's contract makes only simple impls valid) | **`_Simple._` doc tag** on the trait method declaration | Trait declarations carry no body; the tag publishes the contract so callers, reviewers, and hand-written-impl authors all see the same signal. A hand-written impl whose body is non-simple violates the trait contract. |

Generic simple fns and tagged trait methods get the doc tag *instead
of* `#[inline]`, not in addition to it — pick one based on shape.

**Carve-out: default trait methods inside a `pub trait` body** (e.g.
`ObjectExt::{id, name, is_on_current_thread}`) stay in the **concrete**
row when their own bodies declare no type parameters. The body lives
in a single source location (the trait declaration); only `Self`
varies. Use `#[inline]`. This carve-out does **not** apply to methods
inside an `impl<T> Trait for Foo<T>` block — those go to the **generic**
row, even when the method itself declares no extra type parameters,
because `Self = Foo<T>` is parametrised by the impl block's generics
and the body is monomorphized per concrete `T`.

**Where to place `_Simple._` for methods inside an `impl<T> Trait for
Foo<T>` block.** Add the line `// _Simple._` (regular line comment,
**not** a `///` doc comment) directly above the `fn` keyword inside
the impl block. Rustdoc treats a `///` line on a trait-impl method as
overriding the trait's inherited docstring — `/// _Simple._` on
`ObjectRef<T>::clone` would replace `Clone::clone`'s actual docstring
with just `_Simple._` on the rustdoc page. A regular `//` comment is
invisible to rustdoc, stays a human-only marker visible in source, and
preserves the trait-inherited docs. Audit grep `rg '_Simple\._'`
matches both forms. Example: `Signal<Args>::default`,
`ObjectRef<T>::clone`, `WeakRef<T>::eq`.

**Marker-form decision tree:**

| Position | Marker form |
|---|---|
| Concrete fn (free fn / inherent method on a concrete type, no own type params) | `#[inline]` attribute |
| Method inside `impl Trait for ConcreteFoo` block — concrete impl on a concrete struct, impl block introduces **no** generics | `#[inline]` attribute (concrete-row equivalent) — load-bearing for cross-crate inlining without LTO; `// _Simple._` is *not* a substitute because it has no codegen effect |
| Generic free fn / inherent generic method (`impl<T> Foo<T> { fn ... }`) | `/// _Simple._` doc line |
| Trait method declaration (default method or method decl in a `pub trait` body) | `/// _Simple._` doc line — becomes part of the trait's docs and is inherited by all impls |
| Method inside an `impl<T> Trait for Foo<T>` block (impl block introduces generics; inherits docs from the trait) | `// _Simple._` line comment — `#[inline]` is redundant (monomorphization already exports MIR per concrete `T`); `///` would override the inherited rustdoc |

**Why concrete trait-impl methods get `#[inline]`, not `// _Simple._`:**
the comment form is just a human marker with zero codegen or rustdoc
effect. For a method inside `impl Trait for ConcreteFoo`, the body is
**not** monomorphized into downstream crates (the impl is a single
non-generic symbol), so without `#[inline]` its MIR is not exported
across the crate boundary — downstream callers get a real function
call. `#[inline]` is the only marker that does the cross-crate
inlining work in this position. The trait-declaration tag (when
present) covers the human signal via rustdoc inheritance, so no
additional `// _Simple._` is needed on the impl method; if the trait
declaration is *not* tagged, neither marker is appropriate on the
impl method (consider whether the trait contract should be tagged
instead).

**Decision rule for tagging a trait method:** tag only when *every*
conforming impl is required to be simple. If some valid impls are
non-simple (different by-value shape, branches, loops), do **not** tag
— callers cannot rely on it.

**Trait-tag examples in this codebase:** `AsObject::{object_base,
object_base_mut, as_any, as_any_mut}` (codegen always emits a single
field/`self` access); `Object::{meta_object, connect_signal}` (simple
by codegen contract). `Object::{read_property, write_property,
invoke_method}` are **not** tagged — they contain `match` branches and
are non-simple.

**Typical targets** for `#[inline]` (concrete row): field getters
(`self.field`), trivial wrappers (`.as_deref()`, single delegation
call), `Default::default()` that calls `Self::new()`, `const fn`
struct-literal constructors, wrappers whose body is one call into
another simple fn (e.g. `ObjectExt::id` → `self.object_base().id()`).

**Codegen mirroring:** emit `#[inline]` before each generated simple
fn whose position falls in the concrete row of the decision tree —
this includes both free fns / inherent methods on concrete types
**and** methods inside `impl Trait for ConcreteFoo` blocks (concrete
impl on a concrete struct). Emit a `/// _Simple._` doc line before
each generated simple **generic** fn and before each generated trait
method declaration whose conforming impls are always simple. For
generated methods inside an `impl<T> Trait for Foo<T>` block the
trait declaration's `/// _Simple._` already publishes the contract
via rustdoc inheritance, so no additional marker on the impl method
is needed (and `#[inline]` would be redundant — monomorphization
already exports MIR per concrete `T`). Codegen that emits trait-impl
methods must therefore branch on whether the user struct introduces
type/const params: emit `#[inline]` for concrete-struct emission,
emit nothing for generic-struct emission. `// _Simple._` cannot be
emitted via `quote!` (Rust strips comments before token-stream
parsing), and is unnecessary on the concrete-struct branch (where
`#[inline]` is the canonical marker) and on the generic-struct
branch (where rustdoc inheritance covers the signal).

**Marker maintenance.** When an edit makes a previously-simple fn
non-simple (gains branches or loops, or > 1 call to a non-simple fn),
strip its `#[inline]` attribute or `_Simple._` doc tag in the same
edit — leaving a stale marker turns it into a lie (and a `_Simple._`
tag is actively misleading, since callers count calls into the fn as
"free" in the recursive budget). Then `rg <fn-name>` for callers and
re-evaluate each: every caller whose simplicity depended on this
callee being free must be re-tested by the recursive definition and
de-marked if it no longer qualifies. Cascade until quiescent. The
propagation cost is bounded — the simple class is small in practice
(leaves and trivial wrappers) and the cascade rarely goes more than
2 levels deep. For trait methods: if a new conforming impl
(hand-written or codegen) cannot be simple, strip `_Simple._` from the
**trait declaration**, not the impl — the contract is what changed.

## Generic-fn split for binary size

When a public function takes a conversion-style generic parameter
(`impl Into<T>`, `impl AsRef<T>`, `impl ToString`, …) and the body is
more than ~3 lines, **place the body in a nested non-generic
`fn inner(...)` defined inside the outer fn body**, and let the
generic outer fn perform only the conversion. The outer fn carries
the `_Simple._` doc tag (it is generic and simple by the recursive
definition). Only the trivial conversion shell is monomorphized; the
body ships once.

**Do not name the helper `<outer>_inner` and do not place it as a
sibling impl method.** Nesting keeps the helper out of the type's
namespace, makes it impossible to misuse from elsewhere, and matches
the `std` idiom (`Path::new`, `PathBuf::from`, `OsStr::new`, …).
Promote the helper to a sibling private fn **only if** a second caller
actually needs it.

**Does not apply** when the type parameter is used throughout the body
(genuine generic logic — e.g. `F: FnOnce(...)` invoked inside the
body): there is no non-generic remainder to extract.

Pattern:
```rust
/// Renames the object `id` to `new_name`, updating the name index.
///
/// _Simple._
///
/// # Examples
/// ...
pub fn rename(&mut self, id: ObjectId, new_name: impl Into<String>) {
    fn inner(this: &mut ObjectTree, id: ObjectId, new_name: String) {
        // body
    }
    inner(self, id, new_name.into())
}
```

A nested `fn` cannot capture `self`; pass it explicitly as
`this: &mut Self` (or use a closure when capture matters; the closure
form is acceptable but less greppable).

**If the inner ends up simple, unwrap it.** Run the recursive
simplicity test on every `fn inner` body. If the inner qualifies as
simple (no branches/loops, ≤ 1 call to a non-simple fn), it would be
`#[inline]` per the concrete-row rule — and the compiler will inline
it back into the (per-`T`-monomorphized) outer, **defeating the
split**: the body ends up duplicated per concrete `T` anyway, just
with extra source indirection. **Delete the wrapper and put the body
directly in the outer fn.** Then re-evaluate the outer's `_Simple._`
marker per the marker-maintenance rule: unwrapping moves the inner's
non-simple calls (≤ 1 of them) into the outer, which can flip the
outer from simple to non-simple — strip the tag if so. *Caveat:* if
unwrapping pushes the outer body past ~3 lines, the split rule
re-applies — but a simple `inner` is by definition ≤ ~2 lines, so
this is automatic in practice. **Therefore, an `#[inline]` simple
`fn inner` inside a generic-fn split is a code smell**: either the
split was unnecessary (unwrap), or the inner is mis-marked.

## File size

Target **200–400 lines per `.rs` file excluding `#[cfg(test)]`**
(readability sweet spot — fits in mental RAM, supports cohesive
grouping of a struct + its `impl` blocks + related errors).

- **Soft limit:** 500 lines excl. tests / 800 incl. tests. Trigger a
  split-by-responsibility check (e.g. `models.rs` / `db.rs` /
  `handlers.rs`) — do **not** split mechanically by line count.
- **Hard limit:** 1000 lines excl. tests / 1500 incl. tests. Refactor
  before merge unless an exemption applies.
- **Exemptions:** auto-generated / codegen output (build scripts,
  proc-macro emission targets); a single state machine or `match`
  where splitting would obscure the control flow; `macro_rules!`
  definitions.
- **Counter-rule — do not over-split.** One-struct-per-file (Java /
  C# habit) is not Rust idiom and bloats the `mod` tree. Prefer one
  cohesive 300-line file over three 100-line fragments.
- **Per-function:** Clippy's `too_many_lines` (>100) is the canonical
  fn-level signal — keep functions under it. Small functions
  naturally yield small files.

## Lints that mechanically enforce parts of this convention

CI runs `cargo clippy --all-targets -- -D warnings`, so every clippy
lint listed below is a hard error in practice.

- `missing_docs = "deny"` — every public item has at least a one-line
  doc. (Declared at workspace level in `[workspace.lints.rust]`; each
  crate opts in via `[lints] workspace = true`.) Owned by [Documentation](#documentation).
- `clippy::undocumented_unsafe_blocks = "deny"` — every `unsafe`
  block carries a `// SAFETY:` comment. (Declared at workspace level
  in `[workspace.lints.clippy]`; each crate opts in via
  `[lints] workspace = true`.) Owned by [Documentation](#documentation).
- `clippy::missing_errors_doc` — `# Errors` section on every
  `Result`-returning public fn. Primary owner is
  [`doc-convention.md`](doc-convention.md); cross-referenced from
  [Documentation](#documentation).
- `clippy::missing_panics_doc` — `# Panics` section on every fn that
  can panic. Primary owner [`doc-convention.md`](doc-convention.md).
- `clippy::missing_safety_doc` — `# Safety` section on every
  `unsafe fn`. Primary owner [`doc-convention.md`](doc-convention.md).
- `clippy::doc_markdown` — flags un-backticked `CamelCase` identifiers
  in prose. Primary owner [`doc-convention.md`](doc-convention.md).
- `clippy::too_many_lines` (>100) — canonical fn-level size signal.
  Owned by [File size](#file-size).
- `-D warnings` posture — every clippy warning is an error. Owned by
  [Linter posture](#linter-posture).
- `cargo fmt -- --check` (rustfmt enforcement) — line length,
  whitespace, let-chain layout. Owned by [Source files](#source-files).

## Annotated items

Fields and methods carrying any of the workspace proc-macro attributes
(`#[property]`, `#[signal]`, `#[slot]`, `#[invoke]`, `#[base]`, `#[mixin]`,
`#[widget_children]`) are **annotated items**. The visibility-lift rule:

- Every annotated item is `pub` by default.
- Exception: the item may remain private if an immediately-preceding
  `// why-private: <reason>` comment defends the exception. Acceptable
  reasons: macro-codegen invariant the public surface must not expose;
  internal-only invariant the macro relies on. The comment must be on the
  line immediately before the attribute chain.
- The doc-comment requirement from [`doc-convention.md` § *Annotated
  items*](doc-convention.md#annotated-items) applies regardless of the
  visibility decision — both `pub` and private annotated items require
  `///` docs.

See [`doc-convention.md` § *Annotated items*](doc-convention.md#annotated-items)
for the tri-state diagnostic surface (`#[undocumented(allow|warn|deny)]`,
per-invocation and global scopes, default level = `warn`).

## Behavioural enforcement (what lints cannot check)

Lints cannot verify the rules below; reviewers (and the
`review-findings` / `self-review` agents) check them on every PR.

- **No GUI/UI-framework citations** as design justification (Qt, GTK,
  WinForms, SwiftUI, …). Owned by [Rust idioms](#rust-idioms).
- **Let-chain encouragement.** Reviewer should accept idiomatic let
  chains rather than rewriting to nested `if let`. Owned by
  [Rust idioms](#rust-idioms).
- **Comparison-helper preference** over explicit `if`/`match` for
  simple value selection. Owned by [Rust idioms](#rust-idioms).
- **Mutex `lock().unwrap()` justification.** Reserved for genuine
  unrecoverable broken global invariant; reason string must explain
  *why recovery is impossible*. Owned by
  [Library safety idioms](#library-safety-idioms).
- **Safe primitives over raw pointers.** Owned by
  [Library safety idioms](#library-safety-idioms).
- **`thiserror` vs hand-rolled `Display`/`Error` judgement.** Owned by
  [Error types](#error-types).
- **Span-significance threshold.** "Would someone debugging a live
  system want to see this in a log?" — reviewer judgement, not
  lintable. Owned by [Tracing](#tracing).
- **`#[inline]` / `_Simple._` marker correctness.** The recursive
  simplicity test, the three-marker decision tree (concrete → attribute,
  generic → `///`, trait-impl → `//`), the carve-out for default
  methods inside a `pub trait` body, and the codegen mirroring rules
  are reviewer-checked. Owned by
  [`#[inline]` and the `_Simple._` doc tag](#inline-and-the-_simple_-doc-tag).
- **Marker-maintenance cascade.** When an edit makes a previously-simple
  fn non-simple, strip the marker AND `rg <fn-name>` for callers and
  re-test each. Owned by
  [`#[inline]` and the `_Simple._` doc tag](#inline-and-the-_simple_-doc-tag).
- **`_Simple._` decision rule for trait methods.** Tag only when
  *every* conforming impl is required to be simple. Owned by
  [`#[inline]` and the `_Simple._` doc tag](#inline-and-the-_simple_-doc-tag).
- **Unwrap-simple-`inner` rule.** A simple `fn inner` inside a
  generic-fn split is dead weight; unwrap and re-evaluate the outer's
  `_Simple._`. Owned by
  [Generic-fn split for binary size](#generic-fn-split-for-binary-size).
- **File-size soft-limit responsibility split.** When a file crosses
  the 500/800 line soft limit, trigger split-by-responsibility — do
  **not** split mechanically by line count. Owned by
  [File size](#file-size).
- **No over-splitting (one-struct-per-file anti-pattern).** Owned by
  [File size](#file-size).
