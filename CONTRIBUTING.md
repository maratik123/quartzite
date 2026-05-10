# Contributing to quartzite

Thanks for considering a contribution. quartzite is a GUI and object framework
for Rust drawing on Qt's signals/slots and property/reflection model. This
document walks through the workflow external contributors need to land a PR.
It is a *summary* of the canonical workspace agent rules in
[`AGENTS.md`](AGENTS.md) — that file is the source of truth; this one is a
friendlier on-ramp.

## Workflow

- **Branch first.** Never edit on local `master` when the work is intended for
  a PR. Run `git checkout -b feat/YYYY-MM-DD-short-name` (or `fix/`, `chore/`,
  `docs/` prefix) *before* the first edit. Accumulating uncommitted edits on
  `master` is the failure mode this guards.
- **Branch naming.** Date-prefixed kebab-case under one of `feat/`, `fix/`,
  `chore/`, `docs/` — e.g. `feat/2026-05-08-project-docs`,
  `fix/2026-05-07-stop-before-run-race`.
- **Merge, don't squash.** PRs land via `gh pr merge --merge` (or the
  equivalent GitHub UI button). Squash- and rebase-merge are not used;
  per-commit history matters for `git bisect`.
- **Refresh `Cargo.lock`.** Run `cargo build` before every commit so a changed
  `Cargo.lock` lands in the same commit as the dependency edit that touched it.

