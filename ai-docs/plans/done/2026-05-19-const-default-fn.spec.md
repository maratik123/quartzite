# Const-initialisable `new()` alongside `Default` impls

**Source:** issue #484
**Date:** 2026-05-19
**Tracked in:** #484

## Scope

1. Inventory every type in the workspace that currently provides `Default`
   (via `#[derive(Default)]` or a hand-written `impl Default for T`) and group it as:
   - **(A)** types whose default initialisation contains only const-eligible
     operations (struct literal of `Copy` primitives, other `const`-constructible
     constructors such as `Vec::new` / `String::new` / `AtomicU64::new` /
     `Option::None`, `Self::OTHER_CONST`, etc.) — eligible to gain a
     `pub const fn new() -> Self`.
   - **(B)** types whose default initialisation requires runtime ops the
     compiler rejects in `const` context (atomic `fetch_add` to seed an ID,
     `mpsc::channel`, thread spawn, FFI calls, anything reading a `OnceLock` /
     mutating a global, etc.) — these keep a **non-`const`** `new()`, but only
     if a small local refactor cannot lift them into Group **(A)**.
2. For every Group **(A)** type (including struct-shaped enums whose default
   variant carries data needing const construction — see *Key decisions* row
   on enums):
   - Remove `#[derive(Default)]` (or the hand-written `impl Default`).
   - Add `pub const fn new() -> Self` containing the initialisation expression,
     `#[inline]`, with a doc comment and `# Examples` block per
     [`ai-docs/doc-convention.md`](../doc-convention.md).
   - Add an explicit `impl Default for T { #[inline] fn default() -> Self { Self::new() } }`.
3. For every Group **(B)** type, apply the **opportunistic-refactor rule**
   from round-1 Q2: attempt a small local refactor that makes the
   initialisation const-eligible (e.g., factor a seeded counter out of
   `Default`, or replace a runtime ID source with `Self::FIRST_ID`). If the
   refactor is local, low-risk, and lands the type in Group **(A)** — do it.
   Otherwise, **skip** the type entirely (no `new()` added, no `Default`
   rewrite, no churn). Group **(B)** is therefore a *residual* category after
   the opportunistic pass, not a uniformly-rewritten set.
4. Types that already expose a `pub const fn new(…) -> Self` (zero-arg or
   multi-arg) such that callers can already write `const X: T = T::new(…)`
   are already conformant — leave untouched
   (e.g. `Color`, `Pen`, `Point`, `PointF`, `Rect`, `RectF`, `Size`, `SizeF`,
   `Margins`).  The spec's payoff (callers gain a `const fn` path) is already
   met; replacing `#[derive(Default)]` with an explicit `impl Default` that
   delegates to `Self::new(…zeros…)` is churn with no observable benefit.
5. All public additions get a doc comment + `# Examples` block per workspace
   `missing_docs = "deny"` lint policy.
6. **PR slicing:** a single PR covering every affected crate (per round-1 Q3).
   No crate-by-crate split; design's atomic-task decomposition lives entirely
   inside the one PR.

## Out of scope

- Making `Default::default()` itself `const` — impossible on stable Rust 1.95
  (no `const_trait_impl`). The task's payoff is callers gaining a `const fn new()`
  usable inside `const`/`static` initialisers; `Default` callers keep the
  identical runtime behaviour they have today.
- Behavioural changes to any default value. The defaults stay byte-identical.
- Performance benchmarks / binary-size measurements — `#[inline] const fn` with
  trivial bodies is expected to be neutral.
- Re-shaping `Default` on types that legitimately cannot have one (none in
  scope here; this task does not add `Default` to types that lack it).
- API renames / visibility flips — naming follows `new` per workspace convention
  already in use.

## Deferred

- (empty) — none identified.

## Key decisions

