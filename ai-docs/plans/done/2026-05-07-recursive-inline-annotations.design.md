# Design: Annotate concrete simple fns surfaced by recursive `#[inline]` rule

**Issue:** #115
**Date:** 2026-05-07

## Approach

Annotation-only sweep. No logic, signature, or body changes — every edit is
either an inserted `#[inline]` attribute on a concrete simple fn or an
inserted `/// _Simple._` italic doc-tag line on a generic simple fn or on a
trait method declaration whose every conforming impl is required to be simple.

The recursive *simple* test from `AGENTS.md` Code Style is applied bottom-up:
mark leaf-most candidates first (their callers' simplicity may now depend on
this annotation), then re-evaluate callers, then their callers, until no fn in
the workspace satisfies the recursive simple test without a marker. Per AGENTS.md
the cascade is bounded — the simple class is small (leaves and trivial
wrappers) and rarely goes more than 2 levels deep.

### Marker shape (exhaustive, mutually exclusive)

| Fn shape | Marker | Reason |
|---|---|---|
| Concrete (no own type parameters) | `#[inline]` attribute on the line above the `fn` | Cross-crate inlining requires the attribute. |
| Generic (own `<T>` / `<F: ...>` / `impl Trait`) | `/// _Simple._` doc-tag line under summary | Body is already cross-crate via monomorphization; the tag is the human signal. |
| Trait method *declaration* whose every conforming impl is required to be simple (codegen-driven or contract-enforced) | `/// _Simple._` doc-tag line under summary | Trait declarations have no body; the tag publishes the contract to all impl authors. |

`/// _Simple._` placement: directly after the summary line and any free-form
prose paragraphs, **before** any `# Heading`. Format-mandated by
`ai-docs/doc-convention.md` § *Simple-fn tag*. Searchable as `rg '_Simple\._'`.

### Sources of candidates

1. **Spec-named targets.** Three concrete `ObjectExt` default methods.
2. **`ObjectExt` sibling default methods.** Walked one-by-one against the
   recursive simple test.
3. **Workspace concrete sweep.** Every `pub fn` (and trait-impl-method)
   currently lacking `#[inline]` is reviewed for the recursive simple test.
4. **Workspace generic / trait-decl sweep.** Generic simple fns and trait
   method declarations whose every conforming impl is required to be simple
   get `/// _Simple._`.
5. **Cascade.** After each annotation pass, `rg <fn-name>` for callers and
   re-evaluate.

### Out of scope (per spec)

- Marker stripping for fns that are *no longer* simple.
- Refactoring fn bodies to make them simple.
- Codegen restructuring beyond what the sweep naturally surfaces.
- API renames / signature changes.

### In scope (broad reading of AC5, confirmed with user 2026-05-07)