See [`AGENTS.md` § Workflow](AGENTS.md#workflow) for the canonical rules.

## Staging discipline

Stage files **explicitly by name**:

```sh
git add path/to/file.rs path/to/another.rs
```

**Never** use `git add -A` or `git add .` — they pull in unintended files (IDE
state, accidental scratch files, secrets that slipped through `.gitignore`).

## Before commit

Run the full local gate before committing. CI runs the same set on every PR;
failures here mean failures there:

```sh
cargo build                                                                 # compiles
cargo test                                                                  # all green
cargo clippy --all-targets -- -D warnings                                   # strict lint
cargo fmt -- --check                                                        # formatting
cargo build -p quartzite --no-default-features                              # no_std + derive-free path
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace  # doc gate
```

## Workflow files

If your PR touches `.github/workflows/*.yml`, run `actionlint <file>` (or pass
every changed file in one invocation) and fix all errors before staging.
`actionlint` catches runner-version mismatches, deprecated action versions,
expression syntax errors, and shell quoting issues that `cargo` checks cannot
see.

See [`AGENTS.md` § Build & Test](AGENTS.md#build--test).

## Code style

Canonical reference: [`ai-docs/code-style.md`](ai-docs/code-style.md). The
most-likely-relevant rules:

- Source files are Rust under `src/`; max 100 cols (rustfmt default); format
  via `cargo fmt`, never `rustfmt <file>` directly.
- `#![deny(missing_docs)]` is enabled workspace-wide; every public item gets a
  `///` summary.
- Errors use `thiserror`; hand-rolled `Display`/`Error` is reserved for cases
  the derive cannot express.
- No blanket `#[allow]` without justification.
- Mark recursively-simple fns with `#[inline]` (concrete) or `_Simple._`
  (generic / trait-impl) per the canonical rule.

## Documentation conventions

Canonical reference: [`ai-docs/doc-convention.md`](ai-docs/doc-convention.md).
Every public item documents in third-person present indicative + has a
`# Examples` block; `# Parameters` / `# Errors` / `# Panics` / `# Safety`
sections appear in that strict order when applicable. The `_Simple._` doc tag
and `document_features` placement rules also live here.

## Tests

Canonical reference:
[`AGENTS.md` § Rust Test Conventions](AGENTS.md#rust-test-conventions). Unit
tests live under `#[cfg(test)] mod tests`; integration tests live in `tests/`;
use [`rstest`](https://crates.io/crates/rstest) for parameterised tests,
[`mockall`](https://crates.io/crates/mockall) for trait mocks, and
[`pretty_assertions`](https://crates.io/crates/pretty_assertions) for diffs.
Test names are `snake_case` describing behaviour
(`returns_empty_when_not_found`).

Files under `examples/` and criterion bench files under `benches/` (declared
with `[[bench]] harness = false`) are exempt from the `#[cfg(test)] mod tests`
requirement.

## GPU snapshot tests

The `quartzite-widgets` crate ships an integration-test suite at
`quartzite-widgets/tests/snapshots.rs` that renders each built-in widget
through an offscreen `RenderHarness` (in `quartzite-renderer`) and compares
the readback against committed PNG goldens under
`quartzite-widgets/tests/snapshots/<backend>/`. The Linux lane of the CI
`gpu-tests` job is required at PR merge time; Windows / macOS lanes are
non-blocking until follow-up PRs bootstrap their per-backend goldens.

A separate Linux-only smoke test (`quartzite-renderer/tests/xvfb_smoke.rs`)
exercises the full windowed pipeline (`WindowedApplication` + a real winit
`EventLoop`) under `xvfb-run`. It asserts only on clean startup + clean exit
(no pixel comparison).

### Run snapshots locally

```sh
# Linux (vulkan + lavapipe via mesa-vulkan-drivers)
WGPU_BACKEND=vulkan cargo test -p quartzite-widgets --test snapshots

# Windows (dx12 + WARP)
WGPU_BACKEND=dx12 cargo test -p quartzite-widgets --test snapshots

# macOS (metal)
WGPU_BACKEND=metal cargo test -p quartzite-widgets --test snapshots
```

The backend dir is derived from `WGPU_BACKEND`; an unset value resolves to
`auto` so locally-bootstrapped goldens don't accidentally land in the CI
backend directories.

### Skip GPU work

When you don't have a working GPU adapter locally (or just want a fast
`cargo test` cycle), set the workspace-wide skip env:

```sh
SKIP_RENDER_SNAPSHOT=1 cargo test --workspace
```

The flag is honoured by every snapshot test, the offscreen harness's GPU
smoke test, and the `xvfb_smoke` integration test. Each prints a clear
`eprintln!` notice and passes. The CI `test` job (the non-GPU lane) sets
this env so its runtime is unaffected by the snapshot suite.

### Regenerate goldens

`scripts/update-snapshots.sh` regenerates the committed PNGs:

```sh
# Auto-detect from `uname` / WGPU_BACKEND
scripts/update-snapshots.sh

# Or force a specific backend
scripts/update-snapshots.sh --backend vulkan
scripts/update-snapshots.sh --backend dx12
scripts/update-snapshots.sh --backend metal
```

The script sets `QUARTZITE_REGENERATE_SNAPSHOTS=1` and runs the snapshot
suite; the helper writes new PNGs into `tests/snapshots/<backend>/`
instead of comparing.

### Intentional visual diffs

When a PR's code change *intentionally* alters rendered pixels:

1. Push the code change to the feature branch and let CI run.
2. The Linux `gpu-tests` lane fails. Open the failing run, download the
   `gpu-snapshot-failures-Linux` artifact, and inspect each `*.actual.png`
   / `*.diff.png` next to its golden. Confirm the change is the one you
   intended.
3. Run `scripts/update-snapshots.sh --backend vulkan` locally and commit
   the regenerated PNGs in the same PR. Reviewers see the new pixels in
   the diff.
4. For Windows / macOS lanes (currently non-blocking), bootstrap their
   goldens in a follow-up PR — contributors with access to those
   platforms run the regen script and commit
   `tests/snapshots/{dx12,metal}/`.

See [`AGENTS.md` § Build & Test](AGENTS.md#build--test) for the strict
gate that runs against every PR.

## License

Submitting a contribution means agreeing to dual-license it under MIT and
Apache-2.0:

> Unless you explicitly state otherwise, any contribution intentionally
> submitted for inclusion in the work by you, as defined in the Apache-2.0
> license, shall be dual-licensed as above, without any additional terms or
> conditions.

Full texts: [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

## For agent-driven development

This repository uses Claude Code workflows under `.claude/skills/` and
`.claude/agents/`. Agents follow [`AGENTS.md`](AGENTS.md) — the canonical
workspace agent-instruction file — and a corrections log lives at
[`ai-docs/learnings.md`](ai-docs/learnings.md). Human contributors are
encouraged to read AGENTS.md but are not required to follow agent-only
sections (`## Corrections Log`, `## Propagation Rule`) which exist for the
agent workflow.
