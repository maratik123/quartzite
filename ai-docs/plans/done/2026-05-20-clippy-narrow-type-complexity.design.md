# Design: clippy — narrow `type_complexity` to type alias + per-item allows

**Issue:** #482
**Date:** 2026-05-20

## Approach

**Chosen solution.** Apply the mixed strategy pre-resolved in the spec § Key decisions:

1. In `quartzite-runtime/src/object_tree.rs` (3 sites that share the exact `Arc<Mutex<Vec<(String, Vec<Value>)>>>` shape), introduce a single test-module type alias and replace every occurrence with that alias.
2. In `quartzite-renderer/src/wrapped_handler.rs` (2 sites with distinct one-off multi-arc tuple return shapes), add an item-level `#[allow(clippy::type_complexity, reason = "…")]` on each `fn`.
3. Remove `type_complexity = "allow"` and its `# 5 hits in runtime/object_tree.rs (3) + renderer/wrapped_handler.rs (2); …` justifying comment from root `Cargo.toml`'s `[workspace.lints.clippy]` table.

The lint then inherits `clippy::all = warn` and is escalated to deny by CI's `-D warnings`. The runtime trio is silenced by the alias eliminating the lint trigger (rustfmt-and-clippy-stable: the alias reduces the type-complexity score below the default threshold for that line). The renderer pair is silenced by the narrow `#[allow]`s with site-specific reason strings, following the same reason-string discipline established by PR #443 and reused by the sibling task PR #503 (`significant_drop_in_scrutinee` narrowing).

**Site classification.**

| Site | Shape | Strategy | Why |
|---|---|---|---|
| `object_tree.rs:769` (field) | `Arc<Mutex<Vec<(String, Vec<Value>)>>>` | Type alias | Shared across 3 sites — alias removes duplication AND lint trigger. |
| `object_tree.rs:775` (`fn named` return) | `(Box<dyn Object>, Arc<Mutex<Vec<(String, Vec<Value>)>>>)` | Type alias (second position of tuple) | Same. |
| `object_tree.rs:787` (`fn anonymous` return) | `(Box<dyn Object>, Arc<Mutex<Vec<(String, Vec<Value>)>>>)` | Type alias (second position of tuple) | Same. |
| `wrapped_handler.rs:286` (`CountingRoot::new` return) | `(Self, Arc<Mutex<Vec<Size>>>, Arc<Mutex<u32>>, Arc<Mutex<u32>>)` | Item-level `#[allow]` | One-off shape; alias used in exactly one position would add indirection without reducing duplication. |
| `wrapped_handler.rs:453` (`make_handler_with_root` return) | `(WrappedHandler<NoopHandler>, WinitWindowId, Arc<Mutex<u32>>×4)` | Item-level `#[allow]` | One-off shape; same rationale. |

**Type-alias placement.** Inside the `#[cfg(test)] mod tests { … }` block at `quartzite-runtime/src/object_tree.rs:450`. Place the alias immediately above the `RecordingObject` struct definition (i.e. just before the `// --- RecordingObject: captures emit_signal calls for signal emission tests ---` divider comment at line 765, or directly between that divider and the `struct RecordingObject` line — either ordering is acceptable to rustfmt; the divider stays attached to the fixture region). Test-only scope, no leakage to production namespace.

**Type-alias name.** `EmissionLog` (working name from spec § Key decisions row 4; promoted to the final name here — it accurately describes the captured emissions from `emit_signal` calls and matches the existing `emissions:` field name + the `let log = …` local variable convention already used inside the constructors).

**`#[allow]` placement on the renderer sites.** Item-level on the `fn` itself, matching the PR #443 / #503 narrowing precedent for fn-return-shape sites (spec § Key decisions row 3). Multi-line attribute layout consistent with the existing `#[allow(clippy::struct_field_names, reason = "…")]` on `CountingRoot` at `wrapped_handler.rs:273-276`.

**Rejected alternatives.**

