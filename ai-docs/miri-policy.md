# Miri policy

Workspace-wide deny-list contract for `.github/workflows/miri.yml`. Miri runs
across every workspace crate by default; individual tests / files / crates opt
**out** at the source.

## Workflow command

`miri.yml` runs on push to master and on pull requests targeting master,
with a single primary invocation:

```bash
MIRIFLAGS='-Zmiri-tree-borrows -Zmiri-ignore-leaks -Zmiri-disable-isolation' \
  cargo miri test --workspace --exclude quartzite-renderer
```

No `-p` allow-list, no per-test `--test` allow-list. Adding a new workspace
crate is automatically in scope; the maintenance contract is "tag at the
source", not "edit the workflow".

## Per-test default — `#[cfg_attr(miri, ignore = "<reason>")]`

Default opt-out shape. The `#[test]` fn stays in the harness's discovery list,
surfaces in the `ignored` count of the run summary, and the reason string is
visible to anyone reading the Miri job output.

```rust
#[test]
#[cfg_attr(miri, ignore = "interpreter budget — 10k-iteration stress loop")]
fn loop_stress() {
    // …
}
```

The reason string MUST start with one of the six closed category prefixes:

| Prefix | When to use |
|---|---|
| `"interpreter budget — "` | Wall-clock-bounded assertion, stress / long-loop test, anything where Miri's 10–30× overhead pushes the assertion past its timeout. |
| `"FFI — "` | Test (transitively) calls a C foreign function Miri cannot interpret. |
| `"GPU init — "` | Test (transitively) initialises wgpu / vulkan / a GPU adapter. |
| `"subprocess — "` | Test spawns a child process (`std::process::Command`, `cargo` shell-out). |
| `"isolation — "` | Test makes a syscall Miri's isolation layer cannot emulate (`pidfd_spawnp`, certain filesystem / network syscalls). |
| `"proc-macro host — "` | Test depends on running a proc-macro at Miri-interpretation time. |

### Extension rule — adding a new prefix

The prefix list is **closed**. Adding a seventh prefix requires editing this
file in the same PR that introduces the first use. A free-text reason after a
documented prefix is fine; inventing an undocumented prefix is not.

## Per-file fallback — `#![cfg(not(miri))]`

Used when the **entire** integration test file depends on something Miri
cannot reach — typical cases:

- A `mod support;` import that pulls a renderer dev-dep / FFI binding into the
  test binary's link graph regardless of which `#[test]` is enabled.
- Every test in the file shares the same Miri-hostile setup (e.g., every test
  builds a `RenderHarness`).
- The file as a whole has no per-test mix of Miri-clean and Miri-hostile
  cases — there are no clean tests left if you skip the hostile ones.

The whole-file shape is **required** (not per-test `cfg_attr`) when the
top-of-file `use` declarations themselves reach Miri-hostile code: per-test
gating cannot prevent compilation of the offending import.

## Per-file fallback recipe — comment-block template

Every per-file opt-out carries a top-of-file `//` comment block of **≥ 3
lines** documenting (a) the obstacle category, (b) why per-test was not
sufficient, (c) the alternative coverage path. The exact 5-line skeleton:

```rust
// Skipped under Miri at the file level: <obstacle category>. The whole-file
// shape is required (not per-test `cfg_attr`) because <renderer dev-dep
// imports / FFI / GPU init / etc. reach the module regardless of which
// individual `#[test]` is enabled>. Alternative coverage: native
// `cargo test` exercises this file on the `gpu-tests` job.
#![cfg(not(miri))]
```

Fill in `<obstacle category>` with one of the six documented prefixes (sans
the trailing em-dash); fill in the `<…>` placeholder with the concrete
reason. Longer blocks are welcome when the reason needs more space — see the
exemplars below.

## Exemplars (preserved verbatim)

Two existing files document the v3 fallback recipe by example. Their comment
blocks already match this template and are NOT rewritten:

- [`quartzite-widgets/tests/no_style_dep.rs`](../quartzite-widgets/tests/no_style_dep.rs)
  — subprocess + isolation category. Shells out to `cargo tree` via
  `std::process::Command`; the spawn path routes through `pidfd_spawnp` which
  Miri does not emulate.
- [`quartzite-runtime/tests/timer.rs`](../quartzite-runtime/tests/timer.rs)
  — interpreter budget category. Every assertion is wall-clock bounded
  (200 ms / 500 ms timeouts after 30 ms-interval timers); Miri's interpreter
  overhead cannot preserve those budgets.

## Workflow-level exclusion list

Crates excluded at the workflow level — closed set, single entry under v3:

| Crate | Why excluded |
|---|---|
| `quartzite-renderer` | Prod-deps `winit` / `wgpu` / `vello` reach the lib by construction; no `--lib` / `--test` filter side-steps the dep graph. |

Each `--exclude <crate>` in `miri.yml` carries an **immediately-preceding**
`# Why excluded: <reason>` comment naming the irreducible obstacle. Adding a
new crate to the exclusion list requires a design-phase justification, not
just a workflow edit — the deny-list maintenance contract is enforced at the
source by default; workflow-level exclusion is the last resort.