- Swapping `#[inline]` for `_Simple._` on generic fns whose existing marker is
  shape-wrong (the fn **does** satisfy the recursive simple test but wears the
  wrong row's marker). The wrong-row marker is "lacking the appropriate
  marker" per AC5. Strip-and-add in the same edit. Identified case:
  `Signal::connect<F>` in `quartzite-core/src/signal.rs:365`.

## Audit results

### Named targets — already verified simple

All three targets are concrete (default method on the blanket `impl<T:
AsObject> ObjectExt for T {}`; the body has no own type parameter, so per
AGENTS.md they fall in the **concrete** row → `#[inline]` attribute).

| Fn | File:lines | Body | Callees | Simple? |
|---|---|---|---|---|
| `ObjectExt::id` | `quartzite-core/src/traits.rs:227–229` | `self.object_base().id()` | `AsObject::object_base` (simple by codegen contract — *to be tagged in this PR*); `ObjectBase::id` (`#[inline]`, field read) | yes — 0 branches, 2 simple calls |
| `ObjectExt::name` | `quartzite-core/src/traits.rs:244–246` | `self.object_base().name()` | `AsObject::object_base` (s.a.); `ObjectBase::name` (`#[inline]`, `as_deref`) | yes — same shape |
| `ObjectExt::is_on_current_thread` | `quartzite-core/src/traits.rs:259–263` (under `#[cfg(feature = "std")]`) | `self.object_base().is_on_current_thread()` | `AsObject::object_base` (s.a.); `ObjectBase::is_on_current_thread` (`#[inline]`) | yes — same shape |

### Sibling `ObjectExt` default methods — classification

| Fn | Shape | Body | Verdict | Marker |
|---|---|---|---|---|
| `downcast_ref<T: AsObject + 'static>` | generic | `self.as_any().downcast_ref::<T>()` | simple (`as_any` is simple by codegen contract; `downcast_ref` is a stdlib-level trivial cast) | `_Simple._` doc tag |
| `downcast_mut<T: AsObject + 'static>` | generic | `self.as_any_mut().downcast_mut::<T>()` | simple | `_Simple._` doc tag |
| `is<T: AsObject + 'static>` | generic | `self.as_any().is::<T>()` | simple | `_Simple._` doc tag |

No other default methods exist on `ObjectExt`.

### Trait method declarations to tag with `_Simple._`

Per AGENTS.md → *Trait-tag examples in this codebase*: every conforming impl
must be simple by codegen contract.

| Trait | Method | File | Why simple-by-contract |
|---|---|---|---|
| `AsObject` | `object_base` | `quartzite-core/src/traits.rs:36` | codegen emits `&self.<base-field>` (single field access) |
| `AsObject` | `object_base_mut` | `quartzite-core/src/traits.rs:49` | codegen emits `&mut self.<base-field>` |
| `AsObject` | `as_any` | `quartzite-core/src/traits.rs:63` | codegen emits `self` |
| `AsObject` | `as_any_mut` | `quartzite-core/src/traits.rs:77` | codegen emits `self` |
| `Object` | `meta_object` | `quartzite-core/src/traits.rs:95` | codegen emits `__meta_init_*()` (single delegation) |
| `Object` | `connect_signal` | `quartzite-core/src/traits.rs:181` | codegen emits `__connect_signal_dynamic_*(...)` (single delegation; the `match` lives inside the helper, not in the trait method body) |
| `Object` | `emit_signal` | `quartzite-core/src/traits.rs` (line determined at edit time) | codegen emits `__emit_signal_*(...)` — identical shape to `connect_signal`, single delegation. Added per design-review Note 2 (codegen evidence: `quartzite-macros/src/object_impl/codegen.rs:256-262`). |

**Not tagged** (per AGENTS.md): `Object::read_property`, `Object::write_property`,
`Object::invoke_method`. AGENTS.md treats these as non-simple by contract
because hand-written impls are likely to inline a `match` body — tagging would
overclaim and mislead. We follow AGENTS.md and leave them unmarked.

`Event<T>::event_type` (in `quartzite-event-types/src/event.rs:108`) is
**not** tagged: `Event` is user-implementable and the contract does not require
simple impls. Every impl currently in this workspace happens to be simple, but
that's incidental.

### Concrete fns to mark with `#[inline]`

Found by sweeping every `pub fn` lacking the attribute and applying the
recursive simple test, with the well-established codebase convention that
trivial stdlib constructors (`Arc::new`, `Mutex::new`, `AtomicBool::new`,
`PhantomData`, `i32::min`/`max`) and primitive arithmetic count as "free"
(simple) — consistent with how existing `#[inline]` markers were applied to
e.g. `ObjectId::new` (1 atomic op), `ObjectBase::is_on_current_thread`
(`std::thread::current().id()` chain).

| Fn | File:line | Shape | Body | Notes |
|---|---|---|---|---|
| `Signal::<Args>::default()` | `quartzite-core/src/signal.rs:317` | concrete (own type param `Args` is on the impl, not the fn) | struct-literal: `IndexMap::new()` + cfg-only branches; no runtime branches | live cascade target — `Signal::new()` already carries `#[inline]` and depends on this being simple. Added per design-review Note 1. |
| `Rect::united` | `quartzite-geometry/src/rect.rs:204` | concrete | 4 `min`/`max` calls + `Rect::new(Point::new, Size::new)` (all simple callees) | no branches/loops; arithmetic |
| `Rect::adjusted` | `quartzite-geometry/src/rect.rs:252` | concrete | 4 arithmetic + `Rect::new(Point::new, Size::new)` | identical shape |
| `RectF::united` | `quartzite-geometry/src/rect.rs:462` | concrete | f32 variant of `Rect::united` | identical shape |
| `RectF::adjusted` | `quartzite-geometry/src/rect.rs:513` | concrete | f32 variant of `Rect::adjusted` | identical shape |
| `Clone for ObjectRef<T>::clone` | `quartzite-runtime/src/object_ref.rs:99` | concrete (impl-method; `T` is on impl, not on fn) | `*self` | `Copy`-derived deref |
| `PartialEq for ObjectRef<T>::eq` | `quartzite-runtime/src/object_ref.rs:105` | concrete | `self.id == other.id` | primitive comparison |
| `Hash for ObjectRef<T>::hash<H>` | `quartzite-runtime/src/object_ref.rs:112` | **generic** (own `H` type param) | `self.id.hash(state)` | single delegation; gets `_Simple._` doc tag — see *Generic candidates* below |
| `Clone for WeakRef<T>::clone` | `quartzite-runtime/src/object_ref.rs:208` | concrete | `*self` | s.a. |
| `PartialEq for WeakRef<T>::eq` | `quartzite-runtime/src/object_ref.rs:214` | concrete | `self.id == other.id` | s.a. |
| `Hash for WeakRef<T>::hash<H>` | `quartzite-runtime/src/object_ref.rs:221` | generic | `self.id.hash(state)` | gets `_Simple._` |

**Note (trait-impl-methods):** Per `ai-docs/doc-convention.md`, methods inside
`impl Trait for Type { … }` blocks are **exempt from the doc convention**
(they inherit docs from the trait definition). They are **not** exempt from
the simplicity rule — `#[inline]` (or `_Simple._` for generic ones) still
applies and is the visible marker reviewers look for. Doc-convention exemption
means we don't add `# Examples`/`# Parameters`; the marker still goes above
the `fn` line (or, for `_Simple._`, on a free-standing line above the `fn`,
where rustdoc treats it as a doc comment for the impl method even when the
method otherwise inherits trait docs).

**Borderline / not marked** (left for a future tightening if the user
requests):

- `ConnectionTable::new` (`quartzite-runtime/src/connection_table.rs:76`) —
  struct-literal in `Arc::new(...)` with 3 `RwLock::new(HashMap::new())` calls
  inside the literal. Six trivial-stdlib calls in total. Borderline; in the
  spirit of the rule, qualifies, but six trivial calls strains the
  "single-call delegation" archetype the spec named. Skip this PR; reviewers
  may ask for it explicitly.
- `ThreadDriver::new`, `AppDriver::new` (`quartzite-runtime/src/timer_drivers.rs:63,150`) —
  three trivial-stdlib calls each. Same reasoning.
- `EventLoop::new` (`quartzite-runtime/src/event_loop.rs:47`) — `mpsc::channel()`
  destructure + struct literal with `parking_lot::Mutex::new`,
  `Arc::new(AtomicBool::new(false))`. Borderline.
- `EventLoop::post` (`quartzite-runtime/src/event_loop.rs:71`) — single
  `self.sender.send(f)` plus a feature-gated tracing span guard. The span
  guard counts as one non-simple call; the `send(...)` call is on a non-simple
  callee. Two non-simple calls → over budget. Skip.
- `EventLoop::stop`, `install_for_current_thread`, `uninstall_for_current_thread` —
  each carries a `debug_span!` guard. **Re-evaluation owed during implementation
  (per design-review Note 3):** if the body is `let _g = …_span!(…).entered();
  <single delegation>`, total non-simple calls = 1 (span macro) + 1
  (delegation) = 2 → over budget. If the delegation is itself simple (e.g.
  atomic store), total = 1 → within budget. The implementer reads each body in
  Task 8 and decides.
- `ObjectFactory::install` — `set(...).map_err(...)` chain on `OnceLock` plus
  a trivial closure. Multi-call; skip.
- `ConnectionTable::install_as_dispatcher` — single delegation
  (`set_queued_dispatcher(Arc::clone(self) as ...)`); arguably simple. Mark
  if it lights up callers; otherwise leave for a future pass.
- `MetaObject::eq`, `EnumMeta::eq` — multi-equality `&&` chain. No branches in
  the AGENTS.md sense (`if`/`match`/`while`/`for`), but treats `&&`-chains
  conservatively as outside the "field getter / single delegation" archetype.
  Leave unmarked.
- `Signal::emit_unconditionally`, `Signal::disconnect`, `Signal::connect_typed`,
  `Signal::connect_queued`, `Signal::connect_auto`, `connect_signal_to_signal`,
  `connect_signals` — explicitly non-simple (loops, branches, multiple non-simple
  calls).
- `Application::new`, `try_with_tree`, `ObjectTree::*` (apart from already-marked
  trivial wrappers), `ThreadPool::new`, `ThreadPool::spawn`, `PoolDriver::new`,
  `Timer::new`, `Timer::named`, `Timer::start`, `Timer::stop`, `Timer::emit_tick`,
  `Timer::connect_tick*`, `Timer::disconnect_tick`, `Value::type_name` (match),
  `ReceiverGuard::new_pair`, `ObjectBase::new`, `ObjectBase::named`,
  `ObjectBase::new_with_id` — all clearly non-simple.

### Generic candidates for `_Simple._`

| Fn | File:line | Body | Notes |
|---|---|---|---|
| `Hash for ObjectRef<T>::hash<H: Hasher>` | `quartzite-runtime/src/object_ref.rs:112` | `self.id.hash(state)` | own type param `H`; single delegation |
| `Hash for WeakRef<T>::hash<H: Hasher>` | `quartzite-runtime/src/object_ref.rs:221` | `self.id.hash(state)` | same |
| `ObjectExt::downcast_ref<T>` | `quartzite-core/src/traits.rs:278` | `self.as_any().downcast_ref::<T>()` | own type param `T` |
| `ObjectExt::downcast_mut<T>` | `quartzite-core/src/traits.rs:294` | `self.as_any_mut().downcast_mut::<T>()` | same |
| `ObjectExt::is<T>` | `quartzite-core/src/traits.rs:308` | `self.as_any().is::<T>()` | same |

**Shape-wrong existing marker — IN SCOPE (broad AC5 reading, confirmed
2026-05-07):** `Signal::connect<F>` in `quartzite-core/src/signal.rs:365` is
generic but currently carries `#[inline]` (legacy from before the recursive
rule). The correct shape is `_Simple._`. **Action:** strip the `#[inline]`
attribute, add the `_Simple._` doc tag in the same edit. Tracked as Task 7a
in the decomposition.

### Cascade analysis

After the listed annotations are applied, the only concrete fns in the
workspace whose simplicity changes status are:

- `Margins::apply` (`quartzite-geometry/src/margins.rs:128`) — already
  `#[inline]`. Body is `rect.adjusted(...)`. Was simple under the old
  definition (single delegation); after marking `Rect::adjusted`, it remains
  simple. **No cascade change.**

For `ObjectExt::id`/`name`/`is_on_current_thread`: cascade reaches code that
calls `obj.id()` / `obj.name()` / `obj.is_on_current_thread()` on a generic
`obj: &impl AsObject`. Inspection of every workspace caller (search transcripts
filtered to non-test, non-doctest call sites) finds only two relevant callers,
both already simple-classified or excluded:

- `Timer::start` and `Timer::stop` use `self.base.id()` (the inherent method
  on `ObjectBase`, not the trait method on `ObjectExt`). No cascade from
  `ObjectExt::id`.
- Application code reads `app.main_thread_id()` (its own inherent method).

For the `AsObject` trait-method tagging: many callers benefit transitively
(every fn whose body chains through `object_base()` etc.), but each such
caller already passes the recursive test from a different angle (most are
already `#[inline]`). The cascade-quiescence check in the implementation phase
(re-running the same un-marked-pub-fn audit) should yield zero new
candidates.

## Decomposition

Twelve atomic tasks, ordered bottom-up so each pass's annotations are visible
to the next pass's recursive evaluation. Within each task, edits are mechanical
attribute / doc-tag insertions; no test changes. After the final task, run the
complete CI gate.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Tag `AsObject::{object_base, object_base_mut, as_any, as_any_mut}` with `_Simple._` (trait-decl row). | `quartzite-core/src/traits.rs` | — |
| 2 | Tag `Object::{meta_object, connect_signal, emit_signal}` with `_Simple._` (trait-decl row). Leave `read_property`/`write_property`/`invoke_method` unmarked per AGENTS.md. **`emit_signal` added per design-review Note 2:** codegen at `quartzite-macros/src/object_impl/codegen.rs:256-262` emits a single `__emit_signal_*` delegation, identical shape to `connect_signal`. | `quartzite-core/src/traits.rs` | 1 |
| 3 | Add `#[inline]` to `ObjectExt::{id, name, is_on_current_thread}` (concrete row, default body on blanket-impl trait). Verify each via the recursive simple rule using the now-tagged callees from Tasks 1–2. **`is_on_current_thread`** sits under `#[cfg(feature = "std")]` + `#[cfg_attr(docsrs, doc(cfg(feature = "std")))]` — `#[inline]` goes between the cfg stack and `fn`, matching `ObjectBase::is_on_current_thread`. | `quartzite-core/src/traits.rs` | 1, 2 |
| 4 | Tag `ObjectExt::{downcast_ref, downcast_mut, is}` with `_Simple._` (generic row). | `quartzite-core/src/traits.rs` | 1 |
| 5 | Add `#[inline]` to `Rect::{united, adjusted}` and `RectF::{united, adjusted}` (concrete row, arithmetic + simple-callee chains). | `quartzite-geometry/src/rect.rs` | — |
| 6 | Add `#[inline]` to `Clone for ObjectRef<T>::clone`, `PartialEq for ObjectRef<T>::eq`, `Clone for WeakRef<T>::clone`, `PartialEq for WeakRef<T>::eq` (concrete row, trait-impl methods inside generic impl blocks). | `quartzite-runtime/src/object_ref.rs` | — |
| 7 | Add `#[inline]` to `Signal::<Args>::default()` (concrete row; struct-literal constructor with cfg-only branches). **Added per design-review Note 1:** live cascade target — `Signal::new()` at line 343 already carries `#[inline]` and depends on this being simple. | `quartzite-core/src/signal.rs` | — |
| 7a | **Marker shape-swap on `Signal::connect<F>`** (`quartzite-core/src/signal.rs:365`). Strip the existing `#[inline]` attribute and add a `/// _Simple._` doc tag in the same edit. The fn is generic (own `<F>` type param), so the correct row is `_Simple._`. In scope per the spec's broad AC5 reading. | `quartzite-core/src/signal.rs` | — |
| 8 | **Cascade pass 1** (workspace re-audit). Re-run the same un-marked-pub-fn enumeration after Tasks 1–7a. For every concrete `pub fn` still un-`#[inline]`, re-evaluate against the recursive simple test using the newly-tagged callees from prior tasks. **Also re-evaluate the borderline `EventLoop` fns from the *Borderline* list (Note 3 from design review):** `EventLoop::stop`, `EventLoop::install_for_current_thread`, `EventLoop::uninstall_for_current_thread`. Read each body; count `debug_span!` guard as one non-simple call; if the rest of the body is one delegation, total = 2 → skip; if the rest is simple-callee-only (e.g. atomic store), total = 1 → mark. Document the per-fn outcome in the progress file. Expected outcome: zero or one new fn-candidates. | (audit only unless candidates surface — then a small per-file edit) | 1–7a |
| 9 | **Cascade pass 2** (codegen re-audit). Inspect codegen output (`quartzite-macros/src/extend/codegen.rs`, `quartzite-macros/src/object_impl/codegen.rs`, `quartzite-macros/src/meta_enum/codegen.rs`, `quartzite-macros/src/object/codegen.rs`) to confirm the codegen still emits `#[inline]` on every generated concrete simple fn it should, given the recursive-rule callees. No new generated fns are introduced; the existing ones remain correct. | (audit only — no edits unless gaps surface) | 1–7a |
| 10 | Compilation gate: `cargo build` (refreshes `Cargo.lock`); `cargo build -p quartzite --no-default-features` (no_std / derive-free path). | (no edits — just verification) | 1–8 |
| 11 | Lint + test gate: `cargo clippy -- -D warnings`; `cargo fmt -- --check`; `cargo test`. | (no edits) | 10 |
| 12 | Doc gate: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`. Confirms every newly-inserted `_Simple._` line passes intra-doc-link / formatting checks. | (no edits) | 11 |

The original Task 7 (`_Simple._` doc tag on `Hash for ObjectRef<T>::hash<H>` /
`WeakRef<T>::hash<H>`) is **dropped** per Risk #3: doc-convention exempts
trait-impl-method docs, and adding the tag would override trait-inherited
docs. The `Hash` trait declaration in `core::hash` is upstream / not tag-
eligible. These two impls remain correctly simple but unmarked.

If Task 8 or 9 surfaces an additional candidate beyond the EventLoop
re-evaluation, append a Task 8a / 9a (one file each) and re-run Tasks 10–12
afterwards.

## Test Design

This PR is annotation-only. **No new functional tests** are needed — every
existing test exercises the marked fns and stays green after a pure attribute
or doc-tag insertion.

**Verification by CI gate.** The four CI commands in Tasks 10–12 collectively
prove:

- *Compilation* (`cargo build`, `cargo build -p quartzite --no-default-features`) —
  attribute is syntactically valid and the doc-tag does not break feature gates
  (the named target `ObjectExt::is_on_current_thread` is `#[cfg(feature =
  "std")]` — the inline marker must sit *between* the `cfg`/`cfg_attr` block
  and the `fn` keyword, matching the established pattern in
  `ObjectBase::is_on_current_thread`).
- *Behavioural* (`cargo test`) — every existing unit, integration, and doctest
  remains green.
- *Lint* (`cargo clippy -- -D warnings`) — no clippy lint regressions; in
  particular `clippy::doc_markdown` does not flag the `_Simple._` italic line
  (underscore-italic is canonical Markdown).
- *Format* (`cargo fmt -- --check`) — no formatting drift introduced.
- *Docs* (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps
  --workspace`) — every `///` block remains well-formed; the inserted
  `/// _Simple._` line is valid Markdown that rustdoc renders as italicised
  prose without breaking the strict section order from
  `ai-docs/doc-convention.md`.

