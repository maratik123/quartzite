# Design: inline-simple-fns

**Issue:** user description
**Date:** 2026-05-02

## Approach

Add `#[inline]` to every target function listed in the spec. No logic changes —
pure annotation only. Each `#[inline]` goes on its own line immediately before
the `pub`/`fn` keyword (after any existing outer attributes such as `#[cfg_attr]`
or `#[allow(...)]`, but before `pub`).

`const fn` functions (`PropertyFlags::none/read_write/read_only`,
`PropertyMeta::new`, `ParamMeta::new`, `SignalMeta::new`, `MethodMeta::new`,
`EnumEntry::new`, `EnumMeta::new`, `MetaObject::new`) receive `#[inline]` just
like non-const fns — the attribute is legal on `const fn` and has effect for
non-const call sites.

No generic function is touched. Verification below.

For codegen changes: `#[inline]` is emitted inside `quote!` blocks as the token
`# [inline]` (proc-macro2 serialises each token separately with spaces). Tests
assert `out.contains("# [inline]")` to confirm the attribute appears in emitted
output, satisfying AC9.

### Generic / monomorphized functions confirmed absent from target list

| Function | Is generic? | In scope? |
|---|---|---|
| `Signal<Args>::*` | Yes (`Args: 'static`) | No |
| `ObjectRef<T>::*` | Yes | No |
| `WeakRef<T>::*` | Yes | No |
| `ObjectExt` default methods | Blanket impl | No |
| `ObjectFactory::register` | Yes (`F: Fn(...)`) | No |
| `ObjectTree::with` / `with_mut` | Yes (`F: FnOnce(...)`) | No |
| `ObjectBase::named` | Yes (`impl Into<String>`) | No |
| `ObjectTree::rename` | Yes (`impl Into<String>`) | No |

All targets below are concrete, non-generic functions.

## Decomposition

Each task is one file. Within a file all `#[inline]` insertions are done in a
single edit pass. Tasks are independent — no ordering dependency.

| # | Task | File | Functions annotated | Depends on |
|---|------|------|---------------------|------------|
| 1 | Add `#[inline]` in `id.rs` | `quartzite-core/src/id.rs` | `ObjectId::new`, `ObjectId::raw`, `ObjectId::default`, `ConnectionId::new`, `ConnectionId::raw`, `ConnectionId::default` | — |
| 2 | Add `#[inline]` in `object_base.rs` | `quartzite-core/src/object_base.rs` | `ObjectBase::id`, `ObjectBase::name`, `ObjectBase::receiver_guard`, `ObjectBase::set_name_raw`, `ObjectBase::default` | — |
| 3 | Add `#[inline]` in `meta.rs` | `quartzite-core/src/meta.rs` | `PropertyFlags::none`, `PropertyFlags::read_write`, `PropertyFlags::read_only`, `PropertyFlags::default`, `PropertyMeta::new`, `ParamMeta::new`, `SignalMeta::new`, `MethodMeta::new`, `EnumEntry::new`, `EnumMeta::new`, `EnumMeta::entry_by_name`, `EnumMeta::entry_by_value`, `MetaObject::new`, `MetaObject::property`, `MetaObject::signal`, `MetaObject::method`, `MetaObject::enum_meta`, `noop_lookup_entry_by_name`, `noop_lookup_entry_by_value`, `noop_lookup_property`, `noop_lookup_signal`, `noop_lookup_method`, `noop_lookup_enum` | — |
| 4 | Add `#[inline]` in `signal.rs` | `quartzite-core/src/signal.rs` | `queued_dispatcher` (free function, `#[cfg(feature = "std")]`-gated) | — |
| 5 | Add `#[inline]` in `event_loop.rs` | `quartzite-runtime/src/event_loop.rs` | `EventLoop::is_running`, `EventLoop::sender`, `EventLoop::default` | — |
| 6 | Add `#[inline]` in `timer.rs` | `quartzite-runtime/src/timer.rs` | `Timer::is_running` | — |
| 7 | Add `#[inline]` in `factory.rs` | `quartzite-runtime/src/factory.rs` | `ObjectFactory::default` | — |
| 8 | Add `#[inline]` in `object_tree.rs` | `quartzite-runtime/src/object_tree.rs` | `ObjectTree::default` | — |
| 9 | Emit `#[inline]` in `extend/codegen.rs` + add AC9 tests | `quartzite-macros/src/extend/codegen.rs` | self-ref accessor pair; `AsObject::{object_base, object_base_mut, as_any, as_any_mut}`; parent-chain delegation pair; mixin leaf accessor pair | — |
| 10 | Emit `#[inline]` in `object_impl/codegen.rs` + add AC9 tests | `quartzite-macros/src/object_impl/codegen.rs` | `Object::{meta_object, read_property, write_property, invoke_method, connect_signal}`; `__meta_init_Foo()` | — |
| 11 | Emit `#[inline]` in `meta_enum/codegen.rs` + add AC9 test | `quartzite-macros/src/meta_enum/codegen.rs` | `IntoValue::into_value` | — |