`quartzite-macros` is **not** excluded. Miri cannot host a proc-macro
expansion, but `cargo-miri`'s runner skips proc-macro lib unittests at
runtime with the message:

> `Running unit tests of 'proc-macro' crates is not currently supported by Miri.`

…and exits 0. The crate's integration tests (`tests/extend.rs`,
`tests/meta_enum.rs`, `tests/object.rs`, `tests/object_impl.rs`,
`tests/via_facade.rs`) run under Miri unchanged.

## Audit recipe

The recipe below is the manual `rg` + local dry-run procedure for the next
contributor adding a workspace crate or widening Miri's reach. It is
deliberately not automated under v3; a future `scripts/check-miri-tags.sh`
candidate is recorded in `ai-docs/deferred/_inbox.md`.

### Static enumeration

List every `#[test]` / `#[rstest]` attribute in the workspace
(non-excluded crates only):

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
`<file>:<lineno>:<matched-attribute>`. The result is the attribute-occurrence
list (not unique-test count; `#[rstest]` cases multiply at the parametric
level). Manually triage each match against the three classes:

1. **Miri-clean** — leave untouched.
2. **Interpreter-budget / wall-clock-bounded** — per-test
   `#[cfg_attr(miri, ignore = "interpreter budget — …")]`.
3. **FFI / GPU / subprocess / isolation / proc-macro host** — promote to
   per-file `#![cfg(not(miri))]` if the whole file is in that class;
   otherwise per-test `cfg_attr` with the matching prefix.

### Local dry-run

Before committing the tagging sweep, run:

```bash
MIRIFLAGS='-Zmiri-tree-borrows -Zmiri-ignore-leaks -Zmiri-disable-isolation' \
  cargo +nightly-2026-05-01 miri test --workspace --exclude quartzite-renderer
```

Must exit 0. Record the final test-summary line (`X passed; Y ignored; 0 failed`)
and elapsed wall time in the PR body. If exit ≠ 0, halt — the run identifies a
straggler that needs a source-level tag (or, rarely, a new workflow-level
exclusion with design-phase justification).

## Maintenance contract

The deny-list shape inverts the v2 maintenance burden. New test code is the
caller; the workflow does not need to be edited.

| Situation | Action |
|---|---|
| New test added that Miri can run | No action. The `--workspace` invocation picks it up automatically. |
| New test added that Miri cannot run (FFI / GPU / wall-clock / etc.) | Tag at the source with `#[cfg_attr(miri, ignore = "<prefix> — <reason>")]`. Do NOT edit `miri.yml`. |
| New integration test file where every test is Miri-hostile | Per-file `#![cfg(not(miri))]` + the comment-block template above. |
| New workspace crate added that transitively reaches the renderer | Audit the new crate's test surface (the recipe above); tag at the source. If the crate has whole-lib FFI / GPU init that no per-file fallback can cover, the workflow-level exclusion list grows by one entry — but this requires a design-phase justification in the PR introducing the crate. |
| First master Miri run after merge surfaces an untagged hostile test | File a follow-up issue per AC9 envelope. Do NOT silently absorb the fix into a CI-fix commit on the merging PR. The `continue-on-error: true` posture (preserved verbatim under v3) prevents the failure from turning master red while the follow-up is open. |
| New reason-string prefix needed | Edit this file's "Per-test default" section and add a row to the prefix table. Do NOT invent an undocumented prefix in the test source. |
