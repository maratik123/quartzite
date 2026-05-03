# Design: Replace PropertyFlags bool-struct with enumflags2 bitfield

**Issue:** #69
**Date:** 2026-05-03

## Approach

Replace the hand-rolled `PropertyFlags` bool-struct with a `#[bitflags] #[repr(u8)] enum PropertyFlag`
backed by `enumflags2::BitFlags<PropertyFlag>`. This is a clean breaking change accepted by the spec.

### Chosen solution

1. **`PropertyFlag` enum** — defined in `quartzite-core/src/meta.rs` alongside all other metadata
   types. No new file needed; the module is small and cohesive. The enum carries `#[bitflags]` and
   `#[repr(u8)]`, and the existing `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` cluster is replaced
   by what `enumflags2` requires (the macro emits `Copy + Clone`; `Debug`, `PartialEq`, `Eq` come from
   the derive on the enum itself).

2. **`PropertyFlags` type alias** — `pub type PropertyFlags = BitFlags<PropertyFlag>`. The alias keeps
   all downstream call-sites (`PropertyMeta::new(…, PropertyFlags::read_write())`) compiling without
   change, because the named constructors (`none`, `read_write`, `read_only`) are moved to
   `impl PropertyFlag` returning `BitFlags<Self>` — the same name, same call syntax via the alias.

3. **`const fn` constructors** — `enumflags2` exposes `make_bitflags!(T::{A | B})` which is explicitly
   documented as `const`-compatible and expands to `from_bits_unchecked_c` with `CONST_TOKEN`. The
   macro body uses `let mut n = 0; n |= flag as Numeric;` inside a block — both `let mut` and `|=` in
   a const context have been stable since Rust 1.57, well below the project MSRV of 1.95. All three
   constructors (`none`, `read_write`, `read_only`) use `make_bitflags!` and are declared `pub const
   fn`. This satisfies the constraint that `PropertyMeta::new` is a `const fn` used in `static`
   initialisers.

   Concretely:
   ```rust
   pub const fn none() -> BitFlags<Self> {
       BitFlags::EMPTY   // make_bitflags! does not accept an empty brace list
   }
   pub const fn read_write() -> BitFlags<Self> {
       enumflags2::make_bitflags!(PropertyFlag::{Readable | Writable | Stored | Designable})
   }
   pub const fn read_only() -> BitFlags<Self> {
       enumflags2::make_bitflags!(PropertyFlag::{Readable | Stored | Designable | Constant})
   }
   ```
   `BitFlags::EMPTY` is a `const` associated constant — usable in `const fn` contexts. The
   `make_bitflags!` macro does not accept an empty brace list for `none()`, so `BitFlags::EMPTY` is
   used directly.

4. **`Default` for `PropertyFlags`** — AC4 requires `Default for PropertyFlags` to return
   `read_write()`. `enumflags2` already provides `impl<T: BitFlag> Default for BitFlags<T>`; the
   orphan rule does not apply. The `#[bitflags]` proc-macro supports a `default = ...` argument that
   sets the value returned by `Default::default()` for `BitFlags<T>`.

   The enum declaration becomes:
   ```rust
   #[bitflags(default = Readable | Writable | Stored | Designable)]
   #[repr(u8)]
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum PropertyFlag { … }
   ```
   `BitFlags::<PropertyFlag>::default()` then returns the same set as `PropertyFlag::read_write()`.
   No manual `impl Default` block is needed. No newtype is introduced.

5. **Dependency declaration** — `enumflags2 = "0.7.12"` added to `quartzite-core/Cargo.toml` as a
   direct crate-level dependency (not workspace-level), since only `quartzite-core` uses it.
   `no_std` is supported natively (`#![cfg_attr(all(not(test), not(feature = "std")), no_std)]` inside
   `enumflags2`); the `std` feature in `enumflags2` only adds `Error` impl on `FromBitsError` and is
   not needed. No `default-features = false` override is required because `enumflags2` has no default
   features that pull in `std` unconditionally.

