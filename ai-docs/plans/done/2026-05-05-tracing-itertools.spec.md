# feat(deps): integrate tracing instrumentation and itertools test helpers

**Source:** user description
**Date:** 2026-05-05
**Tracked in:** #89

## Scope

- Add `tracing = "0"` to `quartzite-runtime` and instrument key operations: `EventLoop::post`/`stop`, `ObjectTree::insert`/`destroy`/`rename`/`reparent`, queued signal dispatch in `ConnectionTable`
- Add `tracing = { version = "0", default-features = false }` to `quartzite-core` and instrument `Signal::emit`, `connect`, `connect_queued`, `connect_auto`, `disconnect`
- Add `itertools = "0"` as dev-dep to `quartzite-core` and `quartzite-runtime`; simplify two identified tests
- Update `ai-docs/context.md`: document futures-util deferred decision under Open Questions

## Out of scope

- `futures-util` — no async call sites; async/await strategy is an open design question (deferred)
- `clap` / `clap_complete` — no CLI surface in the library
- Changing production iterator patterns to use itertools
- Adding `tracing-subscriber` or any concrete subscriber — that is the application's responsibility

## Deferred

- Enabling the `std` tracing feature conditionally via `quartzite-core`'s `std` feature flag | straightforward but adds Cargo feature plumbing; low value for now | no separate issue needed
- `futures-util` integration | blocked on async strategy decision | reuse #89 or open a new issue when async strategy is settled

## Key decisions

| Question | Decision |
|---|---|
| `futures-util` | Not added — no call sites; async strategy deferred |
| `tracing` in `quartzite-core` | Add with `default-features = false` (no_std compatible); macros are no-ops until a subscriber is installed |
| `tracing` in `quartzite-runtime` | Add with default features (std); full instrumentation |
| `itertools` scope | Dev-dep only; no production use at this time |
| Subscriber setup | Out of scope — library emits spans/events; callers configure the subscriber |

## Technical constraints

- `quartzite-core` is `no_std + alloc`; `tracing` must be added with `default-features = false`
- `quartzite-geometry` and `quartzite-events` are also no_std — do **not** add tracing there (no meaningful call sites)
- `cargo build -p quartzite --no-default-features` must remain clean after the change
- Dependency version rules from AGENTS.md: `0.x` form for 0.x.y versions; run `cargo update` after adding

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-runtime/Cargo.toml` contains `tracing = "0"` under `[dependencies]` |
| AC2 | `quartzite-core/Cargo.toml` contains `tracing = { version = "0", default-features = false }` under `[dependencies]` |
| AC3 | `quartzite-core/Cargo.toml` and `quartzite-runtime/Cargo.toml` each contain `itertools = "0"` under `[dev-dependencies]` |
| AC4 | `EventLoop::post` and `EventLoop::stop` each emit at least one `tracing::trace!` or `tracing::debug!` call |
| AC5 | `ObjectTree::insert`, `destroy`, `rename`, and `reparent` each emit at least one `tracing::trace!` or `tracing::debug!` call |
| AC6 | `Signal::emit`, `connect`, and `disconnect` each emit at least one `tracing::trace!` call |
| AC7 | `find_by_name_returns_all_with_same_name` (object_tree unit test) is simplified using `itertools::Itertools::sorted_unstable_by_key` — the three-assertion pattern (`contains` × 2 + `len`) replaced by a single `assert_equal` on sorted iterators (`ObjectId` lacks `Ord`, so key-based sort on `id.raw()` is required) |
| AC8 | `destroy_is_depth_first_post_order` (object_tree integration test) position-comparison chain replaced using `tuple_windows().all(|(a, b)| a < b)` |
| AC9 | `cargo build -p quartzite --no-default-features` compiles clean |
| AC10 | `ai-docs/context.md` Open Questions section updated: `futures-util` deferred decision documented |

## Open questions

_None._
