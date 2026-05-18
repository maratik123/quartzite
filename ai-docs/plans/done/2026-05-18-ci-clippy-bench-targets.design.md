# Design: ci — extend clippy check to cover all targets

**Issue:** #465
**Spec:** [`ai-docs/plans/2026-05-18-ci-clippy-bench-targets.spec.md`](2026-05-18-ci-clippy-bench-targets.spec.md)
**Date:** 2026-05-18

## Approach

Two coordinated edits in a single PR. Both are required for the PR to merge green; reversing the order is not viable (widening the gate before fixing the violation breaks `clippy` immediately).

1. **Fix the pre-existing `clippy::let_underscore_lock` violation** in `quartzite-test-helpers/src/lib.rs:75` first, so the tree is already green under the widened invocation before the workflow change lands.
2. **Widen the `Clippy` step** in `.github/workflows/ci.yml` (job `clippy`, step `Clippy`, line 187) from `cargo clippy --workspace -- -D warnings` to `cargo clippy --workspace --all-targets -- -D warnings`.

### `let_underscore_lock` fix idiom — chosen: named binding `let _guard: MutexGuard<'static, ()> = test_lock();`

Lint message offers three remediations: named underscore-prefixed binding, explicit `drop(...)`, or removal. Re-reading the test:

```rust
// Compile-time return-type check — enforces the public signature
// stays stable across refactors.
let _: MutexGuard<'static, ()> = test_lock();
```

The line's **purpose** is a compile-time return-type assertion. The runtime acquire-then-immediately-drop side-effect is incidental, but it does fire: the previous two `drop(g1)` / `drop(g2)` already released the lock, so a third acquire-and-drop in the same single-threaded test is harmless and adds nothing.

Options considered:

| Option | Form | Verdict |
|---|---|---|
| **A. Named underscore binding** | `let _guard: MutexGuard<'static, ()> = test_lock();` | **CHOSEN.** Smallest diff, preserves both the runtime acquire (matches the surrounding test's intent — "acquire, drop, reacquire") and the compile-time type check verbatim, satisfies the lint by giving the guard a name so it lives to end-of-scope. Module-doc warning at line 36 ("Bind the guard to a named variable (`_lock` / `_guard`), **never** `let _ = test_lock();`") explicitly endorses this idiom and names `_guard` verbatim — A brings the test into exact alignment with the doc the lib itself prescribes. |
| **B. Explicit `drop(test_lock())`** | `drop::<MutexGuard<'static, ()>>(test_lock());` (or via a let-bound annotation) | Rejected. Requires a turbofish or a separate let-bound annotation to preserve the type-check; bulkier than A for no behavioural gain. The plain `drop(test_lock())` form would lose the explicit `MutexGuard<'static, ()>` annotation that is the entire point of the line. |
| **C. Move type check out of the runtime test** | E.g. a `const _: fn() -> MutexGuard<'static, ()> = test_lock;` at module scope, then delete line 75. | Rejected. Larger diff, splits the type-check away from the behavioural test that motivates it, and the lint can be satisfied trivially in-place. YAGNI. |

Chosen: **A**. New line:

```rust
let _guard: MutexGuard<'static, ()> = test_lock();
```

The comment above the line stays unchanged — it still describes "compile-time return-type check". No `#[allow]` is introduced.

### Workflow widening

One-line edit on `.github/workflows/ci.yml:187`:

```diff
-        run: cargo clippy --workspace -- -D warnings
+        run: cargo clippy --workspace --all-targets -- -D warnings
```

All three matrix legs (`ubuntu-latest`, `macos-latest`, `windows-latest`) inherit the change automatically — the `run:` line is matrix-independent. The `clippy` cache-shared-key (`${{ runner.os }}-stable-v2-${{ env.ImageVersion }}`) is shared with `build`/`test`/`docs` jobs, so the widened compile graph (benches + tests + examples) reuses cached compilation artefacts when those jobs ran first — cold-cache cost is bounded; warm-cache cost is near-zero. This is already covered by the spec's acceptance of the cost in Q1.

### Why these two edits constitute one logical change

The widened invocation surfaces the `let_underscore_lock` violation immediately — splitting the PR would either (a) leave master red between merges, or (b) require landing the fix as a no-op PR first with no CI signal that it was necessary. Per the spec's *Out of scope* note ("Other clippy-failing patterns surfaced after the `let_underscore_lock` fix … are pulled into scope as a single batch"), this is the established pattern.

### Pre-verification of additional violations

The spec calls for the design agent to re-verify that no *other* `--all-targets` violations exist beyond the one named. The design phase ran `cargo clippy --workspace --all-targets -- -D warnings` locally; the only error reported was the `let_underscore_lock` at `quartzite-test-helpers/src/lib.rs:75`. No additional violations to pull into scope.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Fix `let_underscore_lock` in test by giving the guard a named binding `_guard`. | `quartzite-test-helpers/src/lib.rs` | — |
| 2 | Widen the `Clippy` step invocation to `cargo clippy --workspace --all-targets -- -D warnings`; run `actionlint .github/workflows/ci.yml` before commit (AGENTS.md AXIOM); locally rerun `cargo clippy --workspace --all-targets -- -D warnings` to confirm exit 0. | `.github/workflows/ci.yml` | 1 |

Task count: 2. Within `/task` budget; no split-into-multiple-issues escalation.

## Handoff plan

Total subtasks: M = 2. Group cap is 3 consecutive subtasks; non-terminal groups MUST be exactly 3; terminal groups MUST be in `1..=3`.

- **Group A:** subtasks 1–2 — terminal group (2 subtasks; within the `1..=3` range). No handoff between groups; the single group completes Step 8 in its own `/context-reset` subagent.

Per `.claude/skills/task/SKILL.md` Step 8 + `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry), the entry into Group A is bound to a `/context-reset` handoff at the start of the group — the every-group handoff contract applies to single-group designs as well.

## Risks

- **R1 — Cold-cache compile time on the `clippy` job grows.** Widening to `--all-targets` adds bench + integration-test + example compilation to clippy's analysis pass. **Mitigation:** sccache + the shared `${{ runner.os }}-stable-v2-${{ env.ImageVersion }}` cache key reuse build artefacts produced by the `build` and `test` jobs; warm-cache cost is near-zero. Spec Q1 accepted this trade-off explicitly.
- **R2 — Platform-specific clippy lint divergence surfaces on macOS / Windows but not Linux.** Some workspace-lint hits depend on `#[cfg(target_os = …)]` paths that the local pre-verification did not exercise. **Mitigation:** matrix preserved (3 OS legs); PR CI will surface any such divergence before merge. If hit, treat as the spec's "single batch" carve-out — fix in this PR.
- **R3 — Behavioural regression in `test_lock_acquires_and_releases` from binding the guard.** The bound `_g` now lives to end-of-scope, holding the lock past the third reacquire (instead of dropping immediately). **Mitigation:** end-of-scope is end-of-function in this single-threaded test body; `_g` is dropped on function return; no other thread is waiting on the lock; no other test in the same `mod tests` block acquires it. Behaviour-preserving.
- **R4 — Doc-line at `quartzite-test-helpers/src/lib.rs:36` ("Bind the guard to a named variable … never `let _ = test_lock();`") drifts further from the test.** **Mitigation:** the chosen fix actively brings the test *into compliance* with that doc — no drift introduced; arguably the violation existed precisely because the doc rule was not mechanically enforced. Post-fix, the workspace lint enforces it on every consumer.
- **R5 — `actionlint` regression on the modified workflow file.** **Mitigation:** AGENTS.md AXIOM is enforced as Step 2's pre-commit check; the diff is a single-line `run:` value change which is unlikely to trip any action-version / expression-syntax / shell-quoting rule, but verification is mandatory regardless.

## Test Design

No new tests required. The existing test `test_lock_acquires_and_releases` continues to exercise both the compile-time return-type assertion (via the type-annotated binding) and the runtime acquire/drop sequence. The widened CI invocation *is* the gate's own integration test — AC4 verifies the gate exercises bench / integration-test / example targets by reading the green job log.

**Self-verification commands** (run before commit, not as new test code):

1. `cargo clippy --workspace --all-targets -- -D warnings` — must exit 0 (AC3).
2. `cargo test -p quartzite-test-helpers --lib` — must exit 0 (the modified test still passes).
3. `actionlint .github/workflows/ci.yml` — must exit 0 (AC2, AGENTS.md AXIOM).
4. `cargo fmt -- --check` — must exit 0 (no formatting drift introduced by the one-char edit).

Post-push verification (AC4) reads the green PR `Clippy` job log to confirm compilation of `benches/macro_object.rs`, `quartzite-core/benches/signal_property.rs`, `quartzite-runtime/benches/object_tree.rs`, at least one `*/tests/*.rs` integration test, and at least one file under `examples/`.

## Open questions

(none — spec answered all Q1-round items; design-phase choice on the fix idiom is resolved in § Approach above.)
