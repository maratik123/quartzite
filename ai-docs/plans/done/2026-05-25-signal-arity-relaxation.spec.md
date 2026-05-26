# Signal arity relaxation (signal-to-slot / signal-to-signal)

**Source:** issue #566
**Date:** 2026-05-25
**Tracked in:** #566

## Background

Issue #566 ("How arity check works?") asks: *"Can we connect signal with (T) as args to slot with () as args?"*

Current behaviour in `quartzite-core/src/connect.rs`:

- **`connect_signal_to_slot(source, signal_name, target, slot_name)`** — always invokes the target slot with `&[]` (empty args) regardless of the source signal's arity or the slot's declared parameters. Type-checking on the slot side is **not performed**; the slot is treated as zero-arg. Non-zero-arity slots silently no-op (documented).
- **`connect_signal_to_signal` / `connect_signals` (typed)** — require **strict arity equality** between source signal and target signal, returning `SignalConnectionError::ArityMismatch { from, to }` when arities differ.

The Qt-style "signal with more args → slot/signal with fewer args, excess silently dropped" model is therefore *partially* supported today: it works for `connect_signal_to_slot` (because every slot is treated as zero-arg), but does **not** work for signal-to-signal forwarding.

Round 1 decision (user, "Both 2 & 3"):

- **Generalise arity-relaxation to signal-to-signal forwarding**: allow `from_arity >= to_arity`, truncate extras at emit time.
- **Tighten `connect_signal_to_slot`**: validate the slot's declared arity (currently unchecked) instead of always invoking with `&[]`.

## Scope

1. **Signal-to-signal arity relaxation** in `connect_signal_to_signal`, `connect_signals` (typed), and any other typed signal-to-signal forwarding path:
   - Replace strict `from_arity != to_arity` check with `from_arity < to_arity` rejection only.
   - When `from_arity >= to_arity`, validate `type_name` equality on the **first `to_arity` parameters** (retained prefix).
   - At emit time, the forwarding callback passes `&args[..to_arity]` (signal-to-signal) or projects the typed `Args` tuple down to the retained prefix (typed path — via the same `ArgsToValues::to_values()` slicing).
2. **Slot arity validation** in `connect_signal_to_slot`:
   - At connection time, look up the slot's `MethodMeta` via `target.lock().meta_object().method(slot_name)`. When present, enforce `from_arity >= slot_arity` and validate `type_name` equality on the retained prefix.
   - At emit time, invoke the slot with `&args[..slot_arity]` (sliced to the declared method arity) instead of `&[]`.
3. **Error variants**:
   - Repurpose `SignalConnectionError::ArityMismatch { from, to }` to mean "source signal has fewer parameters than required by target" (the only remaining failure mode). The `from` / `to` field names stay; semantics tighten from "≠" to "<".
4. **Slot meta lookup fallback** (preserves current "lazy / no-op" semantics for hand-rolled objects whose `invoke_method` is not advertised via the meta system):
   - When `meta_object().method(slot_name)` returns `None`, **fall back to current behaviour**: invoke the slot with `&[]` at emit time (silent no-op for unknown / non-zero-arity slots). Connection-time arity validation is **skipped** in this branch.
   - This preserves the documented escape hatch for objects that hand-roll `invoke_method` without a meta entry, while making the common (meta-registered) path strictly validated.
5. Documentation refresh on all three connect APIs reflecting the new contract (rustdoc + `# Errors` blocks).
6. Tests covering: (a) relaxed signal-to-signal forwarding with truncation, (b) slot arity validation against `MethodMeta`, (c) `ArityMismatch` for signal-arity < target-arity in both paths, (d) `TypeMismatch` on the retained prefix, (e) meta-less slot fallback still no-ops.

## Out of scope

- Cycle detection (still documented as caller responsibility).
- New error variants beyond repurposing `ArityMismatch` (e.g. no `InsufficientArity` rename; see Key Decisions for the naming choice if the design subagent prefers a clearer name).
- Slot **return-type** propagation (slots remain fire-and-forget; return type is ignored).
- Reordering / projection (only **prefix truncation** is in scope; no `(a, b, c) → (a, c)` slot adapter).

