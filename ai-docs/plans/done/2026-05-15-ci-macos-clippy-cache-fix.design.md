# Design: CI macOS Clippy `rustup-init` cache pollution fix

**Issue:** #340
**Date:** 2026-05-15

## Approach

### Root-cause summary (verified)

`actions-rust-lang/setup-rust-toolchain@v1` runs its steps in this order
(verified live against the action's `action.yml` `runs:` section, 2026-05-15):

1. `Unbork mac` (install bash on macOS).
2. Construct rustup flags.
3. Set Rust env vars (`CARGO_INCREMENTAL`, `RUSTFLAGS`, …).
4. Install Rust problem matcher.
5. **Install rustup** (downloads `rustup-init`; adds `${CARGO_HOME:-$HOME/.cargo}/bin` to `PATH`).
6. **`rustup toolchain install`** + components/targets/override.
7. Print installed versions; emit cache key.
8. Downgrade registry protocol (if needed).
9. **`Swatinem/rust-cache@v2`** — `cache-bin` defaults to `true`, so
   `${CARGO_HOME}/bin` is restored from the cache **after** the toolchain
   install of step 6.

The consequence: a polluted `~/.cargo/bin/cargo` shim baked into a cache
on macOS image `15.7.4` (where some Homebrew or rustup-init quirk left
`cargo` as a `rustup-init` symlink instead of the real cargo shim) is
restored onto a fresh `15.7.5` runner **after** the toolchain install
already deposited a working binary. The cache restore wins, and the
job's later `cargo clippy` invocation resolves to `rustup-init`, which
emits `error: unexpected argument 'clippy' found`. Therefore the
cache-key change is the **whole** mitigation, not belt-and-braces — the
spec's "either way" framing in Technical Constraints can be tightened
to "the action does NOT overwrite the shim; cache-key change is the
entire fix".

### Chosen mitigation

Append the GitHub-hosted runner image-version env var `ImageVersion`
to every `cache-shared-key` in every workflow. The variable is
deliberately set by `actions/runner-images` build scripts
(verified live, 2026-05-15):

- **Linux (ubuntu-latest):** `images/ubuntu/scripts/build/configure-environment.sh` calls
  `set_etc_environment_variable "ImageVersion" "${IMAGE_VERSION}"` — written to `/etc/environment`,
  read by the runner into the workflow environment.
- **Windows (windows-latest):** `images/windows/scripts/build/Configure-SystemEnvironment.ps1`
  performs the equivalent set-machine-environment registration.
- **macOS (macos-latest):** `images/macos/scripts/build/configure-preimagedata.sh` appends
  `export ImageVersion=$image_version` to `~/.bashrc`. The shell-init route is the documented
  exposure path; community workflows depend on `${{ env.ImageVersion }}` resolving on
  macOS in production. The image-version segment will appear in cache keys as the
  current image's version string (e.g. `20251104.1`); on a runner where the variable
  cannot be expanded (theoretical edge case), the segment becomes empty — the cache
  key then degrades to the pre-fix shape on that one runner, which is no worse than today.

Concretely, every `cache-shared-key` value is rewritten from
`<existing-key>-v2` to `<existing-key>-v2-${{ env.ImageVersion }}`. The
image-version segment goes at the **end** of the key (after the `-v2`
generation suffix and, in the `features` job, after the
`-${{ matrix.features }}` partition segment) so that:

- per-OS / per-feature partitioning is preserved (cache buckets do not
  merge);
- a runner image bump invalidates every OS leg's cache automatically;
- the existing `-v2` generation suffix retains its role as a
  human-triggered manual bust knob.

### Diagnostic step

A new step named `Verify cargo identity` runs on every cargo-running
job, immediately after the `setup-rust-toolchain` step and before the
first `cargo <subcommand>` step:

```yaml
- name: Verify cargo identity
  shell: bash
  run: |
    # See #340 — macOS cache pollution made `cargo` resolve to
    # rustup-init. A real cargo prints a version like "cargo 1.xx.0
    # (...)"; rustup-init prints argparse errors. Fail loudly here
    # rather than letting a confusing argparse error surface from the
    # first real cargo subcommand.
    cargo --version
    rustc --version
```

`shell: bash` is mandated explicitly because the default shell on
Windows is PowerShell — heredoc-style multi-line `run:` blocks
behave inconsistently across the matrix without an explicit shell.
`bash` is preinstalled on all three GitHub-hosted runner OS legs.

### Near-step comment (AC5)

Every modified `cache-shared-key` line and every new `Verify cargo
identity` step carries a near-step comment of the shape:

```yaml
# Why ImageVersion in the cache key + the cargo-identity check: see
# issue #340 — a polluted ~/.cargo/bin/cargo shim was being restored
# from a macos-15.7.4 cache onto a macos-15.7.5 runner, leaving cargo
# resolving to rustup-init. The image-version segment busts the cache
# on every image bump; the identity check turns a recurrence into a
# visible failure rather than a confusing argparse error.
```

The comment block lives **once per workflow file** (above the first
modified site in each file) and the other modified sites reference
issue #340 in a one-line comment, to avoid bloating the file.

### Format job (AC2 enumeration vs. "every cargo-running job")

Spec scope item #3 enumerates `build, test, clippy, gpu-tests, docs,
features` in `ci.yml` and explicitly limits the cache-key fix to jobs
that use a `cache-shared-key`. The `format` job uses
`actions-rust-lang/setup-rust-toolchain@v1` and runs `cargo fmt`, but
does **not** use a `cache-shared-key`. Two reads of AC2 are possible:

- **Literal AC2 ("every cargo-running job"):** add a diagnostic step
  to `format` as well.
- **Spec-enumeration AC2:** only the six enumerated jobs get the
  diagnostic step; `format` is excluded.

This design follows the **literal AC2** read and adds the diagnostic
step to `format` too. Rationale: AC2's stated purpose is to make any
future recurrence of the same vector surface as a visible signal. The
`format` job runs `cargo fmt` — the same `cargo` shim resolution that
broke `clippy` would break `fmt` identically, and the marginal cost of
two extra echoed lines is negligible. The cache-key fix is **not**
applied to `format` (no `cache-shared-key` to change). Flagged as
Open question 1 below so the reviewer / product owner can over-rule
if the spec enumeration was deliberate.

### Rejected alternatives

| Alternative | Why rejected |
|---|---|
| Insert a `rustup default stable && which cargo` step that hard-overwrites `~/.cargo/bin/cargo` after the cache restore | Cures the symptom but assumes `rustup` itself isn't polluted. Wider blast radius (PATH manipulation). Cache-key change addresses the root cause. |
| Switch to `dtolnay/rust-toolchain` (no caching layer) | Loses cache benefits across the workspace; orthogonal to root cause. Issue body's secondary hypothesis; explicitly out of scope per spec. |
| Bump the manual `-v2` suffix to `-v3` | One-time bust only — does not protect against the next image-version bump. The image-version segment is self-busting per image; the `-v2` knob retains its independent manual-bust role. Explicitly out of scope per spec. |
| Disable `cache-bin` (set `cache-bin: false` on every `setup-rust-toolchain` step) | Solves the pollution vector at the cost of needing to redownload every cargo-installed binary on every job. Higher steady-state cost than the cache-key change. |
| Compose `ImageVersion` only into the macOS key | Leaves Ubuntu and Windows exposed to the same vector. Spec Q2 explicitly chose the defensive uniform approach. |

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 0 | **Pre-merge probes (must pass before Task 1).** (a) Validate that `${{ env.ImageVersion }}` is accepted by `actionlint`: write a minimal throwaway YAML fragment (`/tmp/probe.yml`) containing exactly `cache-shared-key: foo-${{ env.ImageVersion }}`, run `actionlint /tmp/probe.yml`. If actionlint flags it, adopt the `env:` alias strategy (`env: { CACHE_IMAGE_VERSION: ${{ env.ImageVersion }} }` per job + `${{ env.CACHE_IMAGE_VERSION }}` in the key) and update Tasks 1–3. (b) Verify `${{ env.ImageVersion }}` resolves non-empty on macOS by inspecting the Actions runner docs or a live PR run log showing `Using cache key:` for `macos-latest`. If the segment is empty on macOS, switch the macOS leg to a shell-step that exports the variable from `~/.bashrc` into `$GITHUB_ENV` before the `setup-rust-toolchain` step. Record the probe result in the Decisions log. | `/tmp/probe.yml` (throwaway) | — |
| 1 | Append `-${{ env.ImageVersion }}` to every `cache-shared-key` in `ci.yml` (six sites: `build`, `test`, `clippy`, `gpu-tests`, `docs`, `features`). For `features`, place the segment **after** `-${{ matrix.features }}` to preserve per-feature partitioning. Add the near-step rationale comment block above the first modified site. | `.github/workflows/ci.yml` | 0 |
| 2 | Append `-${{ env.ImageVersion }}` to the `cache-shared-key` in `coverage.yml`. Add a one-line `# See #340` comment. | `.github/workflows/coverage.yml` | 0 |
| 3 | Append `-${{ env.ImageVersion }}` to the `cache-shared-key` in `docs.yml`. Add a one-line `# See #340` comment. | `.github/workflows/docs.yml` | 0 |
| 4 | Add the `Verify cargo identity` step (running `cargo --version && rustc --version` under `shell: bash`) immediately after the `setup-rust-toolchain` step on every cargo-running job in `ci.yml`: `format`, `build`, `test`, `clippy`, `gpu-tests`, `docs`, `features`. Place the rationale comment block above the step on the `clippy` job (most thematic to #340); other jobs carry a one-line `# See #340` comment. | `.github/workflows/ci.yml` | 1 |
| 5 | Add the same `Verify cargo identity` step to the cargo-running job in `coverage.yml` (the single `coverage` job; placed after `setup-rust-toolchain` and before `Install cargo-llvm-cov`). Add a one-line `# See #340` comment. | `.github/workflows/coverage.yml` | 2 |
| 6 | Add the same `Verify cargo identity` step to the `build` (docs) job in `docs.yml`, placed after `setup-rust-toolchain` and before `Build docs`. Add a one-line `# See #340` comment. | `.github/workflows/docs.yml` | 3 |
| 7 | Run `actionlint .github/workflows/ci.yml .github/workflows/coverage.yml .github/workflows/docs.yml` and resolve any reported issues. **Required gate** per AGENTS.md AXIOM. | (verification only) | 1, 2, 3, 4, 5, 6 |

Task count: 8 (Task 0 + Tasks 1–7; all tasks are mechanical and
file-local). No splitting into multiple issues — the eight tasks
form one atomic CI mitigation.

### Per-site reference table (Task 1)

| ci.yml line | Current key | New key |
|---|---|---|
| 65 (`build`) | `${{ runner.os }}-stable-v2` | `${{ runner.os }}-stable-v2-${{ env.ImageVersion }}` |
| 100 (`test`) | `${{ runner.os }}-stable-v2` | `${{ runner.os }}-stable-v2-${{ env.ImageVersion }}` |
| 132 (`clippy`) | `${{ runner.os }}-stable-v2` | `${{ runner.os }}-stable-v2-${{ env.ImageVersion }}` |
| 177 (`gpu-tests`) | `${{ runner.os }}-stable-gpu-v2` | `${{ runner.os }}-stable-gpu-v2-${{ env.ImageVersion }}` |
| 368 (`docs`) | `${{ runner.os }}-stable-v2` | `${{ runner.os }}-stable-v2-${{ env.ImageVersion }}` |
| 416 (`features`) | `${{ runner.os }}-stable-features-${{ matrix.features }}-v2` | `${{ runner.os }}-stable-features-${{ matrix.features }}-v2-${{ env.ImageVersion }}` |

`coverage.yml` line 36: `${{ runner.os }}-cargo-coverage-v2` → `${{ runner.os }}-cargo-coverage-v2-${{ env.ImageVersion }}`.

`docs.yml` line 32: `${{ runner.os }}-cargo-v2` → `${{ runner.os }}-cargo-v2-${{ env.ImageVersion }}`.

## Risks

- **`ImageVersion` undefined on macOS workflow context** (low). Direct
  evidence: Linux `set_etc_environment_variable` and Windows
  `Configure-SystemEnvironment.ps1` write `ImageVersion` to OS-wide
  environment stores; macOS only appends to `~/.bashrc`. If the
  Actions runner on macOS does not source `~/.bashrc` when populating
  the workflow expression context (runner uses `bash --noprofile --norc`
  for `run:` steps), `${{ env.ImageVersion }}` resolves to the empty
  string on macOS — zero protection on the platform with the actual bug.
  **Mitigation (mandatory pre-merge, Task 0b):** verify resolution via a
  live run or Actions runner docs BEFORE Task 1. If the variable is empty,
  add a macOS-specific shell step that reads `~/.bashrc` and exports
  `ImageVersion` to `$GITHUB_ENV` before the `setup-rust-toolchain` step.
  Deferring this discovery to "first PR run log inspection" is too late —
  an empty-segment outcome requires re-editing all 8 key sites.
- **One-time cache miss across all OS legs on first run after merge**
  (expected). The `-${{ env.ImageVersion }}` suffix changes every
  key. First PR push and first master push will see a full miss on
  each OS; `cargo build` rebuilds from scratch (a few minutes per
  job). Cache repopulates on the master push since
  `cache-save-if: ${{ github.ref == 'refs/heads/master' }}` is set
  on all jobs that participate in cache save. **Mitigation:** none
  needed — acceptable per spec scope item #3 and Technical Constraint
  bullet "expected and acceptable".
- **`actionlint` may flag the new `${{ env.ImageVersion }}` expression
  as an unknown env var** (low, behaviour-dependent). **Mitigation
  (mandatory pre-merge, Task 0a):** validate with a minimal YAML
  fragment before any bulk edits. If actionlint rejects the expression,
  switch to the `env:` alias strategy across all 8 key sites before
  starting Task 1 (avoid re-editing all sites after Task 7 flags them).
- **Polluted shim survives a manual `-v2` → `-v3` bump in flight**
  (low). If a future change adds `-v3` to invalidate the cache for
  unrelated reasons during the same image-version window, the
  combined key bust still works. No mitigation needed.
- **AC1 verification window** (process risk). Acceptance requires 3
  consecutive green macOS CI runs of `ci.yml` post-merge. If a
  flake of unrelated origin recurs in the window, the verdict is
  ambiguous. **Mitigation:** AC2's positive `cargo --version`
  diagnostic step distinguishes "macOS clippy died because cache
  pollution" (the step itself fails with the argparse error) from
  "macOS clippy died because flaky network during apt install" (a
  later step fails). The diagnostic step IS the signal; the 3-run
  window is corroboration.

## Test Design

CI workflow changes are intrinsically verified by CI itself; there
is no Rust unit-test surface. Verification plan:

- **Static check (pre-merge, local):** Task 7 runs `actionlint` on all
  three modified workflow files. Required gate per AGENTS.md AXIOM —
  same status as `cargo clippy --workspace -- -D warnings`.
- **Live check (PR open, all OS):** First push to the PR branch
  triggers `ci.yml` on `ubuntu-latest`, `macos-latest`, `windows-latest`.
  Each job's `setup-rust-toolchain` step logs the rust-cache key in the
  form `Using cache key: <key>`. Manually inspect the macOS leg's log
  to confirm the printed key ends with the image-version string
  (e.g. `Macos-stable-v2-20251104.1`). The diagnostic step's
  `cargo --version` line shows a real cargo version, not an
  argparse error.
- **Live check (post-merge, master, AC1 window):** Three consecutive
  `master` runs of `ci.yml` complete with `Clippy (macos-latest)`
  green. The combination of "no recurrence" + AC2 positive shape
  closes #340.
- **Negative control (not required for merge, optional future
  follow-up):** if a future runner-image bump occurs during the AC1
  window, the cache key changes automatically; the first job after
  the bump sees a cache miss and rebuilds. Confirms the mechanism
  end-to-end. No deliberate negative test is added — fabricating a
  rustup-init pollution to verify the diagnostic step would require
  injecting a faulty cache, which is outside the spec's "narrowest
  blast radius" principle.

## Open questions

(none — all design-affecting ambiguities resolved; see Decisions section above)