## Exact edit locations

### Task 1 — `quartzite-core/src/id.rs`

| Function | Insert before line | Current text at that line |
|---|---|---|
| `ObjectId::new` | 33 | `    pub fn new() -> Self {` |
| `ObjectId::raw` | 51 | `    pub fn raw(self) -> u64 {` |
| `ObjectId::default` | 57 | `    fn default() -> Self {` |
| `ConnectionId::new` | 94 | `    pub fn new() -> Self {` |
| `ConnectionId::raw` | 109 | `    pub fn raw(self) -> u64 {` |
| `ConnectionId::default` | 115 | `    fn default() -> Self {` |

`Default` impls are plain non-generic `fn default() -> Self` wrappers. They
delegate to `Self::new()` (single call, no branches) — meets the criteria.

### Task 2 — `quartzite-core/src/object_base.rs`

| Function | Line | Notes |
|---|---|---|
| `ObjectBase::name` | 111 | `pub fn name(&self) -> Option<&str>` — single `as_deref()` call |
| `ObjectBase::set_name_raw` | 131 | `pub fn set_name_raw(&mut self, name: Option<String>)` — single assignment |
| `ObjectBase::id` | 145 | `pub fn id(&self) -> ObjectId` — field read, single expression |
| `ObjectBase::receiver_guard` | 160 | `pub fn receiver_guard(&self) -> &Arc<ReceiverGuard>` — field ref |
| `ObjectBase::default` | 184 | `fn default() -> Self` inside `impl Default` — delegates to `Self::new()` |

`ObjectBase::new` and `ObjectBase::named` are **excluded**: `new` has multiple
statements and a `#[cfg(feature = "std")]` conditional; `named` chains two calls
(`name.into()`, `..Self::new()`). Neither fits "at most one function call".

### Task 3 — `quartzite-core/src/meta.rs`

| Function | Kind | Line | Notes |
|---|---|---|---|
| `PropertyFlags::none` | `pub const fn` | 38 | struct literal, no calls |
| `PropertyFlags::read_write` | `pub const fn` | 62 | struct literal, no calls |
| `PropertyFlags::read_only` | `pub const fn` | 86 | struct literal, no calls |
| `PropertyFlags::default` | `fn default` | 100 | single call `Self::read_write()` |
| `PropertyMeta::new` | `pub const fn` | 127 | struct literal, no calls |
| `ParamMeta::new` | `pub const fn` | 157 | struct literal, no calls |
| `SignalMeta::new` | `pub const fn` | 183 | struct literal, no calls |
| `MethodMeta::new` | `pub const fn` | 211 | struct literal, no calls |
| `EnumEntry::new` | `pub const fn` | 245 | struct literal, no calls |
| `noop_lookup_entry_by_name` | free `pub fn` | 251 | returns `None` directly |
| `noop_lookup_entry_by_value` | free `pub fn` | 256 | returns `None` directly |
| `EnumMeta::new` | `pub const fn` | 307 | struct literal, no calls |
| `EnumMeta::entry_by_name` | `pub fn` | 332 | single fn-pointer call |
| `EnumMeta::entry_by_value` | `pub fn` | 347 | single fn-pointer call |
| `noop_lookup_property` | free `pub fn` | 353 | returns `None` directly |
| `noop_lookup_signal` | free `pub fn` | 358 | returns `None` directly |
| `noop_lookup_method` | free `pub fn` | 363 | returns `None` directly |
| `noop_lookup_enum` | free `pub fn` | 368 | returns `None` directly |
| `MetaObject::new` | `pub const fn` | 445 | has `#[allow(clippy::too_many_arguments)]` — insert `#[inline]` **after** that allow attr |
| `MetaObject::property` | `pub fn` | 496 | single fn-pointer call |
| `MetaObject::signal` | `pub fn` | 527 | single fn-pointer call |
| `MetaObject::method` | `pub fn` | 558 | single fn-pointer call |
| `MetaObject::enum_meta` | `pub fn` | 592 | single fn-pointer call |

