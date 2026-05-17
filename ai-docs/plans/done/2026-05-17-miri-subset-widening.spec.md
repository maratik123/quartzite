# Miri subset widening v2 (`--lib` + `--test` filtering)

**Source:** issue #431
**Date:** 2026-05-17
**Tracked in:** #431

## Scope

1. Extend the existing `.github/workflows/miri.yml` job (the v1 9-crate FFI-free subset) with **additional `cargo miri test` invocations** that exercise three workspace crates the v1 design excluded on dep-graph membership but whose `src/` and a subset of `tests/*.rs` are renderer-free when the right `--lib` / `--test <name>` target selector is used.
2. The v2 invocation set is **six cargo invocations** (three from the issue body + three additional FFI-free integration tests confirmed widenable in Round 1). MAY be combined into fewer cargo invocations if the design agent prefers (e.g., the macros `--test` filtering already demonstrates multi-test syntax). Final shape is a design-phase call (see Open questions); either form satisfies the ACs.
   1. `cargo miri test --lib -p quartzite-widgets` (lib tests only — `tests/snapshots.rs` and `tests/support/mod.rs` import the renderer and are side-stepped by `--lib`).
   2. `cargo miri test --lib -p quartzite-style` (same shape).
   3. `cargo miri test -p quartzite-macros --test extend --test meta_enum --test object --test object_impl` (four FFI-free integration tests; `tests/via_facade.rs` uses the facade and stays excluded by allow-list rather than `--exclude`).
   4. `cargo miri test -p quartzite-widgets --test no_style_dep` (subject to AC-bound design-time re-audit, see AC6 / Scope §3).
   5. `cargo miri test -p quartzite-widgets --test re_exports` (same re-audit gate).
   6. `cargo miri test -p quartzite-style --test third_party_paint` (same re-audit gate).
3. **Design-time re-audit gate (binding).** Before the design subtask that adds invocations §2.4 / §2.5 / §2.6 to `miri.yml`, the design agent MUST re-verify each named test file is `--test`-FFI-free by **both** of the following recipes returning zero hits:
   - `grep -nE 'use quartzite_renderer|extern crate quartzite_renderer|mod support' quartzite-widgets/tests/no_style_dep.rs quartzite-widgets/tests/re_exports.rs quartzite-style/tests/third_party_paint.rs` → zero matches. If `mod support;` IS declared by any file, the support module itself must also be renderer-free under the same grep.
   - The test file's `use` declarations reference only renderer-free crates / std (a `cargo tree`-equivalent check on the dev-deps actually pulled by the chosen `--test` target binary).
   
   Any file failing the re-audit is **demoted** from the v2 invocation list and the demotion is recorded in the design's `## Rejected alternatives` section with the specific failing reason. The widening of invocations §2.4–§2.6 is therefore conditional; the final v2 set has between 3 and 6 invocations.
