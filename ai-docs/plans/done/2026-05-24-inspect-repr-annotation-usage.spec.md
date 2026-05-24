# Inspect repr annotation usage

**Source:** issue #558
**Date:** 2026-05-24
**Tracked in:** #558

## Scope

1. Audit every `#[repr(...)]` annotation on user-facing enums in the workspace and classify each as either **load-bearing** (required by an external contract — macro, FFI, on-disk format) or **gratuitous** (decorative; could be dropped or narrowed without behavioural change).
2. Strip every gratuitous `#[repr(...)]` annotation; narrow any over-wide load-bearing annotation that exceeds the minimum width fitting its variants.
3. Verify the workspace builds clean (`cargo build`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) after the changes.

The 10 annotated enums in scope (initial inventory — `rg '#\[repr\(' --type rust -n`):

| # | File | Enum | Current `#[repr]` | Bound by |
|---|---|---|---|---|
| 1 | `quartzite-paint-api/src/font.rs:249` | `FontWeight` | `u16` | OpenType `usWeightClass` is `u16`; one `as u16` call site (`vello_painter.rs:432` feeds `f32::from(_)` losslessly) |
| 2 | `quartzite-widgets/src/enums.rs:15` | `FocusPolicy` | `i64` | None — `MetaEnum` codegen does `self as i64` regardless of repr (see `quartzite-macros/src/meta_enum/codegen.rs:89`) |
| 3 | `quartzite-widgets/src/enums.rs:38` | `SizePolicy` | `i64` | None — same as `FocusPolicy` |
| 4 | `quartzite-widgets/src/enums.rs:63` | `CursorShape` | `i64` | None — same as `FocusPolicy` |
| 5 | `quartzite-widgets/src/widgets/scroll_area.rs:18` | `ScrollPolicy` | `i64` | None — same as `FocusPolicy` |
| 6 | `quartzite-widgets/src/widget_base.rs:29` | `WidgetState` | `u8` | `enumflags2::bitflags` macro requires `#[repr(uN)]`; 6 variants → `u8` is minimum |
| 7 | `quartzite-geometry/src/h_alignment.rs:22` | `HAlignment` | `i64` | None — same as `FocusPolicy` (sibling `VAlignment` already lacks `#[repr]` and works) |
| 8 | `quartzite-events/src/mouse.rs:25` | `MouseButton` | `u8` | `enumflags2::bitflags`; 5 variants → `u8` is minimum |
| 9 | `quartzite-events/src/keyboard.rs:22` | `KeyModifier` | `u8` | `enumflags2::bitflags`; 4 variants → `u8` is minimum |
| 10 | `quartzite-core/src/meta.rs:21` | `PropertyFlag` | `u8` | `enumflags2::bitflags`; 7 variants → `u8` is minimum |

## Out of scope

- `#[repr(C)]` / `#[repr(transparent)]` on structs (none exist in current source for this concern; this issue is enum-repr focused).
- Adding new enums or restructuring `MetaEnum` / `enumflags2` macro contracts.
- Changing `MetaEnum`'s widening to `i64` inside `IntoValue::into_value` — the wire format (`Value::Int(i64)`) is independent of the source-enum repr.
- Auditing third-party crates' repr choices (e.g. `parley::FontWeight`).

## Deferred

- (none)

## Key decisions