## Deferred

- `(a, b, c) → (a, c)` projection adapters — separate issue would be needed if/when use cases arise; not in scope here.
- Return-type checking for slots — slots are fire-and-forget today.

## Key decisions

| Question | Decision |
|---|---|
| Truncation semantics | Prefix truncation only: `args[..to_arity]` (signal-to-signal) / `args[..slot_arity]` (slot). No reordering / projection. |
| Type-name validation on retained prefix | Enforced — identical to current behaviour, just applied to the first `to_arity` / `slot_arity` parameters. |
| Slot meta lookup fallback | When `MethodMeta` is absent, fall back to current "silent / `&[]`" semantics. Validation only fires when meta info is available. |
| `ArityMismatch` variant fate | Repurposed: now means "source signal has fewer parameters than target slot/signal requires". Field names `from` / `to` stay. The `design` Subagent MAY propose a rename (e.g. `InsufficientArity`) per AGENTS.md API Naming if it improves clarity — pre-publish rename is free. |
| Typed path (`connect_signals<Args>`) interaction with truncation | Truncation happens through `ArgsToValues::to_values()` slicing on the emit side — same call site, different slice bound. No new generic parameter required. |

## Technical constraints

- Pre-publish workspace: arity-relaxation changes the contract of existing `connect_*` functions without shims (AGENTS.md § *API Stability*).
- Arity checks live in `quartzite-core/src/connect.rs` for signal-to-signal (lines ~129–137 and ~368–376); slot path is in the same file (`connect_signal_to_slot` ~lines 259–283).
- Argument truncation must preserve `type_name` checks on the *retained* prefix (signal-to-signal already validates `type_name` per position via `params` zip — the zip just becomes bounded).
- `SignalCallback` is `Box<dyn Fn(&[Value])>` — args arrive as a slice, so truncation at emit time is a sub-slice on the receiving end (`&args[..n]`).
- `connect_signal_to_slot` currently does not look up `MethodMeta::params` for the target slot — extending it to validate arity needs `meta_object().method(slot_name)` (this function exists; see `quartzite-core/src/meta.rs` line 808).
- `MethodMeta` has `params: &'static [ParamMeta]` with `type_name`, fully usable for the same validation logic as `SignalMeta`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `connect_signal_to_signal` returns `Ok` when `from_arity >= to_arity` with matching `type_name` on the first `to_arity` parameters; the forwarding callback emits the target signal with `&args[..to_arity]`. |
| AC2 | `connect_signal_to_signal` returns `Err(ArityMismatch { from, to })` when `from_arity < to_arity`. |
| AC3 | `connect_signal_to_signal` returns `Err(TypeMismatch { index, .. })` when any `type_name` differs in the first `to_arity` positions. |
| AC4 | `connect_signals` (typed) behaves identically to AC1/AC2/AC3 with truncation realised through `ArgsToValues::to_values()` slicing. |
| AC5 | `connect_signal_to_slot`, given a `target` whose `meta_object().method(slot_name)` returns `Some(meta)`, returns `Ok` when `from_arity >= meta.params.len()` with matching `type_name` on the retained prefix; the slot is invoked with `&args[..meta.params.len()]`. |
| AC6 | `connect_signal_to_slot` returns `Err(ArityMismatch { from, to })` when `from_arity < slot_arity` (meta path). |
| AC7 | `connect_signal_to_slot` returns `Err(TypeMismatch { .. })` on type-name mismatch in the retained prefix (meta path). |
| AC8 | `connect_signal_to_slot`, given a `target` whose `meta_object().method(slot_name)` returns `None`, falls back to current "silent / `&[]`" behaviour — `Ok` is returned at connect time, slot is invoked with `&[]` at emit time. |
| AC9 | All three rustdoc bodies (`connect_signal_to_signal`, `connect_signals`, `connect_signal_to_slot`) reflect the new contract incl. updated `# Errors` blocks. |
| AC10 | `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all pass. |

## Open questions

- None outstanding — design subagent owns the rest (rename of `ArityMismatch`, typed-path implementation detail, test layout).
