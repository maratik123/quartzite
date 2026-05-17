# Design: Miri deny-list inversion (workspace-wide + per-test opt-out)

**Issue:** #436
**Date:** 2026-05-17

## Approach

Adopt **Shape B** of the spec's Open question §1 — keep `quartzite-macros` inside
`--workspace` and rely on Miri's native graceful skip for the proc-macro lib's
unittests; rely on a per-file `#![cfg(not(miri))]` on the renderer-touching
integration files for opt-out. The workflow collapses to a **single primary**
`cargo miri test --workspace --exclude quartzite-renderer` invocation.

### Why Shape B (chosen)

Empirically settled by local dry-run on `nightly-2026-05-01` (commands in
*Audit recipe execution* below):

1. **Miri skips proc-macro lib unittests with a clean exit.** Running
   `cargo +nightly-2026-05-01 miri test --lib -p quartzite-macros` (under the
   workflow's full `MIRIFLAGS`) prints exactly:
   > `Running unit tests of 'proc-macro' crates is not currently supported by Miri.`
   …and the test-runner *exits 0*. No `error:` / non-zero exit. Under
   `--workspace`, the proc-macro lib's unittest target binary is built (Cargo's
   normal behaviour — proc-macros build for the host target) and then quietly
   skipped at runtime. The runtime skip is built into `cargo-miri`'s runner
   logic, not a Miri error path.
2. **`quartzite-macros/tests/via_facade.rs` passes Miri unchanged.** It uses
   `quartzite::core::*` + `quartzite::macros::*` (top-level facade), whose
   default features are `["std", "derive"]` — neither `widgets` nor `style`
   nor anything that pulls `quartzite-renderer`. The spec's hypothesis that
   `via_facade.rs` needs `#![cfg(not(miri))]` is **falsified** by this dry-run.
   Shape B therefore needs **zero** per-file or per-test tags inside
   `quartzite-macros/tests/` — its four currently-allow-listed targets
   (`extend`, `meta_enum`, `object`, `object_impl`) plus `via_facade` all
   pass Miri under `--workspace --exclude quartzite-renderer`.
3. **No second cargo invocation required.** Shape A's sibling step
   (`cargo miri test -p quartzite-macros --test {extend,meta_enum,object,object_impl}`)
   would be the *only* surviving `-p`/`--test` allow-list in the workflow under
   the deny-list contract — purely to re-include what Shape B picks up for
   free. Shape A is strictly more workflow surface for the same coverage.
4. **AC1 alignment.** AC1 calls for a *single primary* `cargo miri test
   --workspace [--exclude …]`. Shape B is the literal minimum.

### Why not Shape A (rejected)

Shape A — `--exclude quartzite-macros` + a sibling `cargo miri test -p
quartzite-macros --test <list>` step — was the spec's first-named alternative.
Rejected on three grounds:

- Adds a second workflow step purely to re-include four integration tests that
  Shape B picks up automatically.
- The sibling step would be the **only** surviving `-p`/`--test` allow-list
  in the workflow. The whole point of the inversion is to retire that
  maintenance contract.
- The `# Why excluded:` comment would have to apologise: "macros lib excluded
  because Miri can't run proc-macros, but its integration tests are
  re-included below". Shape B replaces this with a single `# Why excluded:
  prod-dep FFI: winit/wgpu/vello` comment over the renderer-only exclusion.

### What changes

- **Workflow:** `.github/workflows/miri.yml` — delete 7 of the existing
  per-crate / per-test step blocks (lines 85–113 inclusive) PLUS the
  3-line dangling comment block immediately above them (lines 82–84,
  which references the now-retired FFI-free `-p` audit). Replace the
  combined range (lines 82–113) with one `cargo miri test --workspace
  --exclude quartzite-renderer` step preceded by the new `# Why
  excluded:` block. The job-level `env: MIRIFLAGS: …` block (lines
  25–44), `continue-on-error: true` (line 24), nightly toolchain pin
  (line 55), `cargo miri setup` step (lines 72–73), Linux deps step
  (lines 74–81), and the master-only comment block (lines 1–4) are
  preserved verbatim.
- **Per-file `#![cfg(not(miri))]` additions** (3 new files): widgets
  `tests/snapshots.rs`, widgets `tests/support_internals.rs`, style
  `tests/snapshots.rs`. Each receives a ≥ 3-line top-of-file `//` comment
  block per the spec's fallback recipe.