**Spot-check protocol** (manual, recorded in the Implementation phase progress
file): pick three just-annotated fns at random, `rg <fn-name>` for callers,
and confirm none of those callers became newly-eligible (or, if any did,
extend the cascade per Tasks 8/8a until they're marked).

**Mutual-exclusion check (AC9)** — two greps confirming no fn carries both
markers:

```bash
# Files that contain both markers anywhere — review each match by hand.
rg -l '#\[inline\]' -g '!**/target/**' \
  | xargs rg -l '_Simple\._' 2>/dev/null

# Per-fn proximity check: the worry is the same `pub fn`/`fn` line being
# preceded *both* by an `#[inline]` attribute and a `_Simple._` doc-comment
# line. Any survivor of this two-pass grep is a manual review item:
rg -B5 '^[[:space:]]*(pub )?(const )?(unsafe )?fn ' -g '*.rs' \
  -g '!**/target/**' \
  | rg -B0 -A0 '_Simple\._' \
  | rg -B5 -A0 '#\[inline\]'
```

## Risks

- **Stdlib-call interpretation drift.** The recursive simple rule's "calls to
  non-simple fns" budget is well-defined for fns inside this workspace; for
  stdlib calls (`Arc::new`, `Mutex::new`, `i32::min`, `Atomic*::store/load`,
  `std::thread::current().id()`) the codebase implicitly treats trivial
  constructors / accessors as "free." This is consistent with how previous
  `#[inline]` markers were applied (`ObjectId::new`,
  `ObjectBase::is_on_current_thread`). **Mitigation:** the audit table above
  uses this same convention; any reviewer disagreement on a specific stdlib
  call's classification is resolved by *not* marking the fn (path of least
  resistance is to leave it unmarked), and the borderline list explicitly
  flags candidates the convention would catch but which strain the spirit of
  "trivial wrapper" — those are deferred.

