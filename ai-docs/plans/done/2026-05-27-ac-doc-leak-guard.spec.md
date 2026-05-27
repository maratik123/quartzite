# Add automated guard for AC# rustdoc leaks

**Source:** issue #559
**Date:** 2026-05-27
**Tracked in:** #559

## Scope

1. Add a shell-based CI gate script under `scripts/` (e.g. `scripts/check-ac-doc-leaks.sh`) that scans the workspace's Rust source tree and exits non-zero whenever a published-rustdoc doc-comment (`///`, `//!`, or `#[doc = "..."]`) contains a token of the form `AC<digit>+` (e.g. `AC1`, `AC10b`).
2. Wire the script as a new CI step in the existing `docs` job of `.github/workflows/ci.yml` (the job already installs `ripgrep` and runs the sister `check-rustdoc-internal-refs.sh` and `check-rustflags-uniformity.sh` guards immediately after `cargo doc`), so it runs on both `push: branches: [master]` and `pull_request: branches: [master]` triggers without further workflow plumbing.
3. The script MUST treat plain `//` line comments as out-of-scope (they are the trusted test-traceability anchor — many in-tree examples in `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-macros/src/**`, `quartzite-widgets/src/widget_ext.rs`, `quartzite-renderer/src/wrapped_handler.rs`, etc.).
4. Verify zero retained matches at script-authoring time so the gate begins life green (PR #557 manually scrubbed all known leaks; baseline today is clean — verified `rg -n '^\s*///.*\bAC[0-9]+\b' --type rust -g '**/src/**'` returns no hits, ditto `//!`).
5. Update `.github/workflows/ci.yml` and run `actionlint .github/workflows/ci.yml` before commit (AGENTS.md AXIOM).

## Out of scope

- Approaches 2 (rustdoc-output regex test) and 3 (LLM-based semantic guard) from the issue body. Option 1 is the recommended starting point per the issue.
- Adding a new top-level workflow file. The existing `docs` job in `ci.yml` already runs after `cargo doc`, already has `ripgrep` installed, and already hosts the sister `check-rustdoc-internal-refs.sh` gate — a new workflow would duplicate the toolchain-setup overhead with no benefit.
- Touching the separate `.github/workflows/docs.yml` (Pages-deploy workflow); the guard is a PR-blocking gate, and `docs.yml` only runs on `push: master`, which fails the AGENTS.md "PR safety net" rule.
- Retro-rewriting any existing `///` / `//!` doc-comment in the tree (PR #557 already did that work; this task only adds the regression gate).
- Removing or modifying any `// AC<N>: …` line comment in tests.

## Deferred

- LLM-based semantic guard for arbitrary internal requirement IDs (`KD<N>`, `Q<N>`, etc.) | broader scope; defer until a second leak class actually appears | separate issue (none yet — open one if a non-`AC` internal token starts leaking).
- Pre-commit hook variant of the same check | the CI gate is sufficient and easier to maintain centrally | no separate issue.

## Key decisions

| Question | Decision |
|---|---|
| Which of the three approaches in the issue body? | Approach 1 (shell script + CI step). The issue body labels it the recommended starting point: low effort, deterministic, integrates with the existing `actionlint`-checked workflow infrastructure. |
| Which CI workflow file hosts the new step? | `.github/workflows/ci.yml` `docs` job, immediately adjacent to the existing `check-rustdoc-internal-refs.sh` / `check-rustflags-uniformity.sh` steps. Inherits the `ripgrep` install, the toolchain cache, and PR-trigger coverage. |
| Token pattern? | `\bAC[0-9]+[a-z]*\b` — covers `AC1`, `AC10`, `AC6a`, `AC10b` shapes seen in-tree (`quartzite-style-dispatch/src/dispatch.rs:529` has `AC6b`). |
| Doc-comment forms covered? | `///`, `//!`, and `#[doc = "..."]` — mirrors the precedent script `scripts/check-rustdoc-internal-refs.sh`. |
| `//` line comments? | Out of scope — test-traceability anchor; explicit AC in the issue body. |
| File-tree scope? | Workspace Rust source tree under `**/src/**`, excluding `tests/`, `benches/`, `target/`, and `quartzite-test-helpers/src/` (mirrors precedent script). |
| `#[cfg(test)]`-enclosed doc-comments? | Filter them out using the same heuristic as `scripts/check-rustdoc-internal-refs.sh` (inline `#[cfg(test)] mod NAME { ... }` brace-depth tracking + sibling-file `#[cfg(test)] #[path = "NAME.rs"] mod IDENT;`). Test-region doc-comments do not ship in published rustdoc, so a leak there is not a real leak. |
| Reuse precedent script as scaffolding? | Yes — `scripts/check-rustdoc-internal-refs.sh` is the closest in-tree precedent and was authored for an analogous "no internal leaks in published rustdoc" rule (issue #336, PR `feat/2026-05-21-rustdoc-strip-internal-refs`). Design Subagent decides whether the new check shares helpers, lives as a separate script, or extends the existing one. |
| Exit codes? | `0` = no retained hits; `1` = at least one retained hit (gate fails); `2` = internal usage error (mirrors precedent script). |

## Technical constraints

- `actionlint .github/workflows/ci.yml` MUST pass before `git add` on the workflow change (AGENTS.md AXIOM, listed alongside `cargo build` and `cargo clippy --workspace --all-targets -- -D warnings`).
- The CI `docs` job already installs `ripgrep` via `apt-get install -y libfontconfig1-dev ripgrep`. The new script may rely on `rg` being on PATH; no extra install step is needed.
- The script MUST be executable from any cwd (resolve `REPO_ROOT` via `BASH_SOURCE` like the precedent).
- The script MUST `set -euo pipefail`.
- The script MUST exit non-zero on a missing `rg` binary with exit code `2` and a clear error message (precedent style).
- The script MUST be runnable locally as `bash scripts/check-ac-doc-leaks.sh` so a developer can self-verify before pushing.
- Adding the new script is a `scripts/*.sh` addition (workspace-wide gate); the `changes` job in `ci.yml` already lists `'.github/workflows/**'` under the `rust` filter, so any workflow edit triggers the docs job. The script itself living under `scripts/` is fine — the docs job runs whenever the rust filter is hit; pure-script edits without a workflow edit will NOT trigger the docs job, but a missing trigger only matters when someone edits the script in isolation, which is a low-risk scenario for a regression gate that is already green.

## Acceptance Criteria

| #  | Criterion |
|----|-----------|
| AC1 | A new file `scripts/check-ac-doc-leaks.sh` exists and is executable (`chmod +x`), with a `#!/usr/bin/env bash` shebang and `set -euo pipefail`. |
| AC2 | Running `bash scripts/check-ac-doc-leaks.sh` against the current `master` tree exits 0 (no retained hits). |
| AC3 | Adding a `///` doc-comment containing `AC<digit>` to any file under a workspace crate's `src/` (excluding `#[cfg(test)]`-enclosed regions and `quartzite-test-helpers/src/`) causes the script to exit 1 and print the offending file:line. Verified via a manual test injection that is reverted before commit. |
| AC4 | Same as AC3 but for `//!` inner doc-comments. |
| AC5 | Same as AC3 but for `#[doc = "..."]` attribute doc-comments. |
| AC6 | Adding a `// AC<digit>` (plain line comment, NOT a doc-comment) anywhere does NOT cause the script to fail. The existing `// ── AC1: …` / `// AC3: …` markers in `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-macros/src/object_impl/mod.rs`, `quartzite-widgets/src/widget_ext.rs`, `quartzite-renderer/src/wrapped_handler.rs`, and `quartzite-macros/src/extend/{parse,codegen}.rs` remain non-failing. |
| AC7 | A `///` doc-comment containing `AC<digit>` placed inside a `#[cfg(test)] mod tests { ... }` block does NOT cause the script to fail (test-region filter, matching precedent). |
| AC8 | `.github/workflows/ci.yml` `docs` job has a new step named `Check rustdoc has no AC# leaks` (or equivalent) that runs `bash scripts/check-ac-doc-leaks.sh`, placed adjacent to the existing `check-rustdoc-internal-refs.sh` / `check-rustflags-uniformity.sh` steps. |
| AC9 | `actionlint .github/workflows/ci.yml` exits 0 after the workflow edit. |
| AC10 | The CI `Docs` job (which encompasses the new step) runs on both `push: branches: [master]` and `pull_request: branches: [master]` triggers — no new top-level workflow needed because `ci.yml` already declares both. |

## Open questions

- Whether the new script should be a stand-alone script or extend `scripts/check-rustdoc-internal-refs.sh` with an additional pattern. **Defensible default:** stand-alone — keeps each gate's failure message self-contained and lets the precedent script evolve independently. Design Subagent revisits during decomposition if the extracted-helper option proves cleaner.
- Whether to enforce the token pattern only on `[A-Z]C<digit>+` (just `AC`) or a broader internal-token alphabet (`KD`, `Q`, …). **Defensible default:** `AC` only — the deferred LLM guard above tracks the broader case; expanding now would conflate two distinct leak classes.