6. **Proc-macro codegen** — `quartzite-macros/src/object/codegen.rs` currently emits a struct literal:
   ```rust
   ::quartzite::core::PropertyFlags { readable: #readable, writable: #writable, … }
   ```
   This is replaced by a `make_bitflags!`-style conditional union built from the same `bool` variables.
   Because `make_bitflags!` requires variant names statically (not runtime bools), the codegen must
   emit an explicit bit-union expression:

   ```rust
   {
       let mut __f = ::quartzite::core::PropertyFlag::none();
       if #readable  { __f |= ::quartzite::core::PropertyFlag::Readable;  }
       if #writable  { __f |= ::quartzite::core::PropertyFlag::Writable;  }
       if #notify    { __f |= ::quartzite::core::PropertyFlag::Notify;    }
       if #stored    { __f |= ::quartzite::core::PropertyFlag::Stored;    }
       if #designable{ __f |= ::quartzite::core::PropertyFlag::Designable;}
       if #user      { __f |= ::quartzite::core::PropertyFlag::User;      }
       if #constant  { __f |= ::quartzite::core::PropertyFlag::Constant;  }
       __f
   }
   ```

   However, the booleans in the codegen are **compile-time literals** (each is a `bool` literal in the
   `quote!` expansion, computed by the proc-macro at macro-expansion time, not at the user's runtime).
   Therefore `make_bitflags!` **can** be used: the codegen constructs the flag set at Rust-compile time
   by pattern-matching the booleans and emitting the appropriate `make_bitflags!` call directly.

   Concretely, `emit_props_static` already computes `readable`, `writable`, etc. as Rust `bool` values
   inside the proc-macro. The codegen emits a `make_bitflags!` call whose variant list is built by
   filtering those booleans.

   **`make_bitflags!` path constraint:** The macro's grammar is `$enum:ident ::{…}` — it accepts
   only a bare identifier, not a qualified path. Emitting
   `make_bitflags!(::quartzite::core::PropertyFlag::{…})` does **not** compile. To work around this,
   the generated hidden module (`mod __quartzite_Foo { … }`) already emits `use` items for other
   quartzite types. The codegen must additionally emit
   `use ::quartzite::core::PropertyFlag;` inside that hidden module so the bare identifier
   `PropertyFlag` is in scope for `make_bitflags!(PropertyFlag::{…})`.

   ```rust
   // In emit_props_static (proc-macro Rust code, not generated code):
   let flag_variants: Vec<TokenStream> = [
       (readable,   quote!(Readable)),
       (writable,   quote!(Writable)),
       (notify,     quote!(Notify)),
       (stored,     quote!(Stored)),
       (designable, quote!(Designable)),
       (user,       quote!(User)),
       (constant,   quote!(Constant)),
   ]
   .into_iter()
   .filter_map(|(active, tok)| active.then_some(tok))
   .collect();

   // The hidden module (generated by `codegen`) must include:
   //   use ::quartzite::core::PropertyFlag;
   // so the bare ident is resolvable inside the module.

   // Then emit per-property:
   quote! {
       ::quartzite::core::PropertyMeta::new(
           #name,
           ::core::stringify!(#ty),
           ::quartzite::core::enumflags2::make_bitflags!(PropertyFlag::{#(#flag_variants)|*}),
       )
   }
   ```

   The `use ::quartzite::core::PropertyFlag;` import is emitted once at the top of the hidden module,
   alongside the existing use-imports. This makes `PropertyFlag` a bare ident in scope for every
   `make_bitflags!` call in that module.

   This produces a single `make_bitflags!` expression that is a `const` value, satisfying the
   `const fn PropertyMeta::new` requirement.

   > **Re-export path:** `quartzite-macros` does not depend on `enumflags2` directly and should not.
   > The macro is accessed as `::quartzite::core::enumflags2::make_bitflags!` — which resolves
   > because `quartzite-core` re-exports `enumflags2` (see item 8 below) and the facade re-exports
   > `quartzite-core` wholesale.

   Revised plan: add `#[doc(hidden)] pub use enumflags2;` to `quartzite-core/src/lib.rs` so that the
   path `::quartzite::core::enumflags2::make_bitflags!(...)` resolves in user crates, and emit
   `use ::quartzite::core::PropertyFlag;` inside the generated hidden module so `make_bitflags!` can
   reference the bare ident `PropertyFlag`.

7. **Field-access migration** — the only two call sites using `flags.field` notation are in
   `quartzite-core/src/meta.rs` tests. These change to `flags.contains(PropertyFlag::Readable)` etc.
   Doctest examples in `meta.rs` that use `f.readable` are updated to `f.contains(PropertyFlag::Readable)`.

8. **Re-export chain** — `quartzite-core/src/lib.rs` exports:
   ```rust
   pub use meta::{…, PropertyFlag, PropertyFlags, …};
   /// Re-exported solely for use by quartzite proc-macro generated code; not part of the public API.
   #[doc(hidden)]
   pub use enumflags2;
   ```
   Marking the re-export `#[doc(hidden)]` prevents it from appearing in `cargo doc` output while
   still being accessible by path from generated code. The facade `src/lib.rs` uses
   `pub use quartzite_core::*` (via `pub mod core`), so `quartzite::core::PropertyFlag`,
   `quartzite::core::PropertyFlags`, and `quartzite::core::enumflags2` all become available
   automatically.

### Rejected alternatives

- **newtype wrapper** — adds boilerplate `Deref`, `From`, `Into` impls; deferred per spec.
- **separate `property_flag.rs` file** — unnecessary; the type is small and belongs with its peers in `meta.rs`.
- **workspace-level `enumflags2` dep** — only `quartzite-core` needs it; crate-level is sufficient.
- **runtime bit-union in codegen** (`if` chain) — unnecessarily produces non-`const` code; using `make_bitflags!` keeps the static array `const`.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `enumflags2 = "0.7.12"` to `quartzite-core/Cargo.toml`; run `cargo build -p quartzite-core` to update `Cargo.lock` | `quartzite-core/Cargo.toml`, `Cargo.lock` | — |
| 2 | Define `#[bitflags] #[repr(u8)] pub enum PropertyFlag` with 7 variants in `meta.rs`; add `pub type PropertyFlags = BitFlags<PropertyFlag>` type alias; remove old struct | `quartzite-core/src/meta.rs` | 1 |
| 3 | Add `impl PropertyFlag` with `const fn none()`, `const fn read_write()`, `const fn read_only()` using `make_bitflags!` / `BitFlags::EMPTY`; drop old `impl PropertyFlags` block and old `impl Default for PropertyFlags` block (`Default` is now provided by `enumflags2` via `#[bitflags(default = …)]`) | `quartzite-core/src/meta.rs` | 2 |
| 4 | Re-export `PropertyFlag` and `enumflags2` from `quartzite-core/src/lib.rs` | `quartzite-core/src/lib.rs` | 2 |
| 5 | Update in-file doctests in `meta.rs`: replace `f.readable` / `!f.writable` / `f.constant` with `f.contains(PropertyFlag::Readable)` etc.; update use-paths in doctests | `quartzite-core/src/meta.rs` | 3 |
| 6 | Update unit tests in `meta.rs` `#[cfg(test)]` block: replace `prop.flags.readable` etc. with `prop.flags.contains(PropertyFlag::Readable)` | `quartzite-core/src/meta.rs` | 3 |
| 7 | Update proc-macro codegen: replace struct-literal emission with `make_bitflags!`-based emission using filtered variant list; add `use ::quartzite::core::PropertyFlag;` to the hidden-module token stream; rewrite the 4 flag-related unit tests in `codegen.rs` that assert old struct-literal fragments | `quartzite-macros/src/object/codegen.rs` | 4 |
| 8 | Run full test suite (`cargo test`), clippy (`cargo clippy -- -D warnings`), doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`), and no_std path (`cargo build -p quartzite --no-default-features`) | — | 1–7 |

## Risks

- **`make_bitflags!` empty set syntax**: `make_bitflags!(PropertyFlag::{})` — the macro does not
  accept an empty brace list for `none()`. Mitigation: confirmed — use `BitFlags::EMPTY` (a `const`
  associated constant) directly for `none()`.
- **`make_bitflags!` accepts only bare idents**: the macro grammar is `$enum:ident ::{…}`; qualified
  paths like `::quartzite::core::PropertyFlag` do not parse. Mitigation: emit
  `use ::quartzite::core::PropertyFlag;` inside the generated hidden module so the bare ident is in
  scope (see codegen section above).
- **`make_bitflags!` const-safety at MSRV 1.95**: The macro expands to `let mut n = 0; n |= flag as
  Numeric;` inside a block. Both `let mut` in const and `|=` on integers in const have been stable
  since Rust 1.57 — well below the project MSRV of 1.95. No risk.
- **`Default` for `PropertyFlags`**: `enumflags2` provides `impl<T: BitFlag> Default for BitFlags<T>`
  driven by `#[bitflags(default = …)]`. No orphan-rule issue. Mitigation: use
  `#[bitflags(default = Readable | Writable | Stored | Designable)]` on the enum declaration.
- **`make_bitflags!` path in generated code**: the macro must be reachable from user crates via
  `::quartzite::core::enumflags2::make_bitflags!(...)`. Mitigation: add
  `#[doc(hidden)] pub use enumflags2;` to `quartzite-core/src/lib.rs` and verify the path resolves
  in integration tests.
- **`PropertyMeta::new` const stability**: it is already `const fn`; `BitFlags` values produced by
  `make_bitflags!` are `const`, so the static array in proc-macro output remains const. Low risk.
- **`no_std` compatibility**: `enumflags2` supports `no_std` without any feature flag. Verify with
  `cargo build -p quartzite --no-default-features` in task 8.
- **`#[bitflags]` attribute requires `enumflags2_derive`**: the `enumflags2` crate re-exports its
  derive macro, so adding only `enumflags2` to `[dependencies]` is sufficient — no separate
  `enumflags2_derive` needed.

## Test Design

### Task 3 & 6 — Unit tests in `quartzite-core/src/meta.rs` `#[cfg(test)]`

**Location:** `quartzite-core/src/meta.rs`, existing `mod tests`

**Modified tests (field-access → `contains`):**
- `property_meta_flags_readable_writable`: `prop.flags.readable` → `prop.flags.contains(PropertyFlag::Readable)`, etc.
- `property_meta_flags_read_only_constant`: same pattern.

**New tests to add:**
- `property_flag_none_is_empty`: `PropertyFlag::none().is_empty()` is `true`.
- `property_flag_read_write_contains`: asserts `Readable`, `Writable`, `Stored`, `Designable` are
  set; `Notify`, `User`, `Constant` are not.
- `property_flag_read_only_contains`: asserts `Readable`, `Stored`, `Designable`, `Constant` are
  set; `Writable`, `Notify`, `User` are not.
- `property_flags_default_is_read_write`: asserts
  `PropertyFlags::default() == PropertyFlag::read_write()`. `Default` is provided by `enumflags2`
  via `#[bitflags(default = Readable | Writable | Stored | Designable)]` — this test must pass.
- `property_flag_const_constructors`: `const` items in the test module using the constructors to
  verify they are usable in `const` contexts (compiler-checked, no runtime assertion needed).

### Task 7 — Proc-macro codegen unit tests (existing tests that must be rewritten)

**Location:** `quartzite-macros/src/object/codegen.rs` `#[cfg(test)]` module

The current test suite contains 4 tests that call `codegen(ir).to_string()` and assert string
fragments that match the old struct-literal syntax. After the change to `make_bitflags!`, these
assertions fail. They **must** be rewritten as part of Task 7 — not treated as a separate task.

**Tests to rewrite (all in `mod tests` of `codegen.rs`):**

| Test name | Old assertions to remove | New assertions to add |
|---|---|---|
| `writable_prop_flags` | `"readable : true"`, `"writable : true"`, `"notify : false"` | `"make_bitflags"`, `"Readable"`, `"Writable"` present; `"Notify"` absent in flags expression |
| `read_only_prop_has_writable_false` | `"writable : false"` | `"make_bitflags"` present; `"Writable"` absent in the flags tokens for that property |
| `constant_prop_has_writable_false` | `"writable : false"`, `"constant : true"` | `"make_bitflags"` present; `"Constant"` present; `"Writable"` absent |
| `notify_prop_has_notify_true` | `"notify : true"` | `"make_bitflags"` present; `"Notify"` present |

**Strategy note:** `to_string()` on a `TokenStream` produces space-separated tokens. For the new
codegen, a writable property expands to a fragment like:
```
make_bitflags ! (PropertyFlag ::{ Readable | Writable | Stored | Designable })
```
Assertions should check:
- `out.contains("make_bitflags")` — macro was emitted.
- `out.contains("Readable")` — Readable flag present when expected.
- `out.contains("Writable")` — presence or absence depending on property kind.
- `out.contains("Notify")` — presence or absence.
- `out.contains("Constant")` — presence or absence.
- `out.contains("use :: quartzite :: core :: PropertyFlag")` — the use-import is emitted inside the
  hidden module, making the bare ident available for `make_bitflags!`.

**All other tests in `codegen.rs` are unaffected** by this change (they test signals, read/write
dispatch, wrappers, etc.) and require no modification.

### Task 7 — Proc-macro codegen integration tests

**Location:** `quartzite-macros/tests/object.rs` or `quartzite-macros/tests/object_impl.rs`

**Entry point:** existing `#[derive(Object)]` / `#[object_impl]` usage that exercises property codegen.

**Scenarios:**
- Inspect `META.properties[n].flags.contains(PropertyFlag::Readable)` for a known property with
  `#[prop]` — happy path.
- A `#[prop(constant)]` property has `Constant` set and `Writable` unset.
- A `#[prop(notify = signal)]` property has `Notify` set.
- These scenarios already exist implicitly if the compilation succeeds; add explicit `assert!` calls
  where the tests currently do not inspect flags.

## Open questions

- None.