- ~~**Existing shape-wrong markers.**~~ Resolved 2026-05-07: user confirmed
  broad AC5 reading. Marker-shape swap on `Signal::connect<F>` is in scope —
  see Task 7a in the Decomposition.

- **Trait-impl method doc-tag rendering.** Methods inside `impl Trait for
  Type {}` blocks are doc-convention-exempt — they inherit trait docs. Adding
  a `/// _Simple._` line above an impl-method (e.g.
  `Hash for ObjectRef<T>::hash`) is unusual: rustdoc *will* render it as a
  per-impl doc override, displacing the trait-inherited doc. **Mitigation:**
  for trait-impl-methods marked simple via the generic row (Task 7), do *not*
  introduce a `_Simple._` doc-tag — the impl block is exempt and the marker
  has no good home. Instead, rely on the trait declaration's tag (when
  present); if the trait declaration is not tag-eligible (as for `Hash`,
  which is user-implementable), leave the impl unmarked. This means we
  **drop** the `Hash::hash` entries (Task 7) and instead leave them unmarked.
  Open Question below confirms this choice. Concrete (non-generic) impl-block
  methods like `Clone::clone` and `PartialEq::eq` use `#[inline]` (an
  attribute, not a doc tag) and don't have this problem — Task 6 stays.

- **Cascade ambiguity.** If Task 8 surfaces unexpected candidates, the PR
  scope grows. **Mitigation:** the audit performed during design-time is
  thorough; the expected cascade outcome is zero new candidates. If a
  reviewer-found candidate appears mid-implementation, append a Task 8a and
  re-run gates.