For `MetaObject::new` the attribute order will be:
```rust
// Each slice + fn-pointer pair is a distinct, non-groupable concern ...
#[allow(clippy::too_many_arguments)]
#[inline]
pub const fn new(
```
`#[inline]` goes after `#[allow(...)]` and before `pub const fn` — both orders
are accepted by rustc; putting it after the `#[allow]` keeps the lint suppression
adjacent to the item it covers.

### Task 4 — `quartzite-core/src/signal.rs`

| Function | Line | Notes |
|---|---|---|
| `queued_dispatcher` | 119 | free `pub fn`, `#[cfg(feature = "std")]`-gated; single call `QUEUED_DISPATCHER.get()` |

`#[inline]` is inserted after the existing `#[cfg_attr(docsrs, ...)]` attribute
and before `pub fn`:
```rust
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[inline]
pub fn queued_dispatcher() -> ...
```

`set_queued_dispatcher` is **excluded**: spec table does not list it. It also
chains `.set(d).map_err(...)` — two distinct call chains.

### Task 5 — `quartzite-runtime/src/event_loop.rs`

| Function | Line | Notes |
|---|---|---|
| `EventLoop::sender` | 69 | single call `self.sender.clone()` |
| `EventLoop::is_running` | 131 | single call `self.running.load(...)` |
| `EventLoop::default` | 137 | single call `Self::new()` |

`EventLoop::new`, `post`, `run`, `stop` are excluded: they have multiple
statements or branches.

### Task 6 — `quartzite-runtime/src/timer.rs`

| Function | Line | Notes |
|---|---|---|
| `Timer::is_running` | 169 | single call `self.running.load(...)` |

### Task 7 — `quartzite-runtime/src/factory.rs`

| Function | Line | Notes |
|---|---|---|
| `ObjectFactory::default` | 63 | `fn default` in `impl Default` — single call `Self::new()` |

`ObjectFactory::register` is generic (`F: Fn(...)`). `create` maps a closure.
Both excluded.

### Task 8 — `quartzite-runtime/src/object_tree.rs`

| Function | Line | Notes |
|---|---|---|
| `ObjectTree::default` | 323 | `fn default` in `impl Default` — single call `Self::new()` |

All other `ObjectTree` methods have branches/loops or are generic (`with`,
`with_mut`, `rename`).

---

### Task 9 — `quartzite-macros/src/extend/codegen.rs`

The file has four `quote!` emitters that produce simple delegation functions.
`#[inline]` is added as `#[inline]` tokens inside each `quote!` block,
immediately before the `fn` keyword of each target function.

**`emit_root_trait_and_impl`** — self-ref accessor pair (lines 69-72):

Before (current):
```rust
impl #self_trait for #self_ident {
    fn #acc(&self) -> &#self_ident { self }
    fn #acc_mut(&mut self) -> &mut #self_ident { self }
}
```

After:
```rust
impl #self_trait for #self_ident {
    #[inline]
    fn #acc(&self) -> &#self_ident { self }
    #[inline]
    fn #acc_mut(&mut self) -> &mut #self_ident { self }
}
```

**`emit_as_object_impl`** — `AsObject` four methods (lines 103-113):

