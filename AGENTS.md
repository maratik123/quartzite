# Rust OSS Agent Rules

**CRITICALLY**
1) English for all output. Other language only on explicit user request.
2) Minimise thinking output shown to user.

## Project

> Read `ai-docs/context.md` for project purpose, entities, architecture, and design decisions — on demand.

## Permissions

Enforced by `settings.json`:

- **ALLOW:** Edit/Write under project root, `~/.claude`, `.claude`; `Bash(cargo *)`, `Bash(git *)`, `Bash(gh *)`
- **DENY (machine-blocked):** `.idea/**`; `.env` and `.env.*` (any depth); `secrets*` and `.secrets*` (any depth) — read/edit/write all blocked

Enforced server-side:

- **DENY:** `git push --force` to `master` — branch protection on `origin`

Honor-system (no machine check; rule still binding):

- **DENY:** `git push --force` to feature branches — prefer `--force-with-lease` and only after explicit user approval
- **DENY:** files outside project root

**ASK:** any tool not listed above; if access denied — suggest an alternative.

On session start: read `.gitignore`, treat matched paths as a read blacklist.

## Build & Test

```bash
cargo build
cargo test                            # all tests
cargo test test_name                  # filter by substring
cargo test -- --nocapture             # show stdout
cargo clippy -- -D warnings           # lint (strict)
cargo fmt                             # fix formatting
cargo fmt -- --check                  # check only
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace   # doc gate (matches CI)
cargo build -p quartzite --no-default-features   # verify derive-free / no_std path compiles
```

Search: `rg <pattern> --type rust [-l | -C 3]`

## API Stability

The project has **not yet been published to crates.io** and has no downstream clients. Public API may be freely renamed, removed, or restructured without backward-compat shims or deprecation layers. Do not add old-name aliases or `#[deprecated]` wrappers "for compatibility" — just make the clean change. Revisit this rule before the first `cargo publish`.

## API Naming

Follow `std` ecosystem conventions. The unsuffixed name is the **safe, ergonomic default**; suffixes mark deviations. Path of least resistance must be the safe path.

- **`_unchecked` is reserved for `unsafe` fns only.** Every `_unchecked` fn must be marked `unsafe` and document a `# Safety` section listing the invariants the caller must uphold to avoid Undefined Behavior. Examples: `slice::get_unchecked`, `str::from_utf8_unchecked`. **Never use `_unchecked` on a safe fn** — the suffix carries an unsafety implication; co-opting it for "skips an unrelated runtime check" misleads readers and reviewers.
- **Default safe + checked, returns `Result`/`Option` on failure:** safe `do_something()` plus `unsafe do_something_unchecked()`.
- **Default panics on failure:** panicking `do_something()` plus non-panicking `try_do_something()` (preferred) or `do_something_checked()` returning `Result`/`Option`. Optional `unsafe do_something_unchecked()` for UB-on-failure (e.g. `unchecked_add`).
- **Other "with-vs-without runtime behavior X" variants** (neither `unsafe` nor panicking — e.g. flag-aware vs. flag-bypassing): pick descriptive names that say what each variant *does*. Do **not** repurpose `_unchecked`/`_checked`. If one variant is overwhelmingly more common, give it the unsuffixed name and suffix the rare one.

## Code Style

- **All source files must be Rust (`.rs`).** No other languages in `src/`.
- Max line: 100 (rustfmt default).
- Strict clippy (enforced).
- Prefer Rust idioms over literal ports from other languages. When in doubt, ask.
- Let chains (`if let A = x && let B = y { ... }`) are valid in this codebase (edition 2024). Do not avoid them. Always format via `cargo fmt`, never `rustfmt <file>` directly.
- **Documentation:** Every crate must have `#![deny(missing_docs)]` in its `lib.rs`. Every public item must have at least a one-line `///` doc comment. Every new public item with only a single-line doc must include a `# Examples` block. Proc-macro examples use `no_run`; runtime items needing an event loop use `no_run`; pure library types use compiling doctests.
- **`#[inline]`:** Add `#[inline]` to every simple, non-generic function: no branches or loops, at most one function call, no binary bloat. Typical targets: field getters (`self.field`), trivial wrappers (`.as_deref()`, single delegation call), `Default::default()` that calls `Self::new()`, `const fn` struct-literal constructors. **Exclude** generic functions and blanket-impl trait methods — the compiler already has their bodies via monomorphization. Apply the same rule in proc-macro codegen: emit `#[inline]` before each simple generated `fn`.

## Dependency Versions

When adding or editing dependencies in `Cargo.toml`:

