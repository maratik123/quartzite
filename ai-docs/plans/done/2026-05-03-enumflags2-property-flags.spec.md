# Replace PropertyFlags bool-struct with enumflags2 bitfield

**Source:** user description
**Date:** 2026-05-03
**Tracked in:** #69

## Scope
- Add `enumflags2` dependency to `quartzite-core`
- Define `#[bitflags] #[repr(u8)] pub enum PropertyFlag` with 7 variants: `Readable`, `Writable`, `Notify`, `Stored`, `Designable`, `User`, `Constant`
- Expose `pub type PropertyFlags = BitFlags<PropertyFlag>` as the public composite type
- Re-export both `PropertyFlag` and `PropertyFlags` from `quartzite-core` public API
- Move named constructors (`none()`, `read_write()`, `read_only()`) to `impl PropertyFlag` returning `BitFlags<Self>`
- Keep `Default` returning `PropertyFlag::read_write()`
- Update proc-macro codegen (`quartzite-macros`) to construct `BitFlags` from per-field booleans
- Update all field-access sites in the codebase (`flags.readable` → `flags.contains(PropertyFlag::Readable)`)

## Out of scope
- Serialization / deserialization (serde) support
- FFI / integer wire format
- Any changes to other metadata types (`SignalMeta`, `MethodMeta`, etc.)

## Deferred
- `from_bits` / integer round-trip API | not needed today; can be added later

## Key decisions
| Question | Decision |
|---|---|
| Breaking change acceptable? | Yes |
| `PropertyFlag` exposed as public API? | Yes |
| Type alias or newtype for `PropertyFlags`? | Type alias: `pub type PropertyFlags = BitFlags<PropertyFlag>` |
| Named constructors location? | `impl PropertyFlag` — methods returning `BitFlags<Self>` |
| `enumflags2` version? | Latest stable; verify `const` support and `no_std` compatibility |

## Technical constraints
- `quartzite-core` must remain `no_std` compatible — `enumflags2` supports this
- `PropertyMeta::new` is `const fn` and used in `static` initialisers — named constructors must also be `const fn`
- Proc-macro codegen in `quartzite-macros` constructs `PropertyFlags` via struct literal today; must switch to `BitFlags`-compatible construction
- `enumflags2` requires `#[repr(u8)]` (or other integer repr) on the enum

## Acceptance Criteria
| # | Criterion |
|---|-----------|
| AC1 | `PropertyFlag` is a public `#[bitflags] #[repr(u8)]` enum in `quartzite-core` with variants `Readable`, `Writable`, `Notify`, `Stored`, `Designable`, `User`, `Constant` |
| AC2 | `PropertyFlags` is a public type alias `BitFlags<PropertyFlag>` exported from `quartzite-core` |
| AC3 | `PropertyFlag::none()`, `PropertyFlag::read_write()`, `PropertyFlag::read_only()` are `const fn` methods returning `BitFlags<Self>` with the same flag combinations as the old struct constructors |
| AC4 | `Default` for `PropertyFlags` returns `PropertyFlag::read_write()` |
| AC5 | Proc-macro codegen produces valid `BitFlags<PropertyFlag>` values — existing `#[derive(Object)]` / `#[object_impl]` usage compiles without change to user code |
| AC6 | All existing test assertions that checked individual bool fields pass against the new `contains()`-based API |
| AC7 | `cargo build -p quartzite --no-default-features` compiles clean (no_std path) |
| AC8 | `cargo clippy -- -D warnings` passes with no new warnings |

## Open questions
- None