Before:
```rust
impl ::quartzite_core::AsObject for #self_ident {
    fn object_base(&self) -> &::quartzite_core::ObjectBase {
        #object_base_expr
    }
    fn object_base_mut(&mut self) -> &mut ::quartzite_core::ObjectBase {
        #object_base_mut_expr
    }
    fn as_any(&self) -> &dyn ::core::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any { self }
}
```

After:
```rust
impl ::quartzite_core::AsObject for #self_ident {
    #[inline]
    fn object_base(&self) -> &::quartzite_core::ObjectBase {
        #object_base_expr
    }
    #[inline]
    fn object_base_mut(&mut self) -> &mut ::quartzite_core::ObjectBase {
        #object_base_mut_expr
    }
    #[inline]
    fn as_any(&self) -> &dyn ::core::any::Any { self }
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any { self }
}
```

**`emit_delegation_impl`** — parent-chain delegation pair (lines 131-140):

Before:
```rust
impl #parent_trait for #self_ident {
    fn #parent_acc(&self) -> &#parent_ty {
        self.#base_field_ident.#parent_acc()
    }
    fn #parent_acc_mut(&mut self) -> &mut #parent_ty {
        self.#base_field_ident.#parent_acc_mut()
    }
}
```

After:
```rust
impl #parent_trait for #self_ident {
    #[inline]
    fn #parent_acc(&self) -> &#parent_ty {
        self.#base_field_ident.#parent_acc()
    }
    #[inline]
    fn #parent_acc_mut(&mut self) -> &mut #parent_ty {
        self.#base_field_ident.#parent_acc_mut()
    }
}
```

**`emit_mixin_impl`** — mixin leaf accessor pair (lines 154-159):

Before:
```rust
impl #mixin_trait for #self_ident {
    fn #mixin_acc(&self) -> &#mixin_ty { &self.#mixin_field }
    fn #mixin_acc_mut(&mut self) -> &mut #mixin_ty { &mut self.#mixin_field }
}
```

After:
```rust
impl #mixin_trait for #self_ident {
    #[inline]
    fn #mixin_acc(&self) -> &#mixin_ty { &self.#mixin_field }
    #[inline]
    fn #mixin_acc_mut(&mut self) -> &mut #mixin_ty { &mut self.#mixin_field }
}
```

**AC9 tests for Task 9** — add to the existing `#[cfg(test)] mod tests` block:

```rust
// AC9: self-ref accessor pair is emitted with #[inline].
#[test]
fn root_self_ref_accessors_are_inline() {
    let out = emit(quote! {
        #[root]
        struct Widget { x: i32 }
    });
    assert!(out.contains("# [inline]"), "missing #[inline] on self-ref accessor: {out}");
}

// AC9: AsObject methods are emitted with #[inline].
#[test]
fn as_object_methods_are_inline() {
    let out = emit(quote! {
        #[root]
        struct Widget {
            #[base]
            object_base: ObjectBase,
        }
    });
    // Four occurrences: object_base, object_base_mut, as_any, as_any_mut.
    let count = out.matches("# [inline]").count();
    assert!(count >= 4, "expected ≥4 #[inline] tokens, got {count}: {out}");
}

// AC9: parent-chain delegation pair is emitted with #[inline].
#[test]
fn delegation_methods_are_inline() {
    let out = emit(quote! {
        struct Button {
            #[base]
            widget: Widget,
        }
    });
    assert!(out.contains("# [inline]"), "missing #[inline] on delegation method: {out}");
}

// AC9: mixin leaf accessors are emitted with #[inline].
#[test]
fn mixin_accessors_are_inline() {
    let out = emit(quote! {
        struct Panel {
            #[mixin]
            layout_base: LayoutBase,
        }
    });
    assert!(out.contains("# [inline]"), "missing #[inline] on mixin accessor: {out}");
}
```

Note: `quote!` serialises attribute tokens with a space — `#[inline]` becomes
`# [inline]` in `.to_string()`. This is the standard proc-macro2 representation
and is what the existing tests also rely on (e.g., `"impl :: quartzite_core ::`).

---

### Task 10 — `quartzite-macros/src/object_impl/codegen.rs`

