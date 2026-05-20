# clippy: narrow type_complexity to per-item allows or type aliases

**Source:** issue #482
**Date:** 2026-05-20
**Tracked in:** #482

## Scope

1. Remove the workspace-level allow `type_complexity = "allow"` (and the `# 5 hits in runtime/object_tree.rs (3) + renderer/wrapped_handler.rs (2); …` justifying comment immediately above it) from the root `Cargo.toml`'s `[workspace.lints.clippy]` table.
2. Resolve each of the 5 lint sites enumerated in the issue body per the per-site strategy in **Key decisions → narrowing strategy**:
   - **`quartzite-runtime/src/object_tree.rs` trio — type alias.** All three sites share the same `Arc<Mutex<Vec<(String, Vec<Value>)>>>` shape (the emissions log captured by the `RecordingObject` test fixture). Introduce a single test-module type alias (working name `EmissionLog`, final name a design-phase detail) and use it in all three positions:
     - `:769` — `struct RecordingObject { …, emissions: <alias> }` field type.
     - `:775` — `fn named(name: &str) -> (Box<dyn Object>, <alias>)` return type.
     - `:787` — `fn anonymous() -> (Box<dyn Object>, <alias>)` return type.
   - **`quartzite-renderer/src/wrapped_handler.rs` pair — per-item `#[allow]`.** Each tuple has a one-off shape (no other site reuses it), so a type alias would not reduce duplication. Apply `#[allow(clippy::type_complexity, reason = "…")]` at item level:
     - `:286` — `impl CountingRoot { fn new() -> (Self, Arc<Mutex<Vec<Size>>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>) }` inside `#[cfg(test)] mod tests`.
     - `:453` — `fn make_handler_with_root(quit: bool) -> (WrappedHandler<NoopHandler>, WinitWindowId, Arc<Mutex<u32>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>)` inside `#[cfg(test)] mod tests`.
3. All 5 sites live inside `#[cfg(test)] mod tests` blocks — they are test-fixture helpers, not production API. The narrowing applies to test-only code; no production signatures are touched.

## Out of scope

- Refactoring the test fixtures themselves (e.g., replacing the multi-tuple returns with builder structs that own the channels). The fixtures are short-lived test helpers; the lint is the only concern.
- Touching any other workspace allow in `[workspace.lints.clippy]`. The remaining 3 entries (`must_use_candidate`, `redundant_pub_crate`, `return_self_not_must_use`) are unrelated and out of scope here.
- Changing the workspace lint level for `clippy::type_complexity` (it inherits from `clippy::all = warn` once the explicit `allow` is dropped, escalated to deny by CI `-D warnings` — the desired posture).
- Extending the `object_tree.rs` type alias to the `wrapped_handler.rs` sites — different concrete `Mutex<…>` inner types (`Vec<(String, Vec<Value>)>` vs `Vec<Size>`/`u32`); sharing would force an over-generic alias.
- Resolving the issue body's wording that lumps all 5 sites under "Sites" without distinguishing the runtime trio (shared shape) from the renderer pair (one-offs). The spec records the distinction in § Scope; no follow-up needed.

## Deferred

- None.

## Key decisions

| Question | Decision |
|---|---|
| Narrowing strategy: per-site `#[allow]` vs `type` aliases? | **Mixed strategy.** Type alias for the `object_tree.rs` trio because all three share the exact `Arc<Mutex<Vec<(String, Vec<Value>)>>>` shape — the alias eliminates the lint trigger and removes shape duplication. Per-item `#[allow(clippy::type_complexity, reason = "…")]` for the two `wrapped_handler.rs` tuples because each is a one-off shape; an alias used in a single position would add indirection without reducing duplication. (Round-1 user answer.) |
| Reason-string wording for the two `#[allow]` sites? | Design-phase detail. Each `#[allow]` carries a `reason = "…"` naming the fixture's role (e.g., "test fixture: returns multi-Arc<Mutex<…>> channels for assertion read-back from event-handler test"). |
| `#[allow]` placement on the renderer sites? | Item-level on the `fn` (matches the PR #443 / #503 narrowing precedent for fn-return-shape sites). |
| Type-alias placement for the runtime trio? | Inside the same `#[cfg(test)] mod tests` block as the `RecordingObject` fixture, at the top of the module — test-only scope, no leakage to production namespace. Alias name is a design-phase choice (working name `EmissionLog`). |

## Technical constraints

- `cargo clippy --workspace --all-targets -- -D warnings` is the binding verification. The `--all-targets` is load-bearing because every site is inside `#[cfg(test)]` and only compiles under `--tests`.
- `[workspace.lints.clippy]` and per-crate `[lints.clippy]` tables are mutually exclusive under Cargo's lint inheritance model. The `#[allow]` narrows on the renderer sites MUST therefore be in-source `#[allow(..., reason = "…")]` attributes (the same constraint PR #443 documented).
- `reason = "…"` requires Rust 1.81+. The workspace `rust-version` is comfortably past that.
- Type aliases inside `#[cfg(test)]` modules do not affect production code generation, public API surface, or doc gates.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | The line `type_complexity = "allow"` and its `# 5 hits in runtime/object_tree.rs (3) + renderer/wrapped_handler.rs (2); …` comment on the immediately-preceding line are removed from root `Cargo.toml`'s `[workspace.lints.clippy]` table. `grep -n 'type_complexity' Cargo.toml` returns no hits. |
| AC2 | The three `object_tree.rs` sites (`:769` field, `:775` return, `:787` return) all reference a single new test-module type alias defined inside the same `#[cfg(test)] mod tests` block. `grep -c 'Arc<Mutex<Vec<(String, Vec<Value>)>>>' quartzite-runtime/src/object_tree.rs` returns 0 (every instance replaced by the alias). |
| AC3 | The two `wrapped_handler.rs` sites (`:286` and `:453`) each carry an item-level `#[allow(clippy::type_complexity, reason = "…")]` attribute with a non-empty, site-specific reason string (not a copy-paste placeholder). |
| AC4 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0. |
| AC5 | `cargo build` exits 0 and `Cargo.lock` is refreshed before commit per AGENTS.md § Workflow. |
| AC6 | `cargo test --workspace` exits 0 — no behavioural change is intended; this is a pure lint-suppression narrow. |
| AC7 | `cargo fmt -- --check` exits 0. Multi-line `#[allow]` attributes and the type alias follow existing repo style. |

## Open questions

- None.