- **`#[cfg]`-gated attribute placement.** `ObjectExt::is_on_current_thread` is
  under `#[cfg(feature = "std")]` and `#[cfg_attr(docsrs, doc(cfg(feature =
  "std")))]`. The `#[inline]` attribute must sit between the existing
  attribute stack and the `fn` keyword, matching the established pattern in
  `ObjectBase::is_on_current_thread` (`quartzite-core/src/object_base.rs:277–280`):

  ```rust
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  #[inline]
  fn is_on_current_thread(&self) -> bool {
      self.object_base().is_on_current_thread()
  }
  ```

  **Mitigation:** explicit instruction in Task 3.

- **`_Simple._` placement vs section order.** The doc-tag goes *immediately
  after* the summary line and any free-form prose paragraphs, *before* any
  `# Heading`. For methods that have no free-form prose between summary and
  the first heading (most of `ObjectExt::*`), that means: blank `///` line,
  then `/// _Simple._`, then blank `///` line, then `# Examples`. **Mitigation:**
  follow the conforming example in `ai-docs/doc-convention.md` § *Conforming
  example* under *Simple-fn tag*.

- **`no_std` / derive-free path.** Several targeted fns (e.g.
  `ObjectExt::is_on_current_thread`) are gated on `feature = "std"`. The
  derive-free / `no_std` build (`cargo build -p quartzite --no-default-features`)
  must compile clean. **Mitigation:** Task 10 includes the `no_std` build.

## Open questions

1. ~~Shape-wrong existing markers.~~ **Resolved 2026-05-07:** in scope,
   broad AC5 reading; Task 7a covers the `Signal::connect<F>` swap.

2. ~~Trait-impl method `_Simple._` placement.~~ **Resolved 2026-05-07:**
   accept the design's recommendation. Drop original Task 7 — leave
   `Hash for ObjectRef<T>::hash<H>` and `Hash for WeakRef<T>::hash<H>`
   unmarked. Doc-convention exempts trait-impl-method docs; the `Hash` trait
   is upstream / not tag-eligible.

3. **Borderline struct-literal constructors.** Several `pub fn new()`s
   construct via `Arc::new(Mutex::new(...))`-style chains of trivial stdlib
   calls (`ConnectionTable::new`, `ThreadDriver::new`, `AppDriver::new`,
   `EventLoop::new`). Each contains 3–6 trivial-stdlib calls and no branches
   or loops. The recursive rule, applied with the codebase's "trivial stdlib
   = free" convention, marks them simple; the spirit of the rule (single
   delegation / field getter / wrapper) does not. Recommended resolution:
   **defer** — leave these unmarked this PR; revisit if the user wants the
   broader sweep. Documented as borderline above.
