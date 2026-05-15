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
cargo clippy --workspace -- -D warnings   # lint (strict; --workspace catches leaf crates outside the default dep tree)
cargo fmt                             # fix formatting
cargo fmt -- --check                  # check only
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features   # doc gate (matches CI; --all-features so intra-doc links into every feature-gated module — `serde`-gated `snapshot`, `style`, `widgets`, … — resolve regardless of which feature gates them)
cargo build -p quartzite --no-default-features --features libm   # verify derive-free / no_std path compiles
actionlint .github/workflows/<file>.yml   # required gate for any new/modified workflow file
```

> **AXIOM — `actionlint` MUST pass before `git add` on any modified workflow file.**
> Required gate, same status as `cargo build` and `cargo clippy --workspace -- -D warnings`. Skipped twice despite the rule existing — escalated to AGENTS.md after the second occurrence.
>
> | If you see... | Action |
> |---|---|
> | `M .github/workflows/<name>.yml` in `git status` | Run `actionlint <file>` (or pass every changed workflow file in one invocation) **before** `git add` |
> | `actionlint` reports any error | Fix it. **NEVER** bypass. |
> | A NEW `.github/workflows/*.yml` triggered on `push: branches: [master]` **only** (no `pull_request:` sibling) | The PR safety net does NOT exercise this workflow. Either add a `pull_request: branches: [master]` trigger so PR CI runs it, OR add a `# Why master-only: <reason>` comment above the `on:` block AND verify an existing PR workflow exercises an equivalent code path. _See `ai-docs/learnings.md` 2026-05-13 master-only-trigger entry: 14 consecutive master pushes failed `Docs` before the gap was found._ |
> | A commit message claims "all X" / "every Y" while touching a finite set | Enumerate the set programmatically before commit (e.g. `diff <(grep -l 'runs-on:.*ubuntu' .github/workflows/*.yml) <(grep -l '<token>' .github/workflows/*.yml)`) and confirm the diff actually covers it. |
>
> What `actionlint` catches that `cargo` cannot: runner-version mismatches, deprecated action versions, expression-syntax errors, shell-quoting issues.

> **AXIOM — Every project instruction file Claude loads per invocation MUST stay below 40,000 chars.**
> Harness-enforced soft cap; crossing it imposes measurable per-invocation cost on every agent spawn, `/task`, `/triage`, and review pass. Project-side **35,000-char early warning** gives one full `/task` cycle of headroom before the harness warning starts firing. Applies to `AGENTS.md`, `CLAUDE.md`, every `.claude/skills/**/SKILL.md`, every `.claude/agents/**.md`, and `ai-docs/{code-style,doc-convention,context,agent-writing-style,corrections-log}.md`.
>
> | If `wc -c <file>` reports... | Action |
> |---|---|
> | ≥ 40,000 chars | **STOP**. Plan extraction / dedup before the next commit — same model PR #324 used for AGENTS.md (extract verbose subsections into `ai-docs/<topic>.md` reference pages with anchored links from the source file). |
> | 35,000–39,999 chars | Proactive extraction pass; do not let the next `/task` push it over 40k. |
> | < 35,000 chars | OK. |
>
> Quick scan: `wc -c AGENTS.md CLAUDE.md .claude/skills/**/SKILL.md .claude/agents/**.md ai-docs/{code-style,doc-convention,context,agent-writing-style,corrections-log}.md`. Until `scripts/check-instruction-file-sizes.sh` lands as a pre-commit / CI gate, any `/task` whose work touches an instruction file should run this command before commit.

Search: `rg <pattern> --type rust [-l | -C 3]`

## API Stability

> **AXIOM — Pre-publish: clean breaks. No compat shims.**
> The project has **not** been published to crates.io and has no downstream clients. Public API may be freely renamed, removed, or restructured without backward-compat shims, deprecation layers, or `#[deprecated]` wrappers.
>
> | If you're tempted to... | Do this instead |
> |---|---|
> | Add `pub use OldName as NewName;` "for compat" | **REMOVE** the alias — make the clean rename |
> | Wrap removed fn with `#[deprecated] pub fn old() -> _ { new() }` | **DELETE** the wrapper — call sites update directly |
> | Keep both old and new APIs side-by-side temporarily | Pick one — old is gone |

Revisit this rule before the first `cargo publish`.

## API Naming

> **AXIOM — `_unchecked` means `unsafe` AND UB-on-failure. Period.**
> The suffix is reserved exclusively for `unsafe fn` whose contract documents Undefined Behaviour on caller-invariant violation. **NEVER** apply it to a safe fn — even one that "skips a runtime check" — because the suffix carries unsafety implications that mislead readers and reviewers.
>
> | Your fn... | Suffix |
> |---|---|
> | Is `unsafe`, UB on caller violation (`# Safety` section required) | `_unchecked` ✓ (e.g., `slice::get_unchecked`, `str::from_utf8_unchecked`) |
> | Is **safe**, skips a non-safety check (validation, sort-order, etc.) | A descriptive suffix like `_unverified` / `_skip_validation` / `_unsorted` — **NEVER** `_unchecked` |
> | Is **safe**, returns `Result` / `Option` on failure | Unsuffixed (`do_something`); add a `try_*` variant if a panicking sibling exists |

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
- **`#[inline]` and the `_Simple._` doc tag:** mark every recursively-simple fn (no branches/loops, ≤ 1 non-simple call) with the marker matching its shape — `#[inline]` (concrete fn **or** method inside `impl Trait for ConcreteFoo` — concrete-impl trait method needs `#[inline]` for cross-crate inlining without LTO; `// _Simple._` is *not* a substitute), `/// _Simple._` (generic free fn / inherent generic method / trait method declaration whose every conforming impl is required to be simple), `// _Simple._` (method inside `impl<T> Trait for Foo<T>` — avoids overriding trait-inherited rustdoc); strip + cascade re-test of callers when an edit makes a previously-simple fn non-simple. See [`ai-docs/code-style.md` → `#[inline]` and the `_Simple._` doc tag](ai-docs/code-style.md#inline-and-the-_simple_-doc-tag).
- **Generic-fn split for binary size:** public fn with conversion-style generic param (`impl Into<T>` / `impl AsRef<T>` / `impl ToString`) and > 3 line body extracts the body into a nested `fn inner(...)` (NOT a sibling `<outer>_inner` impl method); outer carries `_Simple._`; **if the inner ends up simple, unwrap it** — `#[inline]` simple inner is dead weight. See [`ai-docs/code-style.md` → Generic-fn split for binary size](ai-docs/code-style.md#generic-fn-split-for-binary-size).
- **File size:** target 200–400 lines per `.rs` file excluding `#[cfg(test)]`; soft 500/800; hard 1000/1500 (refactor before merge unless exempt — auto-generated, single state machine / `match`, `macro_rules!`); per-fn `clippy::too_many_lines` (>100); counter-rule against over-splitting (one-struct-per-file is not Rust idiom). See [`ai-docs/code-style.md` → File size](ai-docs/code-style.md#file-size).

See [`ai-docs/code-style.md`](ai-docs/code-style.md) for the canonical reference.

## Dependency Versions

> **AXIOM — Query the live registry BEFORE writing any specific version string OR asserting external-action behaviour. Training data is stale.**
> Treating remembered versions or remembered Action behaviour as authoritative has shipped wrong majors and false load-bearing claims more than once in this repo. See [`ai-docs/dependency-versions.md`](ai-docs/dependency-versions.md) for the per-artifact lookup table (Cargo crate / GitHub Action / Action behaviour-verification recipe). Apply the pinning rule (below) to the **observed** version, never the remembered one.

When adding or editing dependencies in `Cargo.toml`:

- Use `0.x` for `0.x.y` versions — never pin the patch.
- Use `x` for `x.y.z` versions — never pin minor or patch.
- No `~` prefix — Cargo's default `^` semantics are sufficient.
- After changing version constraints, run `cargo update` to pull latest compatible versions, then `cargo build` to verify.

## Workflow

> **AXIOM 1 — NEVER edit on local `master` when work is intended for a PR.**
> Create a feature branch (`git checkout -b feat/...` or `chore/...`) **before** any file edit — not before commit, **before edit**. Accumulating uncommitted edits on `master` leaves the tree dirty on the wrong branch and forces a reactive switch later.
>
> | If `git branch --show-current` returns... | Action |
> |---|---|
> | `master` AND you're about to make a PR-targeted edit | **STOP**. Run `git checkout -b <prefix>/<descriptive-name>` first. Only then edit. |
> | A feature branch | Proceed with edits |
> | `master` AND you've already made commits (recovery) | `git stash` → `git checkout -b <feature>` → `git checkout master && git reset --soft origin/master && git restore --staged .` → push feature branch → open PR. Pop stash on feature branch if needed. |
>
> The first action of any skill/workflow that produces commits (`/task`, `/improve`, `/ai-audit`, etc.) is `git branch --show-current`; if `master`, switch **before** any `Edit`/`Write`. Before any `git push`, confirm again — if it is `master`, stop and apply recovery.

- Merge PRs with a merge commit (`gh pr merge --merge`). Never squash or rebase-merge.
- Run `cargo build` before committing so `Cargo.lock` is refreshed and included in the commit when it changes.
- Stage files explicitly by name. **Never** use `git add -A` or `git add .` — they pull in unintended files (IDE state, accidental scratch files).
- **Before every `git commit` during a PR task**, check `git status` for `ai-docs/learnings.md`. If it appears modified or untracked, stage it together with the related code changes — learnings are part of the task deliverable and must be visible in the PR diff. **After every push** (CI fix, reviewer-comment fix, self-review fix): if a learning entry was written *after* the last code commit landed, give it its own commit on the feature branch in the same turn — do not leave `learnings.md` as an unstaged working-tree change waiting to be bundled with the next code change. Order: write learning → `git add ai-docs/learnings.md` → commit → push.
- **Never** use `git commit --no-verify` (or any other hook-skipping flag). If a hook fails, fix the underlying issue.
- **CI-fix commits get self-review too.** Any code change made in response to a CI failure — even a one-liner in test code — is subject to the same self-review requirement as the original implementation commits. Before pushing a CI-fix commit, spawn the `self-review` agent (or, at minimum, run an inline review covering: correct idiom used, no semantics broken, no adjacent lint issues missed, commit message accurate). The `/task` Step 10 self-review loop applies to every code-producing commit on the branch, not just the initial implementation batch.
- **Never** use `git reset --hard` — it silently discards uncommitted work (working-tree changes, untracked files). Use one of these instead:
  - `git reset --soft HEAD~N` — preserves working tree; commits become staged
  - `git stash` — saves uncommitted changes before switching branches
  - `git cherry-pick` — moves specific commits to another branch
  - Backup branch — `git checkout -b backup/...` before any destructive operation
- Plan first. Tests before prod code (TDD). Lint changed files.
- Any file with substantial logic (~50+ lines of non-trivial code) must have a `#[cfg(test)] mod tests` block. No exceptions for generator, codegen, or utility files. **Exceptions:**
  - Files under `examples/` are runnable demos, not library code — no `#[cfg(test)]` block required.
  - Files under `benches/` declared with `[[bench]] harness = false` (criterion bench binaries) — `criterion_main!` replaces the test runner, so `#[cfg(test)]` items would never be invoked. No `#[cfg(test)]` block required.
- `.gitignore` (not `.arcignore`).
- After generating or moving any markdown file with relative links to siblings (`../`, `../../`), trace at least one link by hand or with `realpath` before committing. From `ai-docs/deferred/file.md`: `..` reaches `ai-docs/`, `../..` reaches the repo root.
- **PR review comment resolution:** After pushing fixes, resolve only the comments addressed by a code change; comments where you posted an objection stay open for the reviewer. GitHub stores review threads (not just comments), and REST `/pulls/{N}/comments` does not expose `isResolved` — use the GraphQL `reviewThreads` query then `resolveReviewThread` mutation. See [`ai-docs/workflow.md` → PR review comment resolution](ai-docs/workflow.md#pr-review-comment-resolution) for the full recipe.

> **AXIOM 2 — Read the PR body via `gh pr view <N>` after EVERY `git push` to a feature branch with an open PR. Unconditional.**
> The READ is mandatory even when the push was a routine typo / format / nit. The EDIT is conditional — only when the body contradicts the new commits.
>
> | After... | Required action |
> |---|---|
> | `git push` to a feature branch with an open PR | Run `gh pr view <N> --json title,body` immediately. Read the body. |
> | The body still describes the diff accurately | No `gh pr edit` needed — read complete |
> | The body contradicts the new commits (renames, scope drift, AC flips, cited counts that drifted) | Run `gh pr edit` to sync |
> | `gh pr create` immediately preceded the push (i.e., this is the first push that opened the PR) | **Skip** the read — the body is what you just authored. The rule fires on the **next** push. |
>
> The upstream tracking **issue**'s title and body are the user's original problem statement — do not rewrite them. Communicate scope changes via `gh issue comment` instead.

> **AXIOM — `ai-docs/deferred/_inbox.md` is written ONLY by `/task` Step 12 and `/triage`.**
> Hand-edits to `_inbox.md` defeat the propagation contract that Issue A2 sets up — they hide rows from the parser and conflict with future Step-12 appends.
>
> | If you see... | Action |
> |---|---|
> | A row in `_inbox.md` you want to move to a thematic file | Run `/triage`; let it sort the row |
> | A row in `_inbox.md` you want to drop | Run `/triage`; mark "drop" during the drain step |
> | A row missing from `_inbox.md` for a freshly-merged spec | Re-run `/task` Step 12 manually (or wait for the next merged spec to trigger it) |
> | An entry whose source-spec section shape was unrecognised by the parser | Step 12 emits a warning; resolve by reformatting the source spec OR by adding the shape to the parser's allow-list (Issue A2 design phase) |

## Propagation Rule

> **AXIOM — Edits to one instruction file MUST propagate to its sync-group siblings in the SAME PR.**
> The Propagation Rule fires whenever you edit an instruction file. Sister files in the same sync group must receive the corresponding change before the PR is opened.
>
> | If you edit... | You MUST also check / update... |
> |---|---|
> | `.claude/skills/code-review/SKILL.md` | `.claude/agents/review-findings.md` AND `.claude/agents/self-review.md` (Review group) |
> | `.claude/agents/review-findings.md` | `.claude/skills/code-review/SKILL.md` AND `.claude/agents/self-review.md` (Review group) |
> | `.claude/agents/self-review.md` | `.claude/skills/code-review/SKILL.md` AND `.claude/agents/review-findings.md` (Review group) |
> | `.claude/skills/interview/SKILL.md` | `.claude/agents/spec-writer.md` (Interview group — Rule-5 substring blacklist mirrors live in `spec-writer.md`) |
> | `.claude/agents/spec-writer.md` | `.claude/skills/interview/SKILL.md` (Interview group — orchestrator-side validation may need to update if the contract shifts) |
> | `.claude/skills/triage/SKILL.md` | `.claude/agents/triage-runner.md` AND `.claude/skills/next/SKILL.md` (Triage group) |
> | `.claude/agents/triage-runner.md` | `.claude/skills/triage/SKILL.md` AND `.claude/skills/next/SKILL.md` (Triage group) |
> | `.claude/skills/next/SKILL.md` | `.claude/skills/triage/SKILL.md` AND `.claude/agents/triage-runner.md` (Triage group) |
> | `AGENTS.md` (rule add / exemption) | Run `grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` and apply the same change to every match. **For new pre-resolved rules** (the kind that should never reach a question): also add a corresponding entry to the Rule-5 substring blacklist in `.claude/agents/spec-writer.md` so the spec-writer subagent enforces it mechanically. |
> | `AGENTS.md` "Corrections Log" section (Boundary rules 1 / 2, entry format, `Escalated?` semantics) | `.claude/agents/self-improve.md` AND `.claude/agents/learnings-escalation-audit.md` (Corrections-Log group — the two agents that read/write `learnings.md` must match the rules they enforce) |
> | `.claude/skills/task/SKILL.md` (Steps 6–8 design phase contract) | `.claude/agents/design.md` AND `.claude/agents/design-review.md` (Task/Design group — design's artefact format, design-review's verdict format incl. GO-with-notes round-trip, and task SKILL Step 8's verification of that round-trip all co-evolve) |
> | `.claude/agents/design.md` | `.claude/skills/task/SKILL.md` Steps 6–8 AND `.claude/agents/design-review.md` (Task/Design group) |
> | `.claude/agents/design-review.md` | `.claude/skills/task/SKILL.md` Steps 6–8 AND `.claude/agents/design.md` (Task/Design group) |
> | `quartzite-widgets/tests/support/mod.rs` | `quartzite-style/tests/support/mod.rs` (Snapshot-helper group) |
> | `quartzite-style/tests/support/mod.rs` | `quartzite-widgets/tests/support/mod.rs` (Snapshot-helper group) |
> | `ai-docs/agent-writing-style.md` (new fail-loud pattern entry under `## Patterns`) | Each `.claude/skills/**/SKILL.md` and `.claude/agents/**.md` that carries — or should carry — the new pattern. The style guide names the *shape*; the downstream consumers carry the *body*. After adding or amending a pattern, run `grep -rn "<pattern-keyword>" .claude/agents/ .claude/skills/` to find any file already half-using the pattern and reconcile. (Pattern 7 *Compaction recovery callout* is the introducing case — see `ai-docs/agent-writing-style.md § Pattern 7` for the variant taxonomy.) |
> | Any other instruction file | Run the same grep — the Procedure (below) catches lingering references |

> The former `task` ↔ `task-issue` group collapsed when `task-issue` was merged into `task` — both entry modes now live in `.claude/skills/task/SKILL.md`. Grep across `.claude/skills/` and `.claude/agents/` per the procedure below to catch any lingering references.

**Procedure:**
1. Before closing the edit, `grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` for any file that references the same rule, exemption, or terminology.
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
| `ai-docs/workflow.md` | Extracted narrative passages from `AGENTS.md` § *Workflow* (PR review comment resolution GraphQL recipe). Read on demand. |
| `ai-docs/corrections-log.md` | Extracted carve-outs from `AGENTS.md` § *Corrections Log* (Boundary rule 1 / 2 Exception bodies + entry-format field glossary). Read on demand. |
| `ai-docs/key-decisions.md` | Extracted Key Design Decisions detail bodies from `ai-docs/context.md` § Key Design Decisions (implementation-detail rows). Read on demand. |
| `ai-docs/dependency-versions.md` | Live-lookup reference for Cargo / GitHub Action versions and Action behaviour verification — extracted from `AGENTS.md` § *Dependency Versions* AXIOM. Read on demand when writing a specific version string or a load-bearing claim about an Action's behaviour. |
| `ai-docs/agent-writing-style.md` | Style for binary rules in instruction files (dual-model readability) — read on demand and when editing any of `AGENTS.md`, `.claude/skills/**`, `.claude/agents/**`, `ai-docs/code-style.md`, `ai-docs/doc-convention.md` |
| `ai-docs/templates/` | Shared reference templates consumed by multiple skills / agents. Multi-consumer reference material lives here (project-level reference, not Claude Code configuration). Single-consumer skill templates remain inside the owning skill directory per the Claude Code [supporting-files pattern](https://code.claude.com/docs/en/skills#add-supporting-files). |
| `ai-docs/templates/progress-format.md` | Canonical `.progress.md` format spec — template + required vs optional fields + lifecycle. Consumed by `/task`, `/code-review`, `/pr-commented`, `review-findings`, `self-review`. |
| `ai-docs/plans/INDEX.md` | Plan index — statuses and dependency order |
| `ai-docs/plans/*.spec.md` | Active task spec + acceptance criteria |
| `ai-docs/plans/*.design.md` | Active task design documents |
| `ai-docs/plans/*.progress.md` | Active task progress / handoff state — **local-only (gitignored)**. Carries the extended compaction-recovery schema (`current_step`, `last_passed_gate`, optional `parent_skill`, optional `entry_args`) plus a `## Decisions log` section per the canonical template at `ai-docs/templates/progress-format.md`. Writers: `/task` (creates at Step 8 and writes at each Step 8–12 boundary), `/code-review` (creates at Phase 1 and writes at each phase boundary), `/pr-commented` (extends with per-round sections), `/bugfix` (extends its own trace file with the same fields inline — see row below). Deleted by `/pr-merged` after the PR merges. Never committed. |
| `ai-docs/pr-comments/pr-<N>.progress.md` | Fallback progress file when `/pr-commented` runs on a PR not produced by `/task` (rare). **Local-only (gitignored)**. Deleted by `/pr-merged`. |
| `ai-docs/triage/triage-YYYY-MM-DD.progress.md` | `/triage` resume state for multi-turn runs (dedupe map summary, bridge classifications, Phase 6 / 7 partitions, `## Next action`). **Local-only (gitignored)**. Created by `triage-runner` at Phase 1.5; extended through Phases 4–7; deleted at Phase 8 after the run summary emits. |
| `ai-docs/plans/done/` | Completed plans (spec + design, implemented) |
| `ai-docs/plans/deferred/` | Blocked or future plans |
| `ai-docs/deferred/_inbox.md` | triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only). |
| `ai-docs/bugfix/trace-*.md` | Bugfix traces — deleted on resolution. Carries the same compaction-recovery fields inline (`**current_step:**`, `**last_passed_gate:**`, `**parent_skill:**`, `**entry_args:**`) plus a `## Decisions log` section — the trace file IS the `/bugfix` durable-state surface, no parallel `.progress.md`. |
| `ai-docs/learnings.md` | Corrections log — feed for `/improve` |
| `.claude/agents/spec-writer.md` | Spec-writer subagent (`model: opus`) — drafts the task spec one interview round per invocation; invoked by the `/interview` orchestrator |
| `.claude/skills/triage/SKILL.md` + `.claude/agents/triage-runner.md` | `/triage` skill — batched promotion of `Tracked` = `—` rows in `ai-docs/deferred/*.md` (+ `🟡 v2` rows in `widget-backlog.md`) to gh issues; drains `_inbox.md` per-entry. Opus subagent; mutation scope strictly `ai-docs/deferred/**` + `gh issue create/edit`. |
| `.claude/skills/pr-commented/SKILL.md` | `/pr-commented` skill — one round of reviewer-comment response on an open PR. Reads unresolved threads, auto-classifies (`fix` / `objection` / `clarify` / `already-fixed` / `defer` / `ignore-bot`), bundles fixes into one commit per invocation, runs `self-review`, pushes, replies + resolves per category. Re-invocable per round. Downstream of `/task` Step 12; does NOT replace `/task`. Never edits `ai-docs/learnings.md` (PR comments are external content). |

## Corrections Log

On non-obvious correction or confirmed approach, write to `ai-docs/learnings.md`. **Read the two boundary rules below before you write — both have been violated multiple times.**

### Boundary rule 1 — `ai-docs/learnings.md` is APPEND-ONLY

> **NEVER** edit, rewrite, reorder, summarise, or delete an existing entry in `ai-docs/learnings.md`. Only append new entries at the end of the file. This applies even when:
> - a newer correction supersedes an older one — write a NEW entry that says so, leave the old one intact
> - an entry turns out to be wrong, redundant, or poorly worded — write a NEW entry that corrects it
> - you are tempted to "tidy up" or "consolidate" the file
>
> The history of corrections (including superseded and wrong ones) is itself the artefact `/improve` audits. Editing past entries destroys that history.
>
> **Exception — `Escalated?` and `Superseded by:` fields, agent-driven only.** These two lines of an existing entry MAY be updated in-place by the `self-improve` agent (via `/improve`) and the `learnings-escalation-audit` agent (via `/ai-audit` Phase 1) — see [`ai-docs/corrections-log.md` → Boundary rule 1 Exception](ai-docs/corrections-log.md#boundary-rule-1-exception) for the per-agent contract. All other lines of an entry (date, category, description, **What happened**, **Rule**) remain immutable. New learning entries are still append-only. Manual user edits to `Escalated?` / `Superseded by:` are **NOT** authorised by this exception — invoke `/ai-audit` or explicitly request the change.

### Boundary rule 2 — writing to `learnings.md` triggers NO other rule-file edits in the same turn

> When you write to `ai-docs/learnings.md`, you **MUST NOT** also edit any of these files in the same conversation turn:
>
> - `AGENTS.md`
> - `CLAUDE.md`
> - `.claude/skills/**` (any file)
> - `.claude/agents/**` (any file)
> - `.claude/settings.json`
> - `ai-docs/code-style.md`
> - `ai-docs/doc-convention.md`
>
> Writing a learning entry is **NOT** authorisation to escalate the rule into instruction files. Set `Escalated? no` and stop. Project-level escalation happens only when:
>
> 1. The user runs `/improve` (which spawns the escalation agent), OR
> 2. The user explicitly asks ("escalate this", "update AGENTS.md", "add to skill X").
>
> The Propagation Rule fires only when you are *already* editing an instruction file for an independent reason — it does not authorise pre-emptive escalation triggered by a fresh `learnings.md` entry. The same applies in reverse: if the user corrects a behaviour and asks you to record it, write to `learnings.md` only — do not also "fix" `AGENTS.md` or `code-style.md` in the same turn.
>
> **Exception — `/improve` and `/ai-audit` workflows.** The `self-improve` agent (via `/improve`) and the `learnings-escalation-audit` agent (via `/ai-audit` Phase 1) MAY update `Escalated?` / `Superseded by:` on existing entries alongside instruction-file edits — see [`ai-docs/corrections-log.md` → Boundary rule 2 Exception](ai-docs/corrections-log.md#boundary-rule-2-exception). The exception applies to **existing-entry `Escalated?` / `Superseded by:` updates ONLY** — NEW learning entries STILL **cannot** be appended in the same turn as instruction-file edits (Rule 2's main protection stays intact).
>
> **Exception — in-flow learning capture during `/task` Steps 8–12.** A NEW learning entry MAY be appended to `ai-docs/learnings.md` in the same conversation turn as an instruction-file edit when **all** of these hold: (a) the running skill is `/task` (Steps 8–12 — Implementation through Finalise), **or any sub-skill (e.g., `/bugfix`, `/context-reset`) invoked from within `/task` Steps 8–12**, (b) the entry documents an insight gained **during** the task being implemented (not a pre-emptive escalation of a rule the task already enforces), AND (c) the entry is marked `Escalated? no` (any project-level escalation requires a separate `/improve` invocation per Rule 2's main body). Bare instruction-file edits outside a `/task` flow remain bound by the main rule. See [`ai-docs/corrections-log.md` → Boundary rule 2 Exception](ai-docs/corrections-log.md#boundary-rule-2-exception) for the full body.

### Entry format

```
### YYYY-MM-DD — [category] — [short description]
**What happened:** [quote or paraphrase]
**Rule:** [what to do instead, or what to keep doing]
**Escalated?** no | AGENTS.md | skill:[name] | hook | settings | agent:[name] | doc-convention | code-style (comma-separate multiple)
**Superseded by:** [ref] — [one-line reason]    (optional; omitted when not applicable)
```

See [`ai-docs/corrections-log.md` → Entry format — field glossary](ai-docs/corrections-log.md#entry-format--field-glossary) for the semantics of each field (`Escalated?` values, `doc-convention` vs `code-style`, `Superseded by:` reference format).

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
