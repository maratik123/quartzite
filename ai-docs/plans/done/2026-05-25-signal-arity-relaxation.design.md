# Design: Signal arity relaxation (signal-to-slot / signal-to-signal)

**Issue:** #566
**Date:** 2026-05-25

## Approach

Three connect entry points in `quartzite-core/src/connect.rs` are loosened and unified around a single rule: **the source signal must carry at least as many parameters as the target requires; excess trailing arguments are dropped via prefix slicing at emit time.** Type-name validation continues, but now only over the retained prefix (the first `to_arity` / `slot_arity` parameters).

### Per-entry behaviour

1. **`connect_signal_to_signal`** (~lines 108–215) — replace `if from_arity != to_arity` with `if from_arity < to_arity`. The `zip` over `from_meta.params` and `to_meta.params` is naturally bounded by the shorter slice (`to_meta.params`), so the type-name loop already covers only the retained prefix; no change is needed there. All four `SignalCallback` arms (Direct, SingleShot, Queued, Auto) currently forward `args` verbatim; each becomes `&args[..to_arity]` (Direct / SingleShot synchronous arms) or slices before the `Vec<Value>::to_vec()` clone for the queued arms (`args[..to_arity].to_vec()`). `to_arity` is captured into the closure as a `usize` move-capture alongside `to_signal_name` / `to_thread_id`.

2. **`connect_signals` (typed)** (~lines 339–447) — same arity-check relaxation (`from_arity < to_arity` rejection). All three closure arms compute `let values = args.to_values();` and then emit; they become `let values = args.to_values(); arc.lock().emit_signal(&to_signal_str, &values[..to_arity])`. Captured `to_arity: usize` alongside `to_signal_str`. `ArgsToValues::to_values()` itself is unchanged — the slice is taken on the receiving side, matching the spec's "truncation realised through `ArgsToValues::to_values()` slicing" wording.