1. **Keep the workspace allow.** Rejected — the issue (#482) tracks the audit-narrowing follow-up of PR #443, and a 5-hit workspace allow loses signal: a future 6th site (e.g. a new fixture introduced during refactoring) would be silently absorbed instead of forcing the author to justify it per-site.
2. **Refactor the test fixtures to remove the multi-tuple returns** (e.g. replace `(Box<dyn Object>, EmissionLog)` with a `RecordingObjectHandle { obj, log }` builder struct). Rejected per spec § Out of scope — the fixtures are short-lived test helpers; the lint is the only concern.
3. **Per-item `#[allow]` on every runtime site instead of a type alias.** Rejected — three sites sharing one exact shape is the textbook case for a type alias (DRY + lint silenced for free). The alias also gives a name to the captured-emissions concept (`EmissionLog`) which is independently useful documentation.
4. **One shared alias spanning both files** (e.g. extract `EmissionLog` to a public test-utilities crate or a `quartzite-runtime/src/testing.rs` module reused from `quartzite-renderer/tests/`). Rejected per spec § Out of scope — different concrete `Mutex<…>` inner types (`Vec<(String, Vec<Value>)>` vs `Vec<Size>`/`u32`); sharing would force an over-generic alias.
5. **`#[expect]` instead of `#[allow]`.** Rejected by parity with PR #503 spec § Key decisions row 1 — `#[expect]` is reserved for cases where lint-firing is the verification signal; here lint-firing on a non-suppressed site would be a behavioural change we don't want.
6. **Per-crate `[lints.clippy]` table in `quartzite-renderer/Cargo.toml`.** Rejected — would still allow the lint across the entire `quartzite-renderer` crate (vs. the 2 named sites), losing the per-site documentation. Also blocked by spec § Technical constraints (per-crate `[lints.clippy]` tables are mutually exclusive with the workspace-inherited `[lints] workspace = true` block).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Introduce `type EmissionLog = Arc<Mutex<Vec<(String, Vec<Value>)>>>;` (using the already-imported `std::sync::Arc` / `parking_lot::Mutex` path forms that the existing test sites use — `std::sync::Arc<parking_lot::Mutex<Vec<(String, Vec<Value>)>>>` if the implementer keeps the existing fully-qualified style, or shorter `Arc<Mutex<…>>` if the alias is placed after the `use` block that already imports `Arc` at `object_tree.rs:451`). Replace the three shapes at `:769` (field), `:775` (`fn named` return — `(Box<dyn Object>, EmissionLog)`), and `:787` (`fn anonymous` return — `(Box<dyn Object>, EmissionLog)`). Verify with `grep -c 'Arc<Mutex<Vec<(String, Vec<Value>)>>>' quartzite-runtime/src/object_tree.rs` returning 0. | `quartzite-runtime/src/object_tree.rs` | — |
| 2 | Add item-level `#[allow(clippy::type_complexity, reason = "…")]` attributes on `CountingRoot::new` (`:286`) and `make_handler_with_root` (`:453`) in `quartzite-renderer/src/wrapped_handler.rs`. Reason strings drafted in the per-site table below; each must be site-specific and non-placeholder. Multi-line attribute layout matches the existing `CountingRoot` `struct_field_names` allow at `:273-276`. | `quartzite-renderer/src/wrapped_handler.rs` | — |
| 3 | Remove the line `type_complexity = "allow"` AND its `# 5 hits in runtime/object_tree.rs (3) + renderer/wrapped_handler.rs (2); …` justifying comment immediately above it from root `Cargo.toml`'s `[workspace.lints.clippy]` table (lines 47–48 in current tree). Run `cargo build` to refresh `Cargo.lock` per AGENTS.md § Workflow. | `Cargo.toml`, `Cargo.lock` | 1, 2 |
| 4 | Verification gates: `cargo clippy --workspace --all-targets -- -D warnings` (AC4), `cargo build` (AC5 + Cargo.lock refresh), `cargo test --workspace` (AC6), `cargo fmt -- --check` (AC7), `grep -n 'type_complexity' Cargo.toml` returns 0 hits (AC1), `grep -c 'Arc<Mutex<Vec<(String, Vec<Value>)>>>' quartzite-runtime/src/object_tree.rs` returns 0 (AC2), `grep -n 'clippy::type_complexity' quartzite-renderer/src/wrapped_handler.rs` returns exactly 2 hits with non-empty `reason = ` strings (AC3). | — (verification only) | 3 |

### Per-site reason-string drafts (design-phase detail; finalised at implementation)

| Site | `reason = "…"` draft |
|---|---|
| `wrapped_handler.rs:286` (`CountingRoot::new`) | `"test fixture: returns the constructed CountingRoot plus three Arc<Mutex<_>> read-back channels (resize log + press + release counters) so individual tests can assert per-method invocation counts without re-borrowing the fixture"` |
| `wrapped_handler.rs:453` (`make_handler_with_root`) | `"test fixture: returns the WrappedHandler under test plus its WinitWindowId and four Arc<Mutex<u32>> counter channels (press, release, key_press, key_release) shared with the registered CountingRoot for assertion read-back"` |

These are *drafts* — the implementer may tighten wording, but must preserve the per-site specificity (no copy-paste placeholders, no generic "test helper returns multiple Arcs" boilerplate). The two reason strings are intentionally distinct in wording because the two fixtures serve different roles (constructor of the fixture vs. composed-handler factory).

## Handoff plan

`M = 4`. Two groups, 3 + 1.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` enters Group A with fresh context.
- **Group A:** subtasks 1–3 — full implementation chunk (type alias for the runtime trio, per-item `#[allow]` attributes on the renderer pair, workspace-allow removal with `Cargo.lock` refresh). 3 subtasks; matches the non-terminal cap of exactly 3.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtask 4 — terminal group (1 subtask; within the 1..=3 range). Verification-only: runs the AC1–AC7 gate commands and reports.

## Risks

- **Type-alias still triggers `type_complexity` if rustfmt expands or threshold is hit elsewhere.** Mitigation: the `type` declaration itself is unlikely to cross the default `clippy.toml` complexity threshold (the score is computed per *use* site, and the alias counts as 1). Subtask 4 verifies via `cargo clippy --workspace --all-targets -- -D warnings`; if the alias *itself* trips the lint, escalate as a Spec Amendment per `.claude/skills/task/SKILL.md` Step 7 (would require a per-item `#[allow]` on the alias OR a fixture-level refactor — currently out of scope). Empirically the shape `Arc<Mutex<Vec<(String, Vec<Value>)>>>` should fit within the default score; the lint fires on use, not declaration.
- **Reason-string copy-paste drift (AC3 says non-empty + site-specific).** Mitigation: per-site draft table above gives the implementer 2 distinct strings. Self-review should check that the two reason strings are not byte-identical and each describes the *specific* fixture role. The `design-review` and `self-review` agents both treat copy-paste reason strings as a `major` finding under PR #443 precedent.
- **`reason =` MSRV.** `reason = "…"` requires Rust 1.81+; workspace `rust-version` is comfortably past that (spec § Technical constraints). No mitigation needed; named here for completeness.
- **Multi-line attribute formatting drift from rustfmt.** Long reason strings may force rustfmt to wrap. PR #443 set the precedent (rustfmt wraps to one-attribute-per-line naturally). Mitigation: `cargo fmt -- --check` is part of the AC7 gate; if rustfmt rewrites the attribute layout, accept its output rather than fighting it.
- **Alias name collision.** `EmissionLog` does not collide with any existing symbol in the `tests` module (`grep -n EmissionLog quartzite-runtime/src/object_tree.rs` returns 0 hits in the current tree). The implementer should re-grep at implementation time; if a collision exists, rename to `RecordingEmissionLog`.
- **A new `type_complexity` site is introduced concurrently** (e.g. by another PR merged between design and implementation). Mitigation: the `cargo clippy` gate would catch it; the implementer should re-grep at implementation time and, if a 6th site surfaces, surface as a Spec Amendment per `.claude/skills/task/SKILL.md` Step 7 rather than silently allowlisting the 6th.
- **`cargo clippy --workspace --all-targets` not exercising the lint due to feature-gating.** All 5 sites live inside `#[cfg(test)] mod tests` blocks; `--all-targets` compiles unit tests (lib + bin test harnesses) which is exactly where these modules live. Spec § Technical constraints already names `--all-targets` as load-bearing.
- **Workspace-comment regression.** Removing the `# 5 hits …` comment loses the rollup count. Acceptable: the per-site reason strings + type alias supersede the rollup; future readers `grep` the codebase if they need the count.

## Test Design

No new tests. This is a pure lint-suppression narrowing — the spec § Acceptance Criteria AC6 (`cargo test --workspace` exits 0) is the regression gate confirming no behavioural change to the `RecordingObject` / `CountingRoot` / `make_handler_with_root` fixtures or any test that consumes them.

- **Verification command set** (subtask 4):
  - `cargo clippy --workspace --all-targets -- -D warnings` — AC4
  - `cargo build` — AC5 + refreshes `Cargo.lock`
  - `cargo test --workspace` — AC6 (no regression)
  - `cargo fmt -- --check` — AC7
  - `grep -n 'type_complexity' Cargo.toml` — AC1 (expect 0 hits)
  - `grep -c 'Arc<Mutex<Vec<(String, Vec<Value>)>>>' quartzite-runtime/src/object_tree.rs` — AC2 (expect 0)
  - `grep -n 'clippy::type_complexity' quartzite-renderer/src/wrapped_handler.rs` — AC3 (expect exactly 2 hits, each with a non-empty `reason = ` string adjacent)

## Open questions

- None. The spec § Key decisions table resolved the narrowing-strategy split (alias for the runtime trio, per-item `#[allow]` for the renderer pair), the alias placement (inside `#[cfg(test)] mod tests`), the `#[allow]` placement (item-level on the `fn`), and the reason-string-specificity contract (design-phase per-site detail). The reason-string drafts in this design satisfy that last commitment, and the alias name (`EmissionLog`) is promoted from "working name" to final. The implementer may tighten reason-string wording or rename the alias if a collision surfaces, without re-entering design.