| Question | Decision |
|---|---|
| Why is `#[repr(T)]` needed in each case? | Two distinct reasons: (a) **`enumflags2::bitflags` requires `#[repr(uN)]`** — without it the macro fails to compile (`WidgetState`, `MouseButton`, `KeyModifier`, `PropertyFlag`); (b) **external numeric contract** — `FontWeight` carries OpenType-spec discriminants (100..900) and is converted into a `parley::FontWeight` numeric value via `as u16`. Everywhere else the `#[repr(...)]` is decorative and the issue's premise stands. |
| Why does `T` vary case to case? | The `u8` group is sized to the bitflags variant count (each variant claims one bit; ≤ 8 variants ⇒ `u8` suffices). `FontWeight` uses `u16` because OpenType `usWeightClass` is `u16` AND the max value (900) exceeds `u8::MAX` (255). The `i64` group is uniform but historical: every `MetaEnum` discriminant was widened to `i64` to match the `Value::Int(i64)` wire format. That widening happens inside the macro-generated `IntoValue::into_value` (`self as i64`) and does not require the source enum to BE `#[repr(i64)]`. |
| For the 5 `#[repr(i64)]` `MetaEnum` enums, strip entirely or swap to a narrower explicit `#[repr(uN)]`? | **Strip entirely.** Sibling enum `VAlignment` (`quartzite-geometry/src/v_alignment.rs`) already uses `#[derive(MetaEnum)]` with **no** `#[repr]` annotation and works correctly — proves the macro requires no repr. AGENTS.md *API Stability* AXIOM (pre-publish, no downstream clients) means ABI stability is not a constraint that would justify an explicit `#[repr(u8)]`. Letting the compiler pick (a 1-byte representation for ≤ 256 variants) maximises storage suppression in containing structs (`WidgetBase`, `ScrollArea`, future style structs) at zero source-code cost. Per-enum savings: 7 bytes (8 → 1) on a 64-bit platform, before struct layout / padding effects. |
| `FontWeight` — keep `#[repr(u16)]`, narrow to `u8`, or drop? | **Keep `#[repr(u16)]`.** (a) `u8::MAX` is 255 — max discriminant `900` does not fit `u8`. (b) `u16` matches the OpenType `usWeightClass` external spec, making the cast at `vello_painter.rs:432` (`f32::from(font.weight() as u16)`) a documented identity. (c) `f32::From<u16>` is infallible, so the `as u16` cast is the lossless idiom. No change. |
| `enumflags2 u8` group (`WidgetState`, `MouseButton`, `KeyModifier`, `PropertyFlag`) — change? | **No change.** `#[repr(uN)]` is mandatory for `enumflags2::bitflags`; `u8` is already the minimum width for the variant counts (4 / 5 / 6 / 7 variants, all ≤ 8). Narrower than `u8` is not a Rust integer type. |
| Doc-test / unit-test impact of stripping `#[repr(i64)]`? | None. `as i64` casts on a default-repr enum with variants ≥ 0 widen losslessly. Every existing doc-test assertion (`assert_eq!(FocusPolicy::NoFocus as i64, 0)` and siblings) and every `into_value()` / `from_value()` round-trip continues to pass. The macro-emitted `self as i64` cast (codegen.rs:89) compiles for any integer-repr enum. |

## Technical constraints

- Workspace lints are strict (`-D warnings` + `missing_docs = deny` + `rustdoc::broken_intra_doc_links = deny`); the changes must not introduce a single warning.
- The `MetaEnum` derive does NOT require `#[repr(...)]` (confirmed by `VAlignment` precedent). The macro's widening to `i64` (for `Value::Int`) happens at the cast site, not at the type-layout level.
- `enumflags2::bitflags` DOES require `#[repr(uN)]` where `uN ∈ {u8, u16, u32, u64}`. Cannot be removed without breaking the derive.
- `FontWeight` discriminants 100..900 do not fit in `u8`. `u16` is the minimum width. Also matches OpenType `usWeightClass` spec width.
- All edits stay within source files — no `Cargo.toml`, dependency, or workflow changes are required.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | The five `#[repr(i64)]` annotations on `MetaEnum`-derived enums are removed: `FocusPolicy`, `SizePolicy`, `CursorShape` (all in `quartzite-widgets/src/enums.rs`), `ScrollPolicy` (`quartzite-widgets/src/widgets/scroll_area.rs`), and `HAlignment` (`quartzite-geometry/src/h_alignment.rs`). No replacement `#[repr]` is added — the compiler picks the tightest representation. |
| AC2 | The four `#[bitflags] #[repr(u8)]` annotations remain unchanged: `WidgetState`, `MouseButton`, `KeyModifier`, `PropertyFlag`. |
| AC3 | `FontWeight`'s `#[repr(u16)]` remains unchanged. |
| AC4 | `cargo build --workspace` succeeds with zero warnings. |
| AC5 | `cargo test --workspace` passes — including the existing doc-tests that cast `FocusPolicy::NoFocus as i64`, `HAlignment::Left as i64`, etc., and the `discriminants_match_legacy_widget_alignment` unit test (`quartzite-geometry/src/h_alignment.rs:46`). |
| AC6 | `cargo clippy --workspace --all-targets -- -D warnings` passes. |
| AC7 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes. |
| AC8 | `cargo build -p quartzite --no-default-features --features libm` passes (no_std / derive-free path). |
| AC9 | A grep for `#[repr(` against `**/*.rs` in the workspace returns exactly five hits after the change: `FontWeight` (u16), `WidgetState` (u8), `MouseButton` (u8), `KeyModifier` (u8), `PropertyFlag` (u8). |
| AC10 | The doc-comment lines `/// assert_eq!(FocusPolicy::NoFocus as i64, 0);`, `/// assert_eq!(SizePolicy::Fixed as i64, 0);`, `/// assert_eq!(CursorShape::Arrow as i64, 0);`, `/// assert_eq!(HAlignment::Left as i64, 0);` remain valid (the `as i64` cast on a default-repr unit enum with non-negative variants compiles and the value is unchanged). No edit to those doc-tests is required to make them pass. |
| AC11 | A `#[repr]` usage rule is added to `ai-docs/code-style.md` documenting when `#[repr(...)]` is required on enums vs when it must be omitted: `enumflags2::bitflags` requires `#[repr(uN)]`, external numeric contracts (e.g. OpenType spec) justify explicit `#[repr]`, and no other case warrants it in this codebase. |

## Open questions

- (none)