| Question | Decision |
|---|---|
| `Default::default()` body when `new()` is const | `Self::new()` — single source of truth; trivial inline. |
| Group **(A)** marker | `#[inline]` and `// _Simple._` on `Default::default`; `#[inline]` on `const fn new()` per [`ai-docs/code-style.md` § `#[inline]`](../code-style.md#inline-and-the-_simple_-doc-tag). |
| Enum types deriving `Default` (round-1 Q1) | **Pure unit-variant enums:** out of scope — callers already write `MyEnum::Variant` directly as a const expression, a `new()` would add noise. **Struct-shaped enums** whose default variant carries data needing const construction: in scope — rewrite same as struct Group **(A)** (remove `#[derive(Default)]`, add `pub const fn new() -> Self`, delegate `Default` to `Self::new()`). |
| Zero-field unit structs (PR #487 round-1 reviewer Q) | **Out of scope for `pub const fn new()`.** A zero-field unit struct (e.g. `DefaultStyle`) is itself already a const expression — callers can write `const X: DefaultStyle = DefaultStyle;` without any `new()`. Adding `new()` adds noise with no payoff; same spirit as the pure-unit-variant enum exclusion. `#[derive(Default)]` stays. |
| Geometry types (`Point`, `PointF`, `Rect`, `RectF`, `Size`, `SizeF`, `Margins`) | **Out of scope (already conformant)** — each type already exposes a `pub const fn new(args…)` so callers can write `const P: Point = Point::new(0, 0)` today. The spec's const-construction payoff is already met; replacing `#[derive(Default)]` with an explicit `impl Default { Self::new(0,0) }` would be churn with no observable benefit. Treated identically to `Color` / `Pen` per Scope item 4. |
| Group **(B)** handling (round-1 Q2) | **Opportunistic.** Rewrite a Group **(B)** type only if a small local refactor lifts it into Group **(A)** (const-eligible). If not — leave the type untouched, including its `#[derive(Default)]`. Design's inventory must record the const-blocker for each *skipped* Group **(B)** entry so the decision is auditable; non-skipped, lifted types appear in the Group **(A)** rewrite list. |
| PR slicing (round-1 Q3) | **Single PR** covering every affected crate. Design decomposes into atomic implementation tasks within the one PR; no crate-by-crate split. |
| Pre-publish API-stability posture | AGENTS.md § *API Stability*: break freely, no shims. Removing `#[derive(Default)]` and re-adding an explicit impl is a clean rewrite. |
| Test policy | Existing tests that call `T::default()` keep working unchanged (identical observable behaviour). Add a doctest-level `# Examples` block on each new `const fn new()` showing it inside a `const` binding to prove const-eligibility. |

## Technical constraints

- **Rust 1.95 stable, edition 2024.** No nightly features. `const_trait_impl`
  is NOT available, so `Default` itself stays non-const.
- **Workspace `missing_docs = "deny"`.** Every added public item gets `///` doc
  + `# Examples` (matches AGENTS.md § *Code Style — Documentation*).
- **`no_std` / derive-free path** (`quartzite` crate `--no-default-features
  --features libm`) must still compile — verify with the AGENTS.md `cargo build`
  invocation.
- **Snapshot tests** under `quartzite-style/tests/snapshots/` must remain
  byte-identical (no behavioural change to defaults).
- **Propagation Rule** does not fire — no instruction-file edits.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | For every type identified as Group **(A)** (excluding zero-field unit structs per Key decisions row): a `pub const fn new() -> Self` (zero-arg) exists with `#[inline]` + `# Examples`; `Default` is provided either via `#[derive(Default)]` (when the derived body produces the correct default value) **or** via an explicit `impl Default for T` delegating to `Self::new()` (only when derive cannot produce the correct body, e.g. `Palette`). No `#[allow(clippy::derivable_impls)]` wrappers anywhere in the diff. Types already exposing a multi-arg `pub const fn new(…)` are already conformant per Scope item 4 and are explicitly excluded. Zero-field unit structs (`DefaultStyle`) are excluded — their struct literal is itself a const expression, so `pub const fn new()` adds no payoff. |
| AC2 | For every Group **(B)** type the design judged liftable via small local refactor: the refactor lands, the type appears in the Group **(A)** rewrite list (AC1 applies), and the original Group **(B)** classification is annotated as "lifted" in the design inventory. Group **(B)** types judged **not** liftable are skipped entirely — `#[derive(Default)]` / hand-written `Default` stays untouched, no `new()` is added by this task. |
| AC3 | Each new `pub const fn new() -> Self` has a doctest demonstrating const usage, e.g. `const P: Point = Point::new(0, 0);`. The doctest passes. |
| AC4 | `cargo test --workspace --all-features` is green. `cargo test -p quartzite --no-default-features --features libm` builds. |
| AC5 | `cargo clippy --workspace --all-targets -- -D warnings` is clean. `cargo fmt -- --check` is clean. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` is clean. |
| AC6 | Existing call-sites of `T::default()` continue to compile unchanged — no caller updates required by this task. |
| AC7 | Snapshot tests under `quartzite-style/tests/snapshots/` remain byte-identical (no `cargo insta review` diffs). |
| AC8 | The design document enumerates every type touched (and every type *considered but skipped* from Group **(B)**), classified into Group **(A)** vs Group **(B)**, with the const-blocker named for each skipped Group **(B)** entry and the chosen lift-refactor named for each lifted entry. |
| AC9 | All work lands in a single PR — no crate-by-crate split (round-1 Q3). |

## Open questions

(none — round-1 Q1 / Q2 / Q3 close every design-affecting ambiguity. The
remaining work is mechanical inventory + classification, which belongs to the
design phase.)