- Use `0.x` for `0.x.y` versions — never pin the patch.
- Use `x` for `x.y.z` versions — never pin minor or patch.
- No `~` prefix — Cargo's default `^` semantics are sufficient.
- After changing version constraints, run `cargo update` to pull latest compatible versions, then `cargo build` to verify.

## Workflow

- Merge PRs with a merge commit (`gh pr merge --merge`). Never squash or rebase-merge.
- **Never commit to local `master` when work is intended for a PR.** Create a feature branch (`git checkout -b feat/...`) *before* making any commits. Before any `git push`, confirm `git branch --show-current` is not `master` — if it is, stop and apply the recovery procedure below.
  - Recovery (commits on local master, not yet pushed): stash any uncommitted changes first (`git stash`), then `git checkout -b feat/...` → `git checkout master && git reset --soft origin/master && git restore --staged .` → push feature branch and open PR from it. Pop the stash on the feature branch if needed.
- Run `cargo build` before committing so `Cargo.lock` is refreshed and included in the commit when it changes.
- Stage files explicitly by name. **Never** use `git add -A` or `git add .` — they pull in unintended files (IDE state, accidental scratch files). The diff stat in PR 1–3 was cleanly scoped because every file was named.
- **Never** use `git commit --no-verify` (or any other hook-skipping flag). If a hook fails, fix the underlying issue.
- Plan first. Tests before prod code (TDD). Lint changed files.
- Any file with substantial logic (~50+ lines of non-trivial code) must have a `#[cfg(test)] mod tests` block. No exceptions for generator, codegen, or utility files. **Exception:** files under `examples/` are runnable demos, not library code — no `#[cfg(test)]` block required.
- `.gitignore` (not `.arcignore`).

## Communication

Interpret user phrasing literally and conservatively. When uncertain — ask, don't guess.

- **"Submit / push to PR"** = `git push` the branch to remote so commits appear in the open PR. **NOT** `gh pr merge`. Only merge when the user explicitly says "merge" or "merge the PR".
- **"wtf?" / "what?" / "huh?"** (or similar surprise/frustration) = the previous action was the opposite of what the user wanted. **Stop immediately**, do not retry, ask what was wrong before doing anything else.
- **IDE files** (`.idea/`, `*.iml`, `.vscode/`, `*.swp`, etc.) — never add, remove, modify, stage, or `.gitignore` them unless the user explicitly asks. They are the user's domain. "add ide files" most likely means **commit and track them**, not gitignore them — confirm before acting.

## Agent Docs

| Path | Purpose |
|------|---------|
| `ai-docs/context.md` | Project context — read on demand |
| `ai-docs/plans/INDEX.md` | Plan index — statuses and dependency order |
| `ai-docs/plans/*.spec.md` | Active task spec + acceptance criteria |
| `ai-docs/plans/*.design.md` | Active task design documents |
| `ai-docs/plans/*.progress.md` | Active task progress / handoff state |
| `ai-docs/plans/done/` | Completed plans (spec + design, implemented) |
| `ai-docs/plans/deferred/` | Blocked or future plans |
| `ai-docs/bugfix/trace-*.md` | Bugfix traces — deleted on resolution |
| `ai-docs/learnings.md` | Corrections log — feed for `/improve` |

## Corrections Log

On non-obvious correction or confirmed approach, write to `ai-docs/learnings.md`:

```
### YYYY-MM-DD — [category] — [short description]
**What happened:** [quote or paraphrase]
**Rule:** [what to do instead, or what to keep doing]
**Escalated?** no | AGENTS.md | skill:[name] | hook | settings | agent:[name] | memory (comma-separate multiple)

> `memory` = saved to global memory only. `/improve` treats it as unescalated — the entry remains a candidate for project-level escalation (AGENTS.md / skill / agent / hook / settings). `settings.local` does NOT count as project-level.
```

Categories: `code-style` | `process` | `architecture` | `testing` | `documentation` | `tooling` | `search` | `other`

Run `/improve` when ≥3 unescalated entries accumulate.

## Rust Test Conventions

- Unit tests in same file under `#[cfg(test)]` module.
- Integration tests in `tests/` directory.
- Use `rstest` for parameterized tests when useful.
- `mockall` for mocking traits.
- Assert with `assert_eq!` / `assert_matches!`; `pretty_assertions` crate encouraged for diffs.
- Test names as `snake_case` describing behavior: `returns_empty_when_not_found`.
- No `unwrap()` in production code without justifying comment; `expect("reason")` preferred.
- No `#[allow(clippy::...)]` / `#[allow(dead_code)]` unless unavoidable.
- Test behavior, transitions, errors, edge cases.