Two emitter functions need changes.

**`emit_meta_static`** — `__meta_init_Foo` function (lines 185-189):

Before:
```rust
#[allow(non_snake_case)]
fn #meta_init_fn() -> &'static ::quartzite_core::MetaObject {
    &#meta_static_name
}
```

After:
```rust
#[allow(non_snake_case)]
#[inline]
fn #meta_init_fn() -> &'static ::quartzite_core::MetaObject {
    &#meta_static_name
}
```

**`emit_object_impl`** — all five `Object` trait methods (lines 202-227):

Before:
```rust
impl ::quartzite_core::Object for #self_ty {
    fn meta_object(&self) -> &'static ::quartzite_core::MetaObject {
        #meta_init()
    }
    fn read_property(&self, name: &str) -> ::core::option::Option<::quartzite_core::Value> {
        #mod_ident::#read_fn(self, name)
    }
    fn write_property(&mut self, name: &str, val: ::quartzite_core::Value) -> bool {
        #mod_ident::#write_fn(self, name, val)
    }
    fn invoke_method(
        &mut self,
        name: &str,
        args: &[::quartzite_core::Value],
    ) -> ::core::option::Option<::quartzite_core::Value> {
        #invoke_fn(self, name, args)
    }
    fn connect_signal(
        &mut self,
        signal: &str,
        callback: ::quartzite_core::SignalCallback,
    ) -> ::core::option::Option<::quartzite_core::ConnectionId> {
        #mod_ident::#connect_fn(self, signal, callback)
    }
}
```

After: each `fn` is preceded by `#[inline]`:
```rust
impl ::quartzite_core::Object for #self_ty {
    #[inline]
    fn meta_object(&self) -> &'static ::quartzite_core::MetaObject {
        #meta_init()
    }
    #[inline]
    fn read_property(&self, name: &str) -> ::core::option::Option<::quartzite_core::Value> {
        #mod_ident::#read_fn(self, name)
    }
    #[inline]
    fn write_property(&mut self, name: &str, val: ::quartzite_core::Value) -> bool {
        #mod_ident::#write_fn(self, name, val)
    }
    #[inline]
    fn invoke_method(
        &mut self,
        name: &str,
        args: &[::quartzite_core::Value],
    ) -> ::core::option::Option<::quartzite_core::Value> {
        #invoke_fn(self, name, args)
    }
    #[inline]
    fn connect_signal(
        &mut self,
        signal: &str,
        callback: ::quartzite_core::SignalCallback,
    ) -> ::core::option::Option<::quartzite_core::ConnectionId> {
        #mod_ident::#connect_fn(self, signal, callback)
    }
}
```

**Excluded from Task 10:**
- `__invoke_method_*`: has a `match name { ... }` — branches present.
- `__lookup_method_*` and `__lookup_enum_*`: have `match` arms — branches present.
- `emit_methods_static`: emits a `const` slice, not a function.

**AC9 tests for Task 10** — add to the existing `#[cfg(test)] mod tests` block:

```rust
// AC9: all five Object trait shims are emitted with #[inline].
#[test]
fn object_trait_shims_are_inline() {
    let out = emit(quote! {
        impl Foo {}
    });
    // Count occurrences: five shims + one __meta_init fn = 6 minimum.
    let count = out.matches("# [inline]").count();
    assert!(count >= 6, "expected ≥6 #[inline] tokens (5 shims + meta_init), got {count}: {out}");
}

// AC9: meta_init fn is emitted with #[inline].
#[test]
fn meta_init_fn_is_inline() {
    let out = emit(quote! {
        impl Foo {}
    });
    assert!(
        out.contains("# [inline] fn __meta_init_Foo"),
        "missing #[inline] on __meta_init_Foo: {out}"
    );
}
```

---

### Task 11 — `quartzite-macros/src/meta_enum/codegen.rs`

Only `IntoValue::into_value` is in scope. `FromValue::from_value` has `if let`
and `match` — excluded per spec.

The `impl ::quartzite_core::IntoValue for #type_ident` block (lines 83-88):

