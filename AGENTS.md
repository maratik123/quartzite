# Rust OSS Agent Rules

**CRITICALLY**
1) English for all output. Other language only on explicit user request.
2) Minimise thinking output shown to user.

## Project

> Read `ai-docs/context.md` for project purpose, entities, architecture, and design decisions — on demand.

## Permissions

Machine-enforced rules live in `.claude/settings.json` (allow/deny entries) and on `origin` (branch protection). Read those files for the authoritative list — duplicating them here lets the two sources drift.

Honor-system rules (no machine check; still binding):

- **DENY:** `git push --force` to feature branches — prefer `--force-with-lease`, and only after explicit user approval. Force-pushing to `master` is server-blocked regardless.
- **DENY:** files outside project root.
- **ASK:** any tool not allow-listed in `settings.json`; if denied — suggest an alternative.

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
- **Prefer non-panicking APIs for libraries:** implement `try_do_something()` returning `Result`/`Option` as the default; leave it to the caller to decide how to handle failure. A panicking `do_something()` convenience wrapper may be added alongside but must not be the only option. Panicking is acceptable only when a fundamental invariant is broken and continuing would leave the application in an inconsistent state — even then, get explicit user approval before adding a panicking API. Document panicking behaviour in the `# Panics` section of the fn doc. Optional `unsafe do_something_unchecked()` for UB-on-failure (e.g. `unchecked_add`).
- **Other "with-vs-without runtime behavior X" variants** (neither `unsafe` nor panicking — e.g. flag-aware vs. flag-bypassing): pick descriptive names that say what each variant *does*. Do **not** repurpose `_unchecked`/`_checked`. If one variant is overwhelmingly more common, give it the unsuffixed name and suffix the rare one.

## Code Style

