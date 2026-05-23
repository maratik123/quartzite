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
through an offscreen `RenderHarness` (in `quartzite-renderer`) and
compares the readback against committed PNG goldens under
`quartzite-widgets/tests/snapshots/`.

Layout:

- `tests/snapshots/shared/<name>.png` — cross-backend default. Used when
  no per-backend override exists. While the renderer is no-op, every
  backend (vulkan, dx12, metal, …) writes byte-identical clear-colour
  pixels, so a single shared golden covers all three CI lanes.
- `tests/snapshots/<backend>/<name>.png` — per-backend override. Created
  when one backend produces pixels that drift beyond `FLIP_TOLERANCE`
  from the shared golden (typical once `VelloPainter` actually
  rasterizes content).

Lookup order is "backend override → shared default → fail". All three
CI `gpu-tests` lanes (Linux/vulkan, Windows/dx12, macOS/metal) are
required at PR merge time.

The Windows lane mirrors `gfx-rs/wgpu`'s own CI: it installs a fresh
**WARP 1.0.19** redistributable, **DXC v1.9.2602**, and the **D3D12
Agility SDK 1.619.2** before running the snapshot suite, because vello
requires modern D3D12 runtime features that `windows-latest` does not
ship by default. Without the install dance the snapshot suite crashes
with `STATUS_ACCESS_VIOLATION` during the first compute dispatch. The
DLLs are placed next to the test binary (`target/debug/` +
`target/debug/deps/`) so wgpu's DLL search order picks them up at run
time.

A separate Linux-only smoke test (`quartzite-renderer/tests/xvfb_smoke.rs`)
exercises the full windowed pipeline (`WindowedApplication` + a real winit
`EventLoop`) under `xvfb-run`. It asserts only on clean startup + clean exit
(no pixel comparison).

### Required tooling

You can short-circuit all of this with `SKIP_RENDER_SNAPSHOT=1` (see
*Skip GPU work* below) and contribute without installing anything. The
following table lists what you need only if you actually want to **run**
the GPU-touching tests locally:

| Tool | Purpose | When required |
|---|---|---|
| Vulkan ICD (driver) | offscreen snapshot tests use a real or software Vulkan adapter via wgpu / vello | unless `SKIP_RENDER_SNAPSHOT=1` |
| `vulkaninfo` | adapter-enumeration diagnostics; useful when wgpu fails to pick the adapter you expected | optional |
| **lavapipe** (software Vulkan, ships in mesa) | reproduces the *exact* adapter the CI Linux lane uses (`WGPU_ADAPTER_NAME=llvmpipe`) | only for full CI parity |
| `xvfb` + `xvfb-run` | hosts the Linux `xvfb_smoke` test under a virtual X server | only when running `--test xvfb_smoke` against a headless display |
| `actionlint` | strict gate before staging any `.github/workflows/*.yml` change (see [`AGENTS.md`](AGENTS.md)) | only when editing workflow files |

#### Install — Linux

CI's Ubuntu lane installs `mesa-vulkan-drivers vulkan-tools xvfb` (one
line in [`.github/workflows/ci.yml`](.github/workflows/ci.yml)) — the
same set on any apt-based distro is enough:

```sh
# Debian / Ubuntu
sudo apt-get install -y \
  mesa-vulkan-drivers vulkan-tools libvulkan1 xvfb libxkbcommon-x11-0
```

```sh
# Fedora
sudo dnf install -y mesa-vulkan-drivers vulkan-tools xorg-x11-server-Xvfb
```

```sh
# Arch
sudo pacman -S vulkan-mesa-layers vulkan-tools xorg-server-xvfb
```

```sh
# Gentoo
sudo emerge -av media-libs/mesa dev-util/vulkan-tools \
                x11-misc/xvfb-run x11-base/xorg-server
# Required USE flags / VIDEO_CARDS:
#   media-libs/mesa     vulkan          (always)  +  VIDEO_CARDS=lavapipe (CI parity)
#   x11-base/xorg-server xvfb           (provides /usr/bin/Xvfb)
```

`actionlint` is not in the main Portage tree; install via `go install
github.com/rhysd/actionlint/cmd/actionlint@latest` (or the equivalent
release binary on any OS).

#### Install — macOS

The offscreen suite uses Metal; no install is needed. The `xvfb_smoke`
test is `#[cfg(target_os = "linux")]` and compiles to a no-op stub on
macOS — nothing to set up. Install `actionlint` via `brew install
actionlint` only if you edit `.github/workflows/*.yml`.

#### Install — Windows