Before:
```rust
impl ::quartzite_core::IntoValue for #type_ident {
    fn into_value(self) -> ::quartzite_core::Value {
        ::quartzite_core::Value::Int(self as i64)
    }
}
```

After:
```rust
impl ::quartzite_core::IntoValue for #type_ident {
    #[inline]
    fn into_value(self) -> ::quartzite_core::Value {
        ::quartzite_core::Value::Int(self as i64)
    }
}
```

**AC9 test for Task 11** — add to the existing `#[cfg(test)] mod tests` block:

```rust
// AC9: IntoValue::into_value is emitted with #[inline].
#[test]
fn into_value_is_inline() {
    let out = emit(quote! { enum Color { Red } });
    assert!(
        out.contains("# [inline]"),
        "missing #[inline] on into_value: {out}"
    );
}

// AC9: FromValue::from_value does NOT get #[inline] (has branches).
#[test]
fn from_value_is_not_inline() {
    let out = emit(quote! { enum Color { Red } });
    // The only #[inline] in the output is the one on into_value.
    // from_value's fn declaration is preceded by nothing — no extra #[inline].
    // We verify this indirectly: exactly one occurrence of # [inline].
    let count = out.matches("# [inline]").count();
    assert_eq!(count, 1, "expected exactly 1 #[inline] (into_value only), got {count}: {out}");
}
```

## Risks

- **clippy false positive:** clippy does not warn about `#[inline]` on simple
  functions; no lint suppression needed.
- **`const fn` + `#[inline]`:** fully supported in all Rust editions, including
  edition 2024. No risk.
- **`#[cfg(feature = "std")]`-gated items:** `#[inline]` on a cfg-gated function
  is inert when the feature is absent. No conditional compilation needed on the
  attribute itself.
- **Existing `#[allow(clippy::too_many_arguments)]` on `MetaObject::new`:**
  inserting `#[inline]` after it is valid; the allow attribute applies to the
  item regardless of attribute order.
- **`cargo fmt` drift:** `#[inline]` on its own line before `pub`/`fn` is the
  canonical format; `cargo fmt` will not reformat it.
- **proc-macro2 token spacing:** `#[inline]` in a `quote!` block serialises to
  `# [inline]` (with a space) in `.to_string()`. All AC9 test assertions use
  `"# [inline]"` as the needle to match this representation correctly.

## Test Design

### Hand-written functions (Tasks 1–8)

No new tests needed — the existing tests in each file remain green after a
pure annotation change, and the CI gates (`cargo build`, `cargo clippy -- -D
warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings" cargo doc
--no-deps`, `cargo test`) provide complete coverage.

### Codegen functions (Tasks 9–11) — AC9

Each codegen file already has a `#[cfg(test)] mod tests` block. New tests are
added to the existing block in each file.

**`extend/codegen.rs` — 4 new tests:**

| Test name | Scenario | Assertion |
|---|---|---|
| `root_self_ref_accessors_are_inline` | root struct with no base | `out.contains("# [inline]")` |
| `as_object_methods_are_inline` | root struct with `ObjectBase` base | `out.matches("# [inline]").count() >= 4` |
| `delegation_methods_are_inline` | non-root struct with widget base | `out.contains("# [inline]")` |
| `mixin_accessors_are_inline` | mixin-only struct | `out.contains("# [inline]")` |

**`object_impl/codegen.rs` — 2 new tests:**

| Test name | Scenario | Assertion |
|---|---|---|
| `object_trait_shims_are_inline` | empty `impl Foo {}` | `out.matches("# [inline]").count() >= 6` |
| `meta_init_fn_is_inline` | empty `impl Foo {}` | `out.contains("# [inline] fn __meta_init_Foo")` |

**`meta_enum/codegen.rs` — 2 new tests:**

| Test name | Scenario | Assertion |
|---|---|---|
| `into_value_is_inline` | single-variant enum | `out.contains("# [inline]")` |
| `from_value_is_not_inline` | single-variant enum | `out.matches("# [inline]").count() == 1` |

Location: all tests live in the existing `#[cfg(test)] mod tests` block within
each codegen file. No new test files are required.

## Open questions

None.