- **Per-test `#[cfg_attr(miri, ignore = "<reason>")]`:** zero new occurrences
  on the empirical evidence. Every test the workspace-wide invocation
  surfaces is either Miri-clean (no tag) OR in a whole-file-FFI/GPU file
  that takes the per-file form. The `cfg_attr` machinery stays documented
  in `ai-docs/miri-policy.md` as the *default* shape; the v3 sweep happens
  to find no per-test cases. This is a finding, not a contract violation
  — the maintenance contract still says "future Miri-hostile tests get
  `cfg_attr` first, fallback to `cfg(not(miri))` only when per-test won't
  work".
- **Two existing `#![cfg(not(miri))]` files** (`quartzite-widgets/tests/no_style_dep.rs`,
  `quartzite-runtime/tests/timer.rs`) stay verbatim per AC11. Their
  top-of-file comment blocks already match the v3 recipe.
- **New file `ai-docs/miri-policy.md`** — full policy body (per-test default,
  per-file fallback, exclusion list with reason format, six reason-prefix
  list with future-extension rule, audit recipe, maintenance contract,
  two-exemplar references).
- **`AGENTS.md § Build & Test`** — single-line cross-reference row added
  inside the existing fenced block's trailing prose section (not inside the
  fenced bash block — see *Section placement* below). Mirrors how
  `ai-docs/dependency-versions.md` is referenced from `AGENTS.md §
  Dependency Versions`.
- **`ai-docs/plans/INDEX.md`** — active-plan row added per `/task` Step 4
  convention; done-move per Step 12 on completion.

### Audit recipe execution (dry-run command + file-touch count)

- **Static enumeration command** (recorded verbatim; reproducible by future
  contributors):
  ```bash
  rg -n '^\s*(#\[(?:tokio::)?test(?:\(.*\))?\]|#\[rstest(\(.*\))?\])' --type rust \
    quartzite-core/src/ quartzite-core/tests/ \
    quartzite-geometry/src/ \
    quartzite-paint-api/src/ \
    quartzite-events/src/ \
    quartzite-event-types/src/ \
    quartzite-paint/src/ \
    quartzite-style-types/src/ \
    quartzite-style-dispatch/src/ \
    quartzite-runtime/src/ quartzite-runtime/tests/ \
    quartzite-style/src/ quartzite-style/tests/ \
    quartzite-widgets/src/ quartzite-widgets/tests/ \
    quartzite-macros/src/ quartzite-macros/tests/ \
    src/ tests/
  ```
  Output is grep-friendly — one match per line, format
  `<file>:<lineno>:<matched-attribute>`. Total matches across all non-excluded
  crates: **~660 attribute hits** (lib unittests + integration tests
  combined). Note: this is attribute-occurrence count, not unique-test count
  (`#[rstest]` cases multiply at the parametric level).
