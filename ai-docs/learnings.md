# Learnings

### 2026-05-02 — code-style — let chains are allowed and formattable in edition 2024

**What happened:** During the macros task, rustfmt errored on a let chain with "let chains are only allowed in Rust 2024 or later". The workspace uses `edition = "2024"`, Rust 1.95, and rustfmt 1.9.0 — all of which support let chains. The error was caused by running rustfmt without `--edition 2024` explicitly, or against a stale binary. The response was to replace let chains wholesale with match expressions as a blanket rule.

**Rule:** Let chains (`if let A = x && let B = y { ... }`) are valid in this codebase. Do not avoid them. Run `rustfmt` via `cargo fmt` (which picks up the workspace edition automatically) rather than invoking `rustfmt <file>` directly.

**Escalated?** AGENTS.md

### 2026-05-02 — process — always create a feature branch before committing; never commit directly to master

**What happened:** When the user said "submit PR", commits were already on local master. Instead of creating a feature branch first, `git push` was run directly against master — pushing the commits to origin/master. `master` is branch-protected (no force push), so the commits could not be removed after the fact and a proper PR became impossible.

**Rule:** When work is intended for a PR, create a feature branch (`git checkout -b feat/...`) *before* making any commits. Never commit to local master with the intention of later turning it into a PR.

Recovery — if commits were accidentally made on local master and not yet pushed:
1. `git checkout -b feat/...` — branch off from current HEAD (carries the commits)
2. `git checkout master && git reset --hard origin/master` — reset local master to remote state
3. Push the feature branch and open the PR from it
4. **Never push master** — not even as an intermediate step.

Before any `git push`: run `git branch --show-current` and confirm it is **not** `master`. If it is master — stop, do not push, apply the recovery procedure above.

If "submit PR" is requested and commits are already pushed to origin/master: stop and tell the user — there is no recovery without a force push, which branch protection may block.

**Escalated?** no

### 2026-05-02 — testing — any sufficiently large file requires unit tests

**What happened:** Three codegen files (`object/codegen.rs`, `object_impl/codegen.rs`, `meta_enum/codegen.rs`) were written without `#[cfg(test)]` modules. Gaps were caught in review and by the user. The original rule was codegen-specific, but the user generalised it: any file with substantial logic needs tests.

**Rule:** Any file with ~50+ lines of non-trivial code must have a `#[cfg(test)] mod tests` block. This applies equally to codegen, parse, util, and any other module — not just files named `codegen.rs`.

**Escalated?** AGENTS.md