3. **`connect_signal_to_slot`** (~lines 259–283) — introduces the meta-lookup branch:
   - At connection time, after the `from_meta` lookup, also call `target.lock().meta_object().method(slot_name)`.
   - **`Some(meta)` branch (validated path):** enforce `from_arity >= meta.params.len()` (return `ArityMismatch { from: from_arity, to: meta.params.len() }` on violation); validate `type_name` equality over the first `meta.params.len()` positions of `from_meta.params` zipped with `meta.params` (same loop shape as signal-to-signal). On success, capture `slot_arity: usize = meta.params.len()` into the callback, and replace the `&[]` emit-time call with `&args[..slot_arity]`.
   - **`None` branch (fallback / "lazy" path):** preserve the current behaviour verbatim — connection succeeds with no further validation, callback invokes the slot with `&[]`. No `slot_arity` capture in this arm. This is the documented escape hatch for hand-rolled objects whose `invoke_method` is not advertised via the meta system (e.g. the test's `ClickSource` whose `lookup_method` is `noop_lookup_method`).

### Variant naming

The `SignalConnectionError::ArityMismatch { from, to }` variant is **kept as-is**. The spec invites a rename to `InsufficientArity` per AGENTS.md § API Stability ("pre-publish: clean breaks. No compat shims."), but **I reject it** for two reasons:

- *Semantic fit.* `ArityMismatch` still accurately describes the post-relaxation failure mode: the arities do not match the required relationship (`from >= to`). The Rust ecosystem precedent (`std::io::ErrorKind::InvalidInput`, `serde::de::Error::invalid_length`) names error variants by the kind of constraint violated, not by the specific direction of the violation.
- *Field-name asymmetry.* `from` / `to` continue to carry the same meaning (source arity, target arity), so the rename would have to also rename fields (`actual` / `required`?) to be self-consistent. That is a deeper refactor with no readability gain — the error's `Display` string (`"arity mismatch: source signal has {from} parameters, target has {to}"`) already conveys the new semantics once the `# Errors` rustdoc explains the `from >= to` rule.

The variant's `#[error("…")]` string is **left unchanged**: it already reads "arity mismatch: source signal has {from} parameters, target has {to}" — under the new rule, the source has fewer than the target needs, which the existing wording communicates correctly.

### Rustdoc updates

All three public functions need `# Errors` block + body prose refresh:

- `connect_signal_to_signal` — body prose drops "Type compatibility is validated at connection time by comparing arity and `type_name` strings" in favour of "arity is validated as `from_arity >= to_arity`; `type_name` strings are compared on the first `to_arity` parameters; extra source arguments are dropped at emit time." `# Errors` adds the directional `from < to` clarification on `ArityMismatch`.
- `connect_signals` (typed) — same change. The `# Errors` block already enumerates the four variants; only the `ArityMismatch` line wording shifts.
- `connect_signal_to_slot` — most substantive change. The body prose explains the meta-lookup branch:
  - When `meta_object().method(slot_name)` returns `Some(meta)`: connection-time arity (`from_arity >= meta.params.len()`) + `type_name` validation; emit-time slice `&args[..meta.params.len()]`.
  - When it returns `None`: lazy fallback (existing behaviour preserved verbatim — connection succeeds, emit-time call uses `&[]`).
  - `# Errors` gains `ArityMismatch` and `TypeMismatch` for the validated branch; `UnknownFromSignal` stays.

### Rejected alternatives

- **New variant `InsufficientArity`** — rejected; see "Variant naming" above. The spec explicitly authorises a rename if it improves clarity, and I judge that it does not.
- **Validate slot arity unconditionally (no fallback)** — rejected per spec Key Decisions row 3. The escape hatch for hand-rolled objects is load-bearing for `ClickSource`-style test fixtures and any future user code that hand-rolls `invoke_method` without a meta entry.
- **Reorder / project `(a, b, c) → (a, c)`** — rejected per spec Out of scope.
- **Lift `slot_arity` into a third generic parameter on `connect_signal_to_slot`** — rejected; the function takes a `&dyn Object` and resolves arity dynamically. A generic parameter would force callers to declare arity at the call site, defeating the dynamic-dispatch design of this entry point.
- **Trim the typed-path `Vec<Value>` in place via `values.truncate(to_arity)` instead of slicing** — equivalent but unnecessary; the `emit_signal` call only borrows the slice, and `&values[..to_arity]` avoids the cosmetic `truncate` call. Both compile to a sub-slice on the stack.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Relax `connect_signal_to_signal` arity check (`!=` → `<`); capture `to_arity: usize` into all four `SignalCallback` arms; replace `args` with `&args[..to_arity]` in the Direct / SingleShot / Auto-same-thread arms, and slice before `to_vec()` in the Queued / Auto-cross-thread arms (`args[..to_arity].to_vec()`). The existing `params.iter().zip(to_meta.params.iter())` is naturally bounded by `to_meta.params` (`zip` stops at the shorter iterator) — no change to the type-name loop. Rustdoc body + `# Errors` block refresh per Approach. | `quartzite-core/src/connect.rs` (lines ~108–215) | — |
| 2 | Relax `connect_signals` (typed) arity check the same way; capture `to_arity: usize` into all three Direct / Queued / Auto closure bodies; replace `&values` with `&values[..to_arity]` in each `emit_signal` call. Rustdoc body + `# Errors` block refresh. | `quartzite-core/src/connect.rs` (lines ~339–447) | — |
| 3 | Extend `connect_signal_to_slot` with the meta-lookup branch: after the `from_meta` lookup, take `to.lock().meta_object().method(slot_name)`. In the `Some(meta)` arm, enforce `from_arity >= meta.params.len()` (return `ArityMismatch`), validate `type_name` over the retained prefix (return `TypeMismatch` with `from`/`to` type names), capture `slot_arity: usize` into the callback, replace `&[]` with `&args[..slot_arity]` at the `invoke_method` call. In the `None` arm, preserve the existing `&[]` callback verbatim. Rustdoc body + `# Errors` block refresh. The from-signal lookup MUST stay eager (above the slot meta lookup) so `UnknownFromSignal` remains the first failure to fire on bad input. | `quartzite-core/src/connect.rs` (lines ~259–283) | — |
| 4 | Update existing tests whose assertions encode the old strict-equality semantics. Both `arity_mismatch_returns_error` (`from=1, to=0`) and `connect_signals_typed_arity_mismatch_returns_error` (`from=2, to=1`) currently feed inputs that satisfy `from >= to` — **valid** under the new rule — so both tests must be **flipped**: swap source / target to expose `from=0, to=1` (signal-to-signal) and `from=1, to=2` (typed) to keep the rejection branch covered. Add new positive test `arity_relaxation_truncates_extras` (1-arg source → 0-arg target; see Test Design for fixture details). | `quartzite-core/src/connect.rs` `#[cfg(test)] mod tests` | 1 |
| 5 | Add `connect_signals` typed-path positive test for truncation: 2-arg source signal `(i32, i32)` → 1-arg target signal `(i32,)`, assert captured value equals source's first arg after `emit_unconditionally(&(11, 99))`. Reuse the existing `Sender2` (2-arg) and `Receiver` (1-arg) test fixtures already present in the test module. | `quartzite-core/src/connect.rs` `#[cfg(test)] mod tests` | 2 |
| 6 | Add `connect_signal_to_slot` tests for the meta-validated path: (a) `slot_arity_validation_truncates_args` — a source signal with 2 i32 params, a target object whose `lookup_method` advertises a 1-param `MethodMeta`, captures the first arg only; (b) `slot_arity_mismatch_returns_error` — 0-arg source → 1-arg meta'd slot, asserts `Err(ArityMismatch { from: 0, to: 1 })`; (c) `slot_type_mismatch_returns_error` — i32 source → bool meta'd slot, asserts `Err(TypeMismatch { index: 0, from: "i32", to: "bool" })`; (d) `slot_meta_absent_falls_back_to_empty_args` — target with `noop_lookup_method`, assert connect succeeds and the slot is invoked with `&[]` at emit time. Build a single `RecordingSlotRecv` test fixture (hand-rolled `Object` with `invoke_method` recording the received `&[Value]`) to cover (a), (b), (c), (d) by varying the static `MetaObject::lookup_method` per test. | `quartzite-core/src/connect.rs` `#[cfg(test)] mod tests` | 3 |
| 7 | Update the facade-crate integration test `tests/signal_to_signal.rs::connect_signal_to_signal_arity_mismatch_returns_error` to exercise the new rule (`from < to`): swap which signal is source and which is target so the test feeds the rejection path (`value_sent` → `clicked` is currently `from=1, to=0` which is **OK** under the new rule; flip to `clicked` → `value_received` so `from=0, to=1` triggers `ArityMismatch`). Add a sibling test `connect_signal_to_signal_truncates_extras` exercising the new positive path (1-arg source → 0-arg target): wire `value_sent` (i32) → `clicked` (no args), connect a counter to `clicked` on the relay side, emit `value_sent(42)`, assert the counter incremented. | `tests/signal_to_signal.rs` | 1 |
| 8 | Run the full gate: `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. Confirm AC22's `ac22_connect_signal_to_slot_stops_application` integration test still passes (the test wires a zero-arg source signal to a zero-arg meta'd `Application::quit`, so `from_arity (0) >= slot_arity (0)` succeeds and `&args[..0]` is `&[]` — semantics preserved). | (verification only) | 4, 5, 6, 7 |

## Handoff plan

`M = 8` → three groups, 3 + 3 + 2:

- **Handoff entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Group A:** subtasks 1–3 — production-source edits to all three connect entry points (signal-to-signal, typed, signal-to-slot). No tests touched yet; the implementation lands here and the test surface is unchanged at end of Group A (existing tests may temporarily fail because the old strict-equality semantics they encode now allow the previously-rejected `from >= to` cases — this is acceptable mid-group; Group B fixes them).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — unit-test surface updates. Repurpose the two old arity-mismatch tests (subtask 4) and add positive-path tests for signal-to-signal (subtask 4), typed (subtask 5), and signal-to-slot (subtask 6) including the meta-lookup fallback.
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group C with fresh context.
- **Group C:** subtasks 7–8 — terminal group (2 subtasks; within the 1..=3 range). Updates the facade-crate integration test, adds a positive sibling, runs the full verification gate (AC10).

## Risks

- **Existing `arity_mismatch_returns_error` tests no longer exercise the rejection path.** Both `connect.rs` unit tests (`from=1, to=0`) and the facade integration test (`value_sent (1) → clicked (0)`) currently feed inputs that are **valid** under the new rule. Subtasks 4 and 7 flip the source / target so the tests continue to exercise the rejection branch. *Mitigation:* enumerated in the respective subtasks; failure to flip would silently drop coverage of the `ArityMismatch` arm.

- **Queued / Auto-cross-thread closure captures `to_arity` and clones `args[..to_arity].to_vec()`.** The current closure captures `args.to_vec()` (full slice clone). After the change, the slice taken before `to_vec()` clones only the retained prefix, **reducing** allocation size — a strict improvement. *Mitigation:* none needed; just call out so the diff reviewer doesn't read it as a regression.

- **`emit_signal` validation on the target may re-reject truncated args.** Each `Object::emit_signal` impl (generated by `#[object_impl]` or hand-rolled) checks `args.len() == expected`. After truncation, `args.len() == to_arity` exactly, so the target's own arity check passes. *Mitigation:* assert covered by subtask 4's positive truncation test and subtask 7's facade test — both run `emit_signal` on a real target after truncation.

- **`SignalCallback` is `Box<dyn Fn(&[Value]) + Send + Sync>` — capturing `to_arity: usize` is `Copy` and `Send`, so no fn-trait constraint break.** *Mitigation:* none needed; only flagging for review.

- **Slot fallback test must use `noop_lookup_method` to hit the `None` branch.** A naively-built `MetaObject` test fixture might wire `lookup_method` to a non-noop closure that returns `None` for unknown names; that's still the `None` branch and the fallback test still works. The risk is the opposite: accidentally registering a `MethodMeta` for the test's slot name, which would push the test through the validated branch. *Mitigation:* subtask 6's fixture explicitly uses `noop_lookup_method` for the fallback case and a name-matching closure for the validated cases — separation called out in the subtask body.

- **AC22 integration test in `quartzite-runtime`** (`ac22_connect_signal_to_slot_stops_application`) connects `ClickSource::click` (zero-arg) → `Application::quit` (zero-arg, meta-registered). Under the new rule, this hits the **validated branch** with `from_arity (0) >= slot_arity (0)`, validates the empty `type_name` prefix (vacuously), and slices `&args[..0]` (== `&[]`). Behaviour-preserving. *Mitigation:* explicit verification in subtask 8.

- **Documentation drift in the `# Errors` block of `connect_signal_to_slot`.** The function gains `ArityMismatch` and `TypeMismatch` as new possible errors **only** on the validated branch. The doc must note the conditionality, e.g. "`ArityMismatch` / `TypeMismatch` are returned only when `target.meta_object().method(slot_name)` returns `Some(_)`." *Mitigation:* explicit in subtask 3's wording above; doc gate in subtask 8 catches any intra-doc-link breakage.

## Test Design

**Subtask 4 — repurposed `arity_mismatch_returns_error` (signal-to-signal)**

- *Location:* `quartzite-core/src/connect.rs` `#[cfg(test)] mod tests` (existing function).
- *Entry point:* `connect_signal_to_signal`.
- *Scenarios:*
  - Flip the existing test so a 0-arg source signal connects to a 1-arg target → `Err(ArityMismatch { from: 0, to: 1 })`.
  - Add `arity_relaxation_truncates_extras`: 1-arg source `(i32,)` → 0-arg target signal, assert `connect_signal_to_signal` returns `Ok`, then `sender.sig_a.emit_unconditionally(&(42,))` and verify the target's `emit_signal` was invoked by checking a counter (`AtomicBool` flag set true).
- *Fixtures:* The existing `NullRecv::emit_signal` is a no-op stub returning `None` — it does not fire the inner `_sig`. For `arity_relaxation_truncates_extras`, introduce a small `RecordingNullRecv` fixture whose `emit_signal` sets an `AtomicBool` directly when `signal == "_sig"`. `NullRecv` itself remains unchanged (it is used by the repurposed rejection test which only cares about connect-time error, not emit-time behaviour).

**Subtask 5 — `connect_signals` typed path truncation**

- *Location:* `quartzite-core/src/connect.rs` `#[cfg(test)] mod tests` (new test alongside `connect_signals_typed_direct_forwards`).
- *Entry point:* `connect_signals::<_, _, (i32, i32)>`.
- *Scenarios:* `Sender2` (2-arg `Signal<(i32, i32)>`) → `Receiver` (1-arg `Signal<(i32,)>`); emit `(11, 99)`; assert captured value is `11` (first arg retained, second dropped).
- *Fixtures:* `Sender2` and `Receiver` already exist in the test module.

**Subtask 6 — `connect_signal_to_slot` four scenarios**

- *Location:* `quartzite-core/src/connect.rs` `#[cfg(test)] mod tests`.
- *Entry point:* `connect_signal_to_slot`.
- *Fixture:* `RecordingSlotRecv` — hand-rolled `Object` whose `invoke_method` records `args.to_vec()` into an `Arc<parking_lot::Mutex<Vec<Value>>>` (workspace default; do NOT use `std::sync::Mutex`). Two static `MetaObject` instances differ only in their `lookup_method`:
  - `RECV_META_VALIDATED_1ARG` — `lookup_method` returns `Some(MethodMeta::new("on_click", &[ParamMeta::new("v", "i32")], "()"))` for `"on_click"`.
  - `RECV_META_VALIDATED_BOOL` — same but `"bool"` in the `ParamMeta`.
  - `RECV_META_FALLBACK` — `noop_lookup_method` (matches existing test fixtures).
- *Scenarios:*
  - **(a) `slot_arity_validation_truncates_args`:** source 2-arg signal `(i32, i32)` (use `Sender2`) → `RECV_META_VALIDATED_1ARG` slot named `"on_click"`. Emit `(7, 8)`. Assert recorded args is `vec![Value::Int(7)]` (length 1, first arg only).
  - **(b) `slot_arity_mismatch_returns_error`:** source 0-arg signal (use a zero-arg sender, similar shape to `NullRecv`) → `RECV_META_VALIDATED_1ARG`. Expect `Err(ArityMismatch { from: 0, to: 1 })`.
  - **(c) `slot_type_mismatch_returns_error`:** source 1-arg `(i32,)` (use `Sender`) → `RECV_META_VALIDATED_BOOL`. Expect `Err(TypeMismatch { index: 0, from: "i32".into(), to: "bool".into() })`.
  - **(d) `slot_meta_absent_falls_back_to_empty_args`:** source 1-arg `(i32,)` (use `Sender`) → `RECV_META_FALLBACK`. Connect succeeds. Emit `(42,)`. Assert recorded args is `vec![]` (fallback path used `&[]`).

**Subtask 7 — facade `signal_to_signal.rs` integration**

- *Location:* `tests/signal_to_signal.rs`.
- *Entry point:* `connect_signal_to_signal` via the facade prelude.
- *Scenarios:*
  - Flip the existing `connect_signal_to_signal_arity_mismatch_returns_error` to use `clicked` (0-arg) as source and `value_received` (1-arg) as target → `Err(ArityMismatch)`.
  - New `connect_signal_to_signal_truncates_extras`: connect `value_sent` (1-arg) → `clicked` (0-arg). Bind a counter to the relay's `clicked` signal. Emit `value_sent(42)`. Assert counter incremented (value irrelevant; what matters is that the zero-arg target signal fired, proving truncation worked end-to-end through the facade re-export).
- *Fixtures:* `Emitter` and `Relay` already exist; both already declare a `clicked: Signal<()>` field.

**Subtask 8 — gate**

- Full workspace `cargo build` / `cargo test` / `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt -- --check` / `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` / `cargo build -p quartzite --no-default-features --features libm`.
- Spot-verify `ac22_connect_signal_to_slot_stops_application` in `quartzite-runtime/tests/connect_signal_to_slot.rs` still passes (it now goes through the meta-validated branch with `from_arity (0) >= slot_arity (0)`, validates the vacuous empty prefix, slices `&args[..0]` == `&[]`).

## Open questions

- _(none — spec is exhaustive, ArityMismatch rename rejected with justification above)_