- **Source files:** Rust-only (`.rs`) under `src/`; max 100 cols (rustfmt default); format via `cargo fmt`, never `rustfmt <file>` directly. See [`ai-docs/code-style.md` → Source files](ai-docs/code-style.md#source-files).
- **Linter posture:** strict clippy enforced (`-D warnings`); no blanket `#[allow]` without justification. See [`ai-docs/code-style.md` → Linter posture](ai-docs/code-style.md#linter-posture).
- **Rust idioms:** prefer Rust over literal ports; let chains valid (edition 2024); comparison helpers (`.min`/`.max`/`.clamp`/`Option::or`/`Option::filter`) over explicit `if`/`match`; **never cite GUI/UI frameworks** (Qt, GTK, WinForms, SwiftUI, …) as design justification. See [`ai-docs/code-style.md` → Rust idioms](ai-docs/code-style.md#rust-idioms).
- **Library safety idioms:** mutex `lock().ok()?` / `.unwrap_or_else(|e| e.into_inner())`; safe primitives (`OnceLock` / `Arc` / `Weak` / `AtomicBool`) over raw pointers + `unsafe`. See [`ai-docs/code-style.md` → Library safety idioms](ai-docs/code-style.md#library-safety-idioms).
- **Documentation:** every crate has `#![deny(missing_docs)]` + `#![warn(clippy::undocumented_unsafe_blocks)]`; every public item has at least one-line `///`; `# Examples` block on new public items with single-line docs; doc-style conventions live in [`ai-docs/doc-convention.md`](ai-docs/doc-convention.md). See [`ai-docs/code-style.md` → Documentation](ai-docs/code-style.md#documentation).
- **Error types:** `thiserror` for new error enum/struct; hand-rolled `Display`/`Error` reserved for cases the derive cannot express. See [`ai-docs/code-style.md` → Error types](ai-docs/code-style.md#error-types).
- **Tracing:** `*_span!` guard wrapping the body of any function that meaningfully mutates application state; `debug_span!` for lifecycle, `trace_span!` for supplementary; high-frequency paths gated behind `verbose-tracing` cargo feature. See [`ai-docs/code-style.md` → Tracing](ai-docs/code-style.md#tracing).
- **`#[inline]` and the `_Simple._` doc tag:** mark every recursively-simple fn (no branches/loops, ≤ 1 non-simple call) with the marker matching its shape — `#[inline]` (concrete), `/// _Simple._` (generic free fn / inherent generic method / trait method declaration whose every conforming impl is required to be simple), `// _Simple._` (method inside `impl<T> Trait for Foo<T>` — avoids overriding trait-inherited rustdoc); strip + cascade re-test of callers when an edit makes a previously-simple fn non-simple. See [`ai-docs/code-style.md` → `#[inline]` and the `_Simple._` doc tag](ai-docs/code-style.md#inline-and-the-_simple_-doc-tag).
- **Generic-fn split for binary size:** public fn with conversion-style generic param (`impl Into<T>` / `impl AsRef<T>` / `impl ToString`) and > 3 line body extracts the body into a nested `fn inner(...)` (NOT a sibling `<outer>_inner` impl method); outer carries `_Simple._`; **if the inner ends up simple, unwrap it** — `#[inline]` simple inner is dead weight. See [`ai-docs/code-style.md` → Generic-fn split for binary size](ai-docs/code-style.md#generic-fn-split-for-binary-size).
- **File size:** target 200–400 lines per `.rs` file excluding `#[cfg(test)]`; soft 500/800; hard 1000/1500 (refactor before merge unless exempt — auto-generated, single state machine / `match`, `macro_rules!`); per-fn `clippy::too_many_lines` (>100); counter-rule against over-splitting (one-struct-per-file is not Rust idiom). See [`ai-docs/code-style.md` → File size](ai-docs/code-style.md#file-size).

See [`ai-docs/code-style.md`](ai-docs/code-style.md) for the canonical reference.

## Dependency Versions

When adding or editing dependencies in `Cargo.toml`:

- Use `0.x` for `0.x.y` versions — never pin the patch.
- Use `x` for `x.y.z` versions — never pin minor or patch.
- No `~` prefix — Cargo's default `^` semantics are sufficient.
- After changing version constraints, run `cargo update` to pull latest compatible versions, then `cargo build` to verify.

## Workflow

- Merge PRs with a merge commit (`gh pr merge --merge`). Never squash or rebase-merge.
- **Never edit on local `master` when work is intended for a PR.** Create a feature branch (`git checkout -b feat/...`) *before* making any file edit — not just before commit. Accumulating uncommitted edits on `master` is the failure mode this rule guards: it leaves the working tree dirty on the wrong branch and forces a reactive switch later. The first action of any skill/workflow that produces commits (`/task`, `/improve`, `/ai-audit`, etc.) is `git branch --show-current`; if `master`, switch before any `Edit`/`Write`. Before any `git push`, confirm `git branch --show-current` is not `master` — if it is, stop and apply the recovery procedure below.
  - Recovery (commits on local master, not yet pushed): stash any uncommitted changes first (`git stash`), then `git checkout -b feat/...` → `git checkout master && git reset --soft origin/master && git restore --staged .` → push feature branch and open PR from it. Pop the stash on the feature branch if needed.
- Run `cargo build` before committing so `Cargo.lock` is refreshed and included in the commit when it changes.
- Stage files explicitly by name. **Never** use `git add -A` or `git add .` — they pull in unintended files (IDE state, accidental scratch files).
- **Before every `git commit` during a PR task**, check `git status` for `ai-docs/learnings.md`. If it appears modified or untracked, stage it together with the related code changes — learnings are part of the task deliverable and must be visible in the PR diff.
- **Never** use `git commit --no-verify` (or any other hook-skipping flag). If a hook fails, fix the underlying issue.
- **Never** use `git reset --hard` — it silently discards uncommitted work (working-tree changes, untracked files). Use one of these instead:
  - `git reset --soft HEAD~N` — preserves working tree; commits become staged
  - `git stash` — saves uncommitted changes before switching branches
  - `git cherry-pick` — moves specific commits to another branch
  - Backup branch — `git checkout -b backup/...` before any destructive operation
- Plan first. Tests before prod code (TDD). Lint changed files.
- Any file with substantial logic (~50+ lines of non-trivial code) must have a `#[cfg(test)] mod tests` block. No exceptions for generator, codegen, or utility files. **Exception:** files under `examples/` are runnable demos, not library code — no `#[cfg(test)]` block required.
- `.gitignore` (not `.arcignore`).
- After generating or moving any markdown file with relative links to siblings (`../`, `../../`), trace at least one link by hand or with `realpath` before committing. From `ai-docs/deferred/file.md`: `..` reaches `ai-docs/`, `../..` reaches the repo root.
- **PR review comment resolution:** After pushing fixes, resolve only the comments that were addressed by a code change. Comments where you posted an objection (explaining why no change was made) must **not** be resolved — leave them for the reviewer to accept or push back on. **Mechanics (GitHub stores review threads, not just comments — REST `/pulls/{N}/comments` does not expose `isResolved`; use GraphQL):**
  1. Reply to each comment via `gh api repos/<OWNER>/<REPO>/pulls/<N>/comments/<comment-id>/replies -f body='...'`.
  2. Query unresolved thread IDs:
     ```bash
     gh api graphql -f query='{ repository(owner:"<OWNER>", name:"<REPO>") { pullRequest(number:<N>) { reviewThreads(first:50) { nodes { id isResolved path comments(first:1) { nodes { databaseId body } } } } } } }'
     ```
     Filter to `isResolved == false` and match each thread to the comment it was opened on (via `comments.nodes[0].databaseId` or `path`). Never guess thread IDs — `NOT_FOUND` means the ID is wrong, not that resolution is unavailable.
  3. Resolve each fixed thread:
     ```bash
     gh api graphql -f query='mutation { resolveReviewThread(input:{threadId:"<id>"}) { thread { isResolved } } }'
     ```
     Verify `isResolved: true` in the response.
  4. Skip threads where you posted an objection — those stay open for the reviewer.
- **PR body sync after every push.** When the current branch has an open PR, every `git push` is followed by `gh pr view <N> --json title,body` *unconditionally* — read the body first, then decide whether to edit. Sync via `gh pr edit` if the body now contradicts the diff (renames, scope drift, AC checkbox flips, cited counts that drifted, dropped/added items). Routine commits within already-described scope do not need an edit, but the **read** is non-negotiable. Reasoning your way out of the read is the failure mode this rule prevents. **Exception:** the read is not required when `gh pr create` immediately followed the push (the body is what you just authored — nothing to discover). The rule starts firing on the *next* push to the branch. The **upstream tracking issue's** title and body are the user's original problem statement — do not rewrite them; communicate scope changes via `gh issue comment` instead.

## Propagation Rule

When editing any instruction file (`AGENTS.md`, `.claude/skills/**`, `.claude/agents/**`, `.claude/settings.json`), propagate the change to every related file in the same operation — before reporting done.

**Sync groups (canonical):**
- **Review group:** `.claude/skills/code-review/SKILL.md` (workflow) ↔ `.claude/agents/review-findings.md` (findings producer) ↔ `.claude/agents/self-review.md` (fix validator)

> The former `task` ↔ `task-issue` group collapsed when `task-issue` was merged into `task` — both entry modes now live in `.claude/skills/task/SKILL.md`. Grep across `.claude/skills/` and `.claude/agents/` per the procedure below to catch any lingering references.

**Procedure:**
1. Before closing the edit, `grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ AGENTS.md` for any file that references the same rule, exemption, or terminology.
2. Apply the same change (or the corresponding enforcement adjustment) in every match.
3. AGENTS.md rule exemptions especially must propagate to agent checklists that enforce the rule (`self-review.md`, `review-findings.md`).

Do not refer to a skill as an "agent" or vice versa — the distinction matters for spawning. (`code-review` is a skill; `review-findings` and `self-review` are agents spawned by it.)

## Communication

Interpret user phrasing literally and conservatively. When uncertain — ask, don't guess.

- **"Submit / push to PR"** = `git push` the branch to remote so commits appear in the open PR. **NOT** `gh pr merge`. Only merge when the user explicitly says "merge" or "merge the PR".
- **"wtf?" / "what?" / "huh?"** (or similar surprise/frustration) = the previous action was the opposite of what the user wanted. **Stop immediately**, do not retry, ask what was wrong before doing anything else.
- **IDE files** (`.idea/`, `*.iml`, `.vscode/`, `*.swp`, etc.) — never add, remove, modify, stage, or `.gitignore` them unless the user explicitly asks. They are the user's domain. "add ide files" most likely means **commit and track them**, not gitignore them — confirm before acting.

## Agent Docs

| Path | Purpose |
|------|---------|
| `ai-docs/context.md` | Project context — read on demand |
| `ai-docs/code-style.md` | Workspace code-style reference — read on demand |
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
**Escalated?** no | AGENTS.md | skill:[name] | hook | settings | agent:[name] | doc-convention | code-style | memory (comma-separate multiple)

> `memory` = saved to global memory only. `/improve` treats it as unescalated — the entry remains a candidate for project-level escalation (AGENTS.md / skill / agent / hook / settings / doc-convention / code-style). `settings.local` does NOT count as project-level.
> `doc-convention` = the rule landed in `ai-docs/doc-convention.md`. Use only for documentation-style rules that genuinely belong in the workspace doc-convention reference rather than in AGENTS.md or a skill.
> `code-style` = the rule landed in `ai-docs/code-style.md`. Use only for code-style rules that genuinely belong in the workspace code-style reference rather than in AGENTS.md or a skill.
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