4. The v1 invocation (9 crates, default target selector) stays exactly as it is today. The v2 widening is purely **additive**.
5. The cumulative `MIRIFLAGS` set (`-Zmiri-tree-borrows -Zmiri-ignore-leaks -Zmiri-disable-isolation`) stays unchanged. The v2 invocations run under the same flag set.
6. `continue-on-error: true` v1 policy is preserved — the v2 invocations also start permissive and are promoted to hard-failure only via the separately-tracked deferred follow-up to #422.
7. `actionlint .github/workflows/miri.yml` must pass after the workflow edit.
8. `ai-docs/unsafe-index.md § Notes` gains a one-line update to the existing Miri cross-reference bullet acknowledging the widened subset (≤ 110-char budget, mirroring the v1 design's AC6 envelope). Design picks: edit the existing bullet in place, OR add a new sibling bullet — either form satisfies AC5 as long as the chosen bullet stays within budget.
9. The first post-merge master run produces a runnable Miri log that captures what was tested across **all final v2 invocations** (v1 9-crate + 3–6 widenings, depending on re-audit outcome) — either green across the board OR per-finding disposed per the AC8 envelope (file follow-up issue OR `cfg_attr(miri, ignore)` OR file-level `#![cfg(not(miri))]` OR `MIRIFLAGS` adjustment). The log is grep-able by a future `/ai-audit` run.

## Out of scope

- **`quartzite-renderer`** — permanently excluded. Its library uses `winit`/`wgpu`/`vello` directly; no `--lib`/`--test` filter can side-step that.
- **`quartzite-widgets/tests/snapshots.rs`** and **`quartzite-style/tests/snapshots.rs`** — GPU snapshot harness; renderer-dependent by design.
- **`quartzite-macros/tests/via_facade.rs`** — uses the facade crate which depends on renderer; stays excluded via `--test` allow-list (`extend`, `meta_enum`, `object`, `object_impl` only).
- **New `MIRIFLAGS`.** The v1 flag set already accumulated three flags across PRs #425–#430; v2 should not add a new flag unless a new failure class surfaces in the widened subset (in which case it follows the AC8 envelope, not this spec).
- **Flipping `continue-on-error` to `false`.** Already a separately-tracked deferred follow-up to #422; this spec does not advance that flip.
- **Cross-target Miri** (Windows / macOS / big-endian / 32-bit). Already deferred per #424.
- **`quartzite-macros` `--test via_facade` under `--no-default-features`.** Aggressive v3 variant — out of scope.
- **Cumulative-runtime ceiling.** The job runtime estimate (~5 min → ~15 min per the issue body) is informational, not a binding budget. No runtime cap is added by this spec; v1's tolerated master-job cadence applies.
- **AGENTS.md edit** — none required by this spec (file currently below the 35k early-warning band).
- **No new spec / design / progress document beyond this task's own.** No `INDEX.md` row movement except the standard `/task` Step 4 add and Step 12 done-move.

## Deferred

- v3 variant: `quartzite-macros --test via_facade --no-default-features` if the facade's renderer dep is feature-gated on a path that `--no-default-features` excludes | requires a separate feature-flag audit | **yes — separate follow-up issue** if `/next` ever prioritises it.
- A v2-aware `[dependencies]`-only audit recipe (the v1 recipe `grep -nE "winit|wgpu|vello" <crate>/Cargo.toml` does not distinguish prod-deps from dev-deps; v2 makes the distinction load-bearing) folded into the AC7-equivalent verification recipe of the v1 design | nice-to-have, not blocking | **no — `ai-docs/deferred/*.md` row** suffices.
- Promotion of the `cargo miri test` invocations to a single combined form vs separate steps (the v2 widening preserves either structural option; design agent picks) | pure presentation choice | **no — design decision**.

## Key decisions

| Question | Decision |
|---|---|
| Where does v2 live | Edits the **existing** `.github/workflows/miri.yml`, not a new workflow file. The v1 single-job, `continue-on-error: true`, distinct `cache-shared-key` shape is preserved. |
| Target-selector strategy | `--lib` for `quartzite-widgets` and `quartzite-style` (sidesteps dev-dep `quartzite-renderer` because Cargo only compiles dev-deps when the build graph for the chosen target requires them — `--lib` does not). Explicit `--test <name>` allow-list for `quartzite-macros` (forward-safety: a new integration test file lands excluded by default until added to the list). |
| Macros allow-list contents | `extend`, `meta_enum`, `object`, `object_impl` — the four `tests/*.rs` files verified renderer-free by the orchestrator's pre-flight grep. `via_facade.rs` excluded. |
| Widget / style integration-test widening (Round-1 answer) | **WIDEN** with design-time re-audit gate. The three additional invocations `quartzite-widgets --test no_style_dep`, `quartzite-widgets --test re_exports`, and `quartzite-style --test third_party_paint` are bound into the v2 AC list NOW. Any failing the design-time `--test`-FFI-free re-audit (Scope §3) is demoted and the demotion recorded in the design's `## Rejected alternatives`. Final v2 invocation count: 3–6. |
| `MIRIFLAGS` posture | Unchanged from v1. The v2 invocations inherit the job-level `MIRIFLAGS` env. No new flag added speculatively. |
| `continue-on-error` posture | Stays `true` for the whole job (v1 + v2 invocations). Promotion to `false` is the separately-tracked deferred follow-up to #422. |
| Failure disposition envelope (AC8 from #424) | Identical to the #424 / #425–#430 stabilisation arc: (a) `MIRIFLAGS` adjustment for whole-class noise; (b) `cfg_attr(miri, ignore)` for a single false-positive test; (c) file-level `#![cfg(not(miri))]` for a whole integration test file; (d) follow-up issue per real-UB finding. The "v2 widening produces 0–N new findings" expectation is explicit; first-master-push log determines per-finding disposition. |
| `actionlint` gate | Enforced per AGENTS.md `## Build & Test` AXIOM. Same gate as v1; binding before `git add` on the modified workflow file. |
| Master-only-comment AXIOM | Already satisfied by the existing v1 workflow's `# Why master-only:` header block — no change required by v2. |
| `ai-docs/unsafe-index.md § Notes` bullet | Edit the existing one-line cross-reference in place OR add a sibling bullet — design picks. Either form satisfies AC5 as long as ≤ 110-char budget is preserved (mirrors the v1 design's AC6 envelope). |
| `INDEX.md` | New active-plan row for this task, added by `/task` Step 4 per the existing convention. No other INDEX movement beyond the standard Step 12 done-move at task close. |
| `learnings.md` posture | Per existing CI-skill convention (`/pr-ci-failed`, `/master-ci-failed` "never edit `learnings.md`" rule), CI fixes flowing from any new Miri findings surfaced by the v2 invocations do NOT append learnings unless the fix is an in-`/task` flow per the Boundary rule 2 in-flow exception. No special-case rule needed in this spec. |
| Cumulative runtime | Informational only (~5 min → ~15 min estimate from the issue body). No runtime cap is added; CI cost is acceptable to the project at v1's tolerated cadence. |
| Combined-step vs separate-steps shape | Design-detail. Either form satisfies the ACs; the design agent picks based on readability + `actionlint` output. Default: keep `--lib` and `--test` invocations as separate cargo commands so failures don't mask each other in the CI summary (separate exit codes per invocation); the design agent may collapse if collapsing is cleaner. |
| Workflow-step placement | Design picks. Default: place the new step(s) immediately after the existing `cargo miri test (FFI-free crate subset)` step, sequentially dependent on toolchain setup; no parallel jobs needed. |

## Technical constraints

- **AGENTS.md AXIOM — `actionlint` MUST pass before `git add`** on the modified `.github/workflows/miri.yml`. (Build & Test § AXIOM.)
- **AGENTS.md AXIOM — master-only workflow guard.** The pre-existing `# Why master-only: <reason>` comment in `miri.yml` is preserved verbatim; v2 does not weaken or extend its scope claim (the rationale — "no PR-side workflow exercises an equivalent Miri path" — is unchanged because v2 still runs master-only).
- **AGENTS.md § Dependency Versions** — no new action-version pins introduced by v2 (the workflow's `actions/checkout@v6` and `actions-rust-lang/setup-rust-toolchain@v1` are unchanged). If the design agent decides to bump the nightly pin for any reason, the live-registry-lookup rule applies per the AXIOM.
- **Audit recipe (v2-aware).** A crate is `--lib`-FFI-free **iff** neither its prod-dep transitive closure nor its `src/` source code imports `winit`/`wgpu`/`vello`. The orchestrator's pre-flight already confirmed: `grep -rEn 'use quartzite_renderer|extern crate quartzite_renderer' quartzite-widgets/src/ quartzite-style/src/` returns zero matches. A crate is `--test <name>`-FFI-free **iff** the named test file's source does not import the renderer AND any `mod support;` it pulls is also renderer-free. The design doc records the per-file recipe verbatim (`grep -nE 'use quartzite_renderer|extern crate quartzite_renderer|mod support' <file>`) so future maintainers know the v2-aware recipe differs from the v1 `Cargo.toml`-only `grep -nE 'winit|wgpu|vello'`.
- **`quartzite-widgets/Cargo.toml`** and **`quartzite-style/Cargo.toml`** both have `[dependencies]` empty of renderer and `[dev-dependencies]: quartzite-renderer = { path = ... }` — the renderer dep is dev-only, which is exactly what makes `--lib` legal here. The widened integration-test invocations (Scope §2.4–§2.6) are legal only because the chosen `--test <name>` target binaries do not transitively pull the renderer dev-dep (this is what the design-time re-audit gate verifies).
- **`quartzite-macros`** is a `[lib] proc-macro = true` crate with `[dev-dependencies]` empty (no dev-dep on the facade or the renderer). Each `tests/*.rs` file is its own integration test binary; Miri interprets the compiled test binary, not the proc-macro itself (which runs host-native at expansion time). `via_facade.rs` is excluded because it `use`s the facade crate (`quartzite = { path = ".." }`) which pulls the renderer; `extend`/`meta_enum`/`object`/`object_impl` use only the proc-macro's own re-exports.
- **`ai-docs/unsafe-index.md`** size is well below any extraction threshold; either an in-place edit to the existing bullet or a sibling bullet does not change the file's size class.
- **`continue-on-error: true` is a job-level setting** in v1's `miri.yml`. Whether the v2 invocations live as additional `steps:` under the same job or as new jobs is a design-detail; spec only requires the cumulative permissive posture be preserved.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.github/workflows/miri.yml` is extended (not replaced) with the v2 invocation set. The set comprises **between 3 and 6** `cargo miri test` invocations (or equivalent combined-step shapes exercising the same target selectors). Concretely: (a) `cargo miri test --lib -p quartzite-widgets`; (b) `cargo miri test --lib -p quartzite-style`; (c) `cargo miri test -p quartzite-macros --test extend --test meta_enum --test object --test object_impl`; AND each of (d) `cargo miri test -p quartzite-widgets --test no_style_dep`, (e) `cargo miri test -p quartzite-widgets --test re_exports`, (f) `cargo miri test -p quartzite-style --test third_party_paint` that **passes** the design-time `--test`-FFI-free re-audit (Scope §3). The v1 9-crate invocation is preserved verbatim. |
| AC2 | The design-time `--test`-FFI-free re-audit is executed against each of invocations (d), (e), (f). For each: the per-file grep `grep -nE 'use quartzite_renderer\|extern crate quartzite_renderer\|mod support' <test-file>` returns zero hits (and if any `mod support;` is present, the support module is also renderer-free under the same grep). Any failure → the invocation is **demoted** from the workflow and the demotion is recorded in the design's `## Rejected alternatives` section with the specific failing pattern. Self-review walks the audit log before APPROVE. |
| AC3 | The cumulative `MIRIFLAGS` job env (`-Zmiri-tree-borrows -Zmiri-ignore-leaks -Zmiri-disable-isolation`) is preserved verbatim and applies to all v2 invocations. No new flag is added by this spec. |
| AC4 | The job-level `continue-on-error: true` setting is preserved verbatim. Any new `steps:` introduced by v2 do not override it. |
| AC5 | `actionlint .github/workflows/miri.yml` exit 0 locally (recorded in the PR body or commit message) and in the post-push CI gate. |
| AC6 | `ai-docs/unsafe-index.md § Notes` Miri cross-reference is updated to acknowledge the widened subset — either the existing bullet edited in place, or a sibling bullet added. The chosen bullet is ≤ 110 chars (mirrors the v1 design's AC6 budget). |
| AC7 | The design doc records the **v2-aware FFI-free audit recipe** (distinguishes prod-deps from dev-deps for the `--lib` invocations; cites the per-file `grep -nE 'use quartzite_renderer\|extern crate quartzite_renderer\|mod support' <file>` recipe for the `--test` allow-list AND the demotion path for any file failing the re-audit). |
| AC8 | First post-merge master run of `miri.yml` produces a Miri log that names which crates and target-selectors were exercised across all final invocations (v1 9-crate + 3–6 v2 widenings). Either: (a) all invocations green, OR (b) each new finding has a follow-up issue OR a recorded `cfg_attr(miri, ignore)` / file-level `#![cfg(not(miri))]` / `MIRIFLAGS` adjustment in a follow-up PR, with that disposition recorded in the PR body. The artifact is grep-able by a future `/ai-audit` run (the invocation step names appear verbatim in the CI log). |
| AC9 | `ai-docs/plans/INDEX.md` gains an active-plan row for this task per `/task` Step 4 convention. |
| AC10 | No edit to `AGENTS.md` is required by this spec. If the design agent finds an unavoidable need to edit it, the 35,000-char early-warning band remains the cap to watch. |

## Open questions

- **Combined-step vs separate-steps shape** for the v2 invocations in `miri.yml`. Both forms satisfy the ACs. Default: keep `--lib` and `--test` invocations as separate cargo commands so failures don't mask each other in the CI summary (separate exit codes per invocation). Design agent may collapse if collapsing is cleaner under `actionlint`. The 3 new `--test` invocations MAY be combined per crate (e.g., `cargo miri test -p quartzite-widgets --test no_style_dep --test re_exports`) — design's call.
- **Should the widget integration-test invocations be combined with the `--lib` invocation into a single `cargo miri test --lib --test no_style_dep --test re_exports -p quartzite-widgets` command?** Default: **NO**. Keep them separate so `--lib` failures don't mask `--test <name>` failures (separate cargo invocations means separate exit codes for the CI summary).
- **Workflow-step placement** relative to the existing `cargo miri test (FFI-free crate subset)` step. Default: design picks — likely immediately after, with sequential dependency on toolchain setup (no parallel jobs needed).
- **`quartzite-macros --test via_facade` under `--no-default-features`** — aggressive v3 candidate per the issue body's *Notes* section. Out of scope for v2; recorded here so it does not need re-discovery.
- **Stabilisation-window threshold for the `continue-on-error: false` flip** — unchanged from the original v1 deferred follow-up to #422. Out of scope; recorded here to make it explicit that v2 does not advance that flip.