- **Local dry-run command** (recorded for the PR body per AC8):
  ```bash
  MIRIFLAGS='-Zmiri-tree-borrows -Zmiri-ignore-leaks -Zmiri-disable-isolation' \
    cargo +nightly-2026-05-01 miri test --workspace --exclude quartzite-renderer
  ```
  After Subtask 2's per-file tags land, this command MUST exit 0 before
  the workflow edit (Subtask 3) commits. The implementation step records
  the elapsed wall time + final test-summary line ("X passed; 0 failed;
  Y ignored") in the PR body.
- **Expected file-touch count.** Production / test source: **3 new
  per-file `#![cfg(not(miri))]` tags** (widgets `snapshots.rs`, widgets
  `support_internals.rs`, style `snapshots.rs`). Workflow: 1 file
  (`.github/workflows/miri.yml`). New doc: 1 (`ai-docs/miri-policy.md`).
  AGENTS.md: 1 single-line addition. INDEX.md: 1 row add (active) + 1
  row move (done) across the PR's two `/task` lifecycle phases. **Total
  ≈ 7 files touched** (3 test files, 1 workflow, 1 new doc, 1 AGENTS.md
  edit, 1 INDEX.md row).

### Workflow diff shape

Before (lines 82–113 — the 3-line dangling comment block at 82–84 +
the 7 step blocks at 85–113; reproduced from `miri.yml`):

```yaml
      # The explicit -p list mirrors the FFI-free audit recorded in
      # ai-docs/plans/done/2026-05-17-miri-master-push-job.design.md and must be
      # re-audited whenever a workspace member is added or removed.
      - name: cargo miri test (FFI-free crate subset)
        run: |
          cargo miri test \
            -p quartzite-core \
            -p quartzite-geometry \
            -p quartzite-paint-api \
            -p quartzite-events \
            -p quartzite-event-types \
            -p quartzite-paint \
            -p quartzite-style-types \
            -p quartzite-style-dispatch \
            -p quartzite-runtime
      - name: cargo miri test (widgets --lib)
        run: cargo miri test --lib -p quartzite-widgets
      - name: cargo miri test (style --lib)
        run: cargo miri test --lib -p quartzite-style
      - name: cargo miri test (macros --test allow-list)
        run: |
          cargo miri test -p quartzite-macros \
            --test extend \
            --test meta_enum \
            --test object \
            --test object_impl
      - name: cargo miri test (widgets --test no_style_dep)
        run: cargo miri test -p quartzite-widgets --test no_style_dep
      - name: cargo miri test (widgets --test re_exports)
        run: cargo miri test -p quartzite-widgets --test re_exports
      - name: cargo miri test (style --test third_party_paint)
        run: cargo miri test -p quartzite-style --test third_party_paint
```

After (single step block, replaces all seven above):

```yaml
      # Why excluded: prod-dep FFI: winit/wgpu/vello reach quartzite-renderer's
      # lib by construction; no --lib/--test filter side-steps the dep graph.
      # Every other workspace crate is in scope; per-test or per-file opt-outs
      # (`#[cfg_attr(miri, ignore = "<reason>")]` / `#![cfg(not(miri))]`)
      # live at the source, NOT in this file. See ai-docs/miri-policy.md.
      - name: cargo miri test (workspace, deny-list)
        run: cargo miri test --workspace --exclude quartzite-renderer
```

Step count: 7 → 1 (net -6 step blocks). Additionally the 3-line
dangling comment block at lines 82–84 (which cites the v2 FFI-free
audit and codifies the "re-audit whenever a workspace member is added
or removed" maintenance contract — the exact contract v3 retires) is
deleted, NOT preserved. Replacing it in-place with the new `# Why
excluded:` block leaves zero stale prose between the Linux deps step
and the workspace `cargo miri test` step. The `# Why excluded:` block
is multi-line by design — AC2 calls for an immediately-preceding
comment naming the irreducible obstacle, and the longer body documents
the AGENTS.md "new crate-level exclusion requires design-phase
justification" contract without forcing the reader to chase the spec.

Mental-diff sanity check (post-edit shape): the only comments
surviving in `miri.yml` above the new workspace step are (i) the
master-only header at lines 1–4, (ii) the `continue-on-error: true`
rationale inside the job block, (iii) the per-flag `MIRIFLAGS`
justification blocks, (iv) the `+nightly` syntax warning above
`cargo miri setup`, (v) the Linux-deps inline comment, and (vi) the
new `# Why excluded:` block. No prose between (v) and (vi) — the old
lines 82–84 are gone.

### Per-crate / per-test tagging matrix

(a) **Workflow-excluded crates** (closed set under v3):

| Crate | Comment marker in `miri.yml` | Reason |
|---|---|---|
| `quartzite-renderer` | `# Why excluded: prod-dep FFI: winit/wgpu/vello …` | `winit` / `wgpu` / `vello` are prod-deps reaching the lib — `--lib` / `--test` filters cannot side-step the dep graph. |

The closed set is **one crate**. `quartzite-macros` is NOT excluded; its
proc-macro lib unittests skip themselves at runtime (Miri's native
"unsupported" path with exit 0), its integration tests run.

(b) **Files taking new `#![cfg(not(miri))]`** (whole-file fallback):

| File | Reason category | Why per-test wouldn't suffice |
|---|---|---|
| `quartzite-widgets/tests/snapshots.rs` | `GPU init` | Every test calls `harness_or_skip(…)` which calls `RenderHarnessBuilder::build()` → `wgpu::Instance::default()`. Per-test `cfg_attr` would leave the `use quartzite_renderer::RenderHarnessBuilder;` import — Miri compiles the module, the dev-dep dep graph still pulls the renderer's FFI into the binary, and `wgpu` initialization at test setup still fires before harness selection. Empirically confirmed by local probe: Miri aborts at `<wgpu::api::instance::Instance as Default>::default` (render_harness.rs:148). |
| `quartzite-widgets/tests/support_internals.rs` | `FFI` | Tests call `support::pixel_diff` which transitively calls `nv_flip_sys::flip_image_color3_new` (a C FFI). Empirically confirmed: Miri aborts with `unsupported operation: can't call foreign function 'flip_image_color3_new' on OS 'linux'` at `nv-flip-0.1.2/src/lib.rs:137`. Per-test would still leave the FFI-bound `support::*` import in the module. |
| `quartzite-style/tests/snapshots.rs` | `GPU init` | Same as widgets `snapshots.rs` — every test calls `harness_or_skip` → `RenderHarnessBuilder::build` → `wgpu::Instance::default`. Empirically confirmed. |

Comment-block shape for each new tag (≥ 3 lines per spec recipe):

```rust
// Skipped under Miri at the file level: <obstacle category>. The whole-file
// shape is required (not per-test `cfg_attr`) because <renderer dev-dep
// imports / FFI / GPU init / etc. reach the module regardless of which
// individual `#[test]` is enabled>. Alternative coverage: native
// `cargo test` exercises this file on the `gpu-tests` job.
#![cfg(not(miri))]
```

(c) **Files staying verbatim** (existing exemplars, AC11):

| File | Existing form | Action under v3 |
|---|---|---|
| `quartzite-widgets/tests/no_style_dep.rs` | `#![cfg(not(miri))]` + 9-line comment block at top (subprocess + isolation category) | **Unchanged.** Comment block already covers the v3 recipe's three points. Becomes a documented exemplar in `ai-docs/miri-policy.md`. |
| `quartzite-runtime/tests/timer.rs` | `#![cfg(not(miri))]` + 13-line comment block at top (interpreter budget category) | **Unchanged.** Same reason. Documented exemplar. |

(d) **Per-test `#[cfg_attr(miri, ignore = "<reason>")]`:** **zero new
occurrences.** The audit / dry-run finds no test that's both
(i) Miri-hostile AND (ii) in a file with other Miri-clean tests. Every
hostile case falls in a whole-file class (the three new `cfg(not(miri))`
above). The `cfg_attr` mechanism is documented in `ai-docs/miri-policy.md`
as the *default* — future contributors reach for it first.

### AGENTS.md cross-reference row — section placement

**Chosen section: `§ Build & Test`** (NOT `§ Agent Docs`).

Rationale:

- Mirrors the existing `ai-docs/dependency-versions.md` row, which lives at
  `AGENTS.md § Dependency Versions` (a *behaviour* section), not in `§ Agent
  Docs` (a doc-index). Miri policy is a build-behaviour concern, not a
  navigation-aid.
- `§ Build & Test` already references the Miri toolchain by listing
  `cargo +nightly-2026-05-01 miri test …` in its command table — the
  cross-reference row sits naturally alongside.
- `§ Agent Docs` is reserved per `AGENTS.md` for *agent-facing* doc rows;
  Miri policy is contributor-facing.

**Row form** (single line, fits the 35k early-warning band):

```markdown
> See [`ai-docs/miri-policy.md`](ai-docs/miri-policy.md) for the per-test `#[cfg_attr(miri, ignore = "…")]` default + per-file `#![cfg(not(miri))]` fallback + workflow-level exclusion-list contract.
```

**Placement within `§ Build & Test`:** immediately after the existing
fenced bash command block (the one containing `cargo build` / `cargo test`
/ `cargo miri test …`), before the AXIOM blockquote on `actionlint`. The
new row is a `> See [`…`](…)` blockquote (consistent with how
`AGENTS.md` cross-references other `ai-docs/*.md` files in adjacent
sections).

**Size budget:** AGENTS.md is currently 33,457 chars (`wc -c` at design
time, 2026-05-17). The row adds ~210 chars (one `> See [`…`](…) for …`
line). Post-edit size projects to **~33,670 chars**, ~1,330 under the 35k
early-warning band. Safe within AC12.

### Handoff plan

(a) Every group fans out through `/context-reset` per
`.claude/agents/design.md` § Rules → handoff-grouping requirement.
(b) Non-terminal groups are exactly 3 subtasks.
(c) Handoff destination: `/context-reset` per
`.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
(d) Terminal-group sizing: 1..=3 subtasks.

This design has **M = 6** subtasks → two groups (3 + 3).

- **Group A:** subtasks 1–3 — policy doc + opt-out source edits + AGENTS.md
  cross-reference. Entry into Group A spawns `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery
  (re-entry); parent `/task` proceeds inside the fresh subagent.
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — workflow edit + dry-run validation + INDEX
  row. Terminal group (3 subtasks; within the 1..=3 range). Entry into
  Group B also spawns `/context-reset` per the same SKILL section.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `ai-docs/miri-policy.md` with the full policy body: per-test `cfg_attr` default + six reason-prefix list + extension rule (new prefix requires editing this doc), per-file `cfg(not(miri))` fallback recipe (≥ 3-line comment block contract), workflow-level exclusion list + `# Why excluded:` comment requirement, audit recipe (`rg` regex + dry-run command), maintenance contract (new Miri-hostile tests get tagged at the source, NOT added to `miri.yml`), two exemplar pointers (`quartzite-widgets/tests/no_style_dep.rs`, `quartzite-runtime/tests/timer.rs`). **Name the per-file comment-block template explicitly** in a section anchored `## Per-file fallback recipe — comment-block template`, reproducing the exact 5-line skeleton shown in the design's *Per-crate / per-test tagging matrix (b)* (lines starting `// Skipped under Miri at the file level: …` through `#![cfg(not(miri))]`). Subtask 2 references this anchor by name so the contract is single-sourced and future PRs that widen the recipe edit one file only. Cross-reference back from `AGENTS.md § Build & Test` via a single blockquote row inserted between the fenced bash command block and the `actionlint` AXIOM blockquote. | `ai-docs/miri-policy.md` (new), `AGENTS.md` | — |
| 2 | Add `#![cfg(not(miri))]` + ≥ 3-line top-of-file comment block to the three renderer-touching integration test files. **Each comment block follows the template named in `ai-docs/miri-policy.md § Per-file fallback recipe — comment-block template` (Subtask 1's anchor) verbatim;** the three points the template requires are (a) the obstacle category (`GPU init` / `FFI`), (b) why per-test would not suffice (renderer dev-dep / FFI binding reaches the module before test selection), (c) the alternative-coverage path (native `cargo test` via `gpu-tests` job). Do NOT touch `quartzite-widgets/tests/no_style_dep.rs` or `quartzite-runtime/tests/timer.rs` (AC11). Do NOT touch `quartzite-macros/tests/via_facade.rs` — empirically Miri-clean. | `quartzite-widgets/tests/snapshots.rs`, `quartzite-widgets/tests/support_internals.rs`, `quartzite-style/tests/snapshots.rs` | 1 |
| 3 | Run local dry-run BEFORE touching the workflow file: `MIRIFLAGS='-Zmiri-tree-borrows -Zmiri-ignore-leaks -Zmiri-disable-isolation' cargo +nightly-2026-05-01 miri test --workspace --exclude quartzite-renderer`. Confirm exit 0 (modulo `continue-on-error` semantics — the local dry-run does NOT use that flag, so exit 0 here proves Miri-clean). Record the final test-summary line in the implementation commit message; the PR body re-records the command + the exit-0 confirmation per AC8. If exit ≠ 0, halt — surface the failure to the user before committing anything. | none modified (validation step) | 1, 2 |
| 4 | Edit `.github/workflows/miri.yml`: delete lines 82–113 — the 3-line dangling comment block at lines 82–84 (which references the v2 FFI-free `-p` allow-list audit and codifies the maintenance contract v3 explicitly retires) AND the seven `cargo miri test …` step blocks at lines 85–113 — replace the combined range with a single step block per the *Workflow diff shape* section above. The new step is preceded by a 5-line `# Why excluded:` comment block (renderer-only). Do NOT preserve lines 82–84 in any form (no rewrite, no relocation, no "see policy.md" stub — they are entirely subsumed by the new `# Why excluded:` block, which references `ai-docs/miri-policy.md` for the v3 contract). Job-level `env: MIRIFLAGS:`, `continue-on-error: true`, the master-only-comment header (lines 1–4), the `setup-rust-toolchain` block + `nightly-2026-05-01` pin, the Linux deps step (libfontconfig1-dev), the `cargo miri setup` step, and the `Export ImageVersion` step are ALL preserved verbatim. | `.github/workflows/miri.yml` | 3 |
| 5 | Run `actionlint .github/workflows/miri.yml` and confirm exit 0 (AGENTS.md *Build & Test* AXIOM). On any reported error, fix in-place and re-run. Re-run the local dry-run from Subtask 3 as a regression smoke test — the workflow edit is YAML-only, but the dry-run confirms the workflow command itself (`cargo miri test --workspace --exclude quartzite-renderer`) is what passed pre-edit. **Record the exact toolchain version used in the dry-run (the full `rustc +nightly-2026-05-01 --version --verbose` output, including commit hash and release date) in the PR body alongside the dry-run exit-0 confirmation.** This baselines the empirical claim that `cargo-miri` skips proc-macro lib unittests at runtime (Risks §3) against a specific nightly: any future toolchain bump's `/master-ci-failed` trace can compare against the merged baseline, and if the skip behaviour regresses the failing nightly is identified by diff against this recorded version. Record both confirmations (actionlint + dry-run) in the PR body. | none modified (validation step) | 4 |
| 6 | Add active-plan row to `ai-docs/plans/INDEX.md` per `/task` Step 4 convention. The row's `Crate(s)` column names `.github/workflows/miri.yml` + the three new `cfg(not(miri))` test files + `ai-docs/miri-policy.md` (new) + `AGENTS.md` (1 line) + `ai-docs/plans/INDEX.md` (this row). Status starts at 🟢 ready (spec + design merged in this PR); on `/task` Step 12 the row moves to `## Completed plans` with the standard `done/` path rewrite and final implementation summary. | `ai-docs/plans/INDEX.md` | 1, 2, 4 |

## Handoff plan

- **Group A:** subtasks 1–3 — policy doc, per-file tag sweep, local
  dry-run. Entry into Group A spawns `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery
  (re-entry); parent `/task` proceeds inside the fresh subagent. (3
  subtasks; within the 1..=3 range.)
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery
  (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — workflow edit, `actionlint` + dry-run
  re-validation, INDEX row. Terminal group (3 subtasks; within the
  1..=3 range). Entry into Group B also spawns `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery
  (re-entry).

## Risks

- **Risk:** First post-merge master Miri run surfaces a Miri-hostile test
  the local dry-run missed (toolchain mismatch, scheduling-dependent
  failure mode, ubuntu-vs-local CI env diff). **Mitigation:** Spec AC9
  envelope — any straggler tagging is filed as a follow-up issue, not
  silently absorbed into a CI-fix commit on this PR's merge-base. The
  `continue-on-error: true` posture (AC5, preserved verbatim) prevents
  the failure from turning master red while the follow-up issue is open.
- **Risk:** Future PR adds a new workspace crate. The workspace-wide
  invocation auto-picks it up — *good* by design, *but* if the new
  crate transitively reaches the renderer or has whole-crate FFI, the
  first master Miri run on that PR's merge will fail. **Mitigation:**
  `ai-docs/miri-policy.md` § Maintenance contract names this case
  explicitly: "adding a new workspace crate triggers a Miri audit of the
  new crate's test surface — tag at the source OR add a new
  `--exclude` clause with a design-phase justification". The contract
  text is part of Subtask 1.
- **Risk:** Cargo proc-macro lib unittest skip behaviour changes in a
  future nightly. Today `cargo-miri` exits 0 on "Running unit tests of
  'proc-macro' crates is not currently supported by Miri."; if a future
  nightly upgrades that to a non-zero exit, the `--workspace`
  invocation would fail at the macros-lib step. **Mitigation:** The
  nightly pin is `nightly-2026-05-01`; any future bump goes through the
  AGENTS.md `Dependency Versions` AXIOM live-registry-lookup rule, and
  the routine post-bump dry-run (re-running Subtask 3) catches the
  regression before merge. If it ever changes, the fallback is Shape A
  — `--exclude quartzite-macros` + a sibling re-include step.
- **Risk:** `actionlint` flags the multi-line `# Why excluded:` comment
  block. **Mitigation:** `actionlint` lints YAML structure + Action
  references + shell-quoting; YAML comments before a step key are not
  in its diagnostic surface. The five existing comment blocks in
  `miri.yml` (the master-only header, the `continue-on-error` rationale,
  the three `MIRIFLAGS` justification blocks, the v1-step audit
  reminder, the `+nightly` syntax warning) all currently pass
  `actionlint`. The new block is the same shape. If `actionlint` does
  flag it, fix in-place per Subtask 5's "on any reported error" clause.
- **Risk:** Per-file `#![cfg(not(miri))]` requires a comment block of ≥ 3
  lines per the spec recipe. Implementation accidentally writes a
  one-line marker. **Mitigation:** Subtask 2 enumerates the three
  required points (obstacle category / why per-test fails / alternative
  coverage) in its task body. `self-review` per `/task` Step 10 catches
  any short block as a `major` finding (recipe-violation severity per
  spec).
- **Risk:** AGENTS.md grows past 35k after the cross-reference row.
  **Mitigation:** Row size is ~210 chars; AGENTS.md is at 33,457 chars
  → projected ~33,670 (1,330-char headroom). Subtask 1's
  `self-review` checklist runs `wc -c AGENTS.md` post-edit (per
  AGENTS.md `instruction-file 35,000-char early warning` AXIOM); if
  the projection is wrong, the failure surfaces before push.
- **Risk:** Reviewer challenges the empirical claim that
  `via_facade.rs` is Miri-clean. **Mitigation:** The dry-run command +
  exit-0 + final test-summary line are recorded in the PR body per AC8
  (Subtask 3). The reviewer can reproduce locally with the recorded
  command.

## Test Design

This PR adds no production logic and no new test code. All "tests" are
in the deny-list-tagging form (`#![cfg(not(miri))]` or `#[cfg_attr(miri,
ignore = …)]`). The test-design surface is therefore the *audit + dry-run*
recipe, not a new `#[cfg(test)] mod tests`.

- **Audit (static enumeration):**
  - Location: terminal (`rg` invocation).
  - Entry point: the full `rg` regex command in *Audit recipe execution*
    above.
  - Scenarios: every workspace crate not in the exclusion list.
  - Fixtures: none (read-only filesystem scan).
- **Local dry-run (Subtask 3, re-run in Subtask 5):**
  - Location: terminal (`cargo miri test` invocation).
  - Entry point: `cargo +nightly-2026-05-01 miri test --workspace
    --exclude quartzite-renderer` under the workflow's `MIRIFLAGS`.
  - Scenarios:
    - **Happy path:** exit 0, test summary shows N passed / Y ignored /
      0 failed. The "Y ignored" count surfaces the proc-macro lib's
      runtime skip + any future `cfg_attr` ignores.
    - **Error case (compile failure on the workspace):** halt, surface
      compile error before any commit. Compile errors here would mean
      the deny-list shape introduced a syntactic regression (e.g.,
      mis-placed `#![cfg(not(miri))]`).
    - **Error case (Miri runtime error in a previously-untagged test):**
      halt. Either tag the new offender at the source (per-test
      `cfg_attr` if the file has mixed tests; per-file `cfg(not(miri))`
      if whole-file) OR escalate to a deferred follow-up if the case is
      genuinely out-of-scope.
  - Fixtures: requires `libfontconfig1-dev` (Linux) at runtime — same
    dep the workflow's `Install Linux dependencies` step installs;
    locally pre-installed via the developer's system package manager.
- **`actionlint` gate (Subtask 5):**
  - Location: terminal (`actionlint .github/workflows/miri.yml`).
  - Entry point: the single command.
  - Scenarios: exit 0 (happy) OR any error → fix per AGENTS.md AXIOM.
  - Fixtures: none.

No `#[cfg(test)] mod tests` block is added by this PR. None of the
files touched have substantial logic — the workflow YAML is config,
the test files get a single attribute + comment block, the policy doc
is pure prose, AGENTS.md gets one line, INDEX.md gets one row.

## Open questions

(All spec-side Open questions were resolved by the empirical dry-run.
Recorded here so the design-review and self-review trail show how each
was settled.)

- **Spec Open Q1 — Macros-crate inclusion mechanic.** *Resolved: Shape B.*
  Empirical dry-run confirms `cargo miri test --workspace --exclude
  quartzite-renderer` (a) compiles the macros lib's unittest binary as a
  host-target build, (b) lets `cargo-miri`'s runner skip it at runtime
  with the message "Running unit tests of 'proc-macro' crates is not
  currently supported by Miri." and exit 0, (c) compiles AND **runs**
  `quartzite-macros/tests/{extend,meta_enum,object,object_impl,via_facade}.rs`
  successfully under the workflow's `MIRIFLAGS`. No `#![cfg(not(miri))]`
  on `via_facade.rs`. No sibling `cargo miri test -p quartzite-macros
  --test …` step.
- **Spec Open Q2 — `cargo miri test --workspace` empirical behaviour on
  the proc-macro crate.** *Resolved.* See Q1 — the runner prints a
  non-error skip message + exit 0. Recorded verbatim in the Approach
  section.
- **Spec Open Q3 — `scripts/check-miri-tags.sh`.** *Out of scope per
  Deferred §3.* The manual `rg` recipe in *Audit recipe execution* is
  the v3 contract. Future v4 may revisit.
- **Spec Open Q4 — Doctest Miri coverage.** *Out of scope per Out of
  scope §1.* Native `cargo test --workspace --doc` continues to cover
  doctests. v3 does not change that.
- **Spec Open Q5 — AGENTS.md `§ Build & Test` AXIOM extension.**
  *Deferred per Deferred §4.* AGENTS.md is at 33,457 chars
  (~1,330 under the 35k early-warning band after this PR's row
  addition). A future post-extraction pass restores headroom; until
  then, the AGENTS.md row + `ai-docs/miri-policy.md` body are the
  workspace surface.