The offscreen suite uses DX12 / WARP; no install is needed. The
`xvfb_smoke` test is Linux-only and compiles to a no-op stub. Install
`actionlint` via `winget install rhysd.actionlint` (or `scoop install
actionlint`) only if you edit `.github/workflows/*.yml`.

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

### Reproduce the CI Linux lane locally

When debugging a CI-only failure, run the exact same env the
`gpu-tests` Linux lane uses (`WGPU_ADAPTER_NAME=llvmpipe` + software
GL):

```sh
WGPU_BACKEND=vulkan WGPU_ADAPTER_NAME=llvmpipe LIBGL_ALWAYS_SOFTWARE=1 \
  cargo test -p quartzite-widgets --test snapshots

timeout 60 xvfb-run -a cargo test -p quartzite-renderer --test xvfb_smoke
```

The `WGPU_ADAPTER_NAME` filter requires lavapipe to be present in your
mesa build — verify with `vulkaninfo --summary | grep llvmpipe`. On
Gentoo this is `VIDEO_CARDS="… lavapipe"`; on Debian / Ubuntu it ships
in `mesa-vulkan-drivers` by default; same for Fedora's
`mesa-vulkan-drivers` and Arch's `vulkan-mesa-layers`.

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
instead of comparing. Regenerated PNGs are *per-backend overrides* — to
seed (or refresh) the cross-backend `tests/snapshots/shared/` default,
run the regen with one backend and then `mv tests/snapshots/<backend>/*
tests/snapshots/shared/`.

### Intentional visual diffs

When a PR's code change *intentionally* alters rendered pixels:

1. Push the code change to the feature branch and let CI run.
2. The Linux `gpu-tests` lane fails. Open the failing run, download the
   `gpu-snapshot-failures-Linux` artifact, and inspect each `*.actual.png`
   / `*.diff.png` next to its golden. Confirm the change is the one you
   intended.
3. Run `scripts/update-snapshots.sh --backend vulkan` locally and commit
   the regenerated PNGs in the same PR.
   - If the new pixels are uniform across all backends (typical for the
     no-op renderer state), `mv` the result from `tests/snapshots/vulkan/`
     to `tests/snapshots/shared/`. One commit covers all CI lanes.
   - If the pixel change is backend-specific (e.g. text rendering drift
     on macOS only), commit the override under
     `tests/snapshots/<backend>/` and leave `shared/` alone. The lookup
     prefers overrides over shared.
4. Reviewers see the new pixels in the PR diff.
5. For Windows / macOS overrides, contributors with access to those
   platforms run the regen script there and commit the resulting
   `tests/snapshots/{dx12,metal}/` files. Until that lands, the shared
   golden is what their CI lane compares against.

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

### Code-search index (`ast-index`)

Agent code-search runs through `ast-index` per
[`.claude/rules/ast-index.md`](.claude/rules/ast-index.md). The upstream
repository
[`defendend/Claude-ast-index-search`](https://github.com/defendend/Claude-ast-index-search)
hosts the installation instructions and the full command reference. Once
the binary is on `PATH`, the `SessionStart` hook in
[`.claude/settings.json`](.claude/settings.json) refreshes the index
automatically on each session open. Human contributors do **not** need
`ast-index` installed for `cargo build` / `cargo test` — it is an
agent-only tool.

### Rust LSP (`rust-analyzer`)

`ast-index` is syntactic (tree-sitter-based); `rust-analyzer` is the
semantic complement. Use it for queries that need type inference, trait
resolution, or precise reference chasing — `goToDefinition`,
`findReferences`, `goToImplementation`, `hover`, call hierarchy.

Claude Code exposes an embedded `LSP` Tool (deferred — load its schema
once per session via `ToolSearch query="select:LSP"`). It routes through
the locally-configured LSP server for the file type, so a
`rust-analyzer` binary on `PATH` is required for any operation against
`.rs` files. Operations available: `goToDefinition`, `findReferences`,
`hover`, `documentSymbol`, `workspaceSymbol`, `goToImplementation`,
`prepareCallHierarchy`, `incomingCalls`, `outgoingCalls`. All operations
take `filePath` + 1-based `line` + 1-based `character`. Prefer `LSP`
over `ast-index` whenever the question is semantic (e.g. "every `impl`
of trait `Style`" → `LSP goToImplementation` beats `ast-index
implementations` for a trait defined in a macro-generated context);
fall back to `ast-index` when the LSP server returns no result or the
symbol is macro-expanded beyond `rust-analyzer`'s reach.
