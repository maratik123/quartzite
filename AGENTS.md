# Rust OSS Agent Rules

**CRITICALLY**
1) English for all output. Other language only on explicit user request.
2) Minimise thinking output shown to user.

## Project

> Read `ai-docs/context.md` for project purpose, entities, architecture, and design decisions — on demand.

## Design system

> Read [`design-system/SKILL.md`](design-system/SKILL.md) (manifest) and [`design-system/README.md`](design-system/README.md) (visual rules) on demand; explore `design-system/preview/`, `design-system/colors_and_type.css`, and `design-system/ui_kits/widgets/` as needed. Pointer-only — not auto-imported. Trigger conditions:
> - When working on `quartzite-style` (any `Style` impl, including `DefaultStyle`)
> - When working on `quartzite-widgets` paint paths, widget views, or any user-facing rendering
> - When changing `Palette` / `ColorRole` semantics or seeds
> - When adding or modifying snapshot tests under `quartzite-style/tests/snapshots/`
> - When working on quartzite-paint-api painter primitives, brush, pen, path, font, or color

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
cargo clippy --workspace --all-targets -- -D warnings   # lint (strict; --all-targets covers benches, tests, examples in addition to lib+bins)
cargo fmt                             # fix formatting
cargo fmt -- --check                  # check only
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features   # doc gate (matches CI; --all-features so intra-doc links into every feature-gated module — `serde`-gated `snapshot`, `style`, `widgets`, … — resolve regardless of which feature gates them)
cargo build -p quartzite --no-default-features --features libm   # verify derive-free / no_std path compiles
actionlint .github/workflows/<file>.yml   # required gate for any new/modified workflow file
```

> See [`ai-docs/miri-policy.md`](ai-docs/miri-policy.md) for the per-test `#[cfg_attr(miri, ignore = "…")]` default + per-file `#![cfg(not(miri))]` fallback + workflow-level exclusion-list contract.

> **AXIOM — `actionlint` MUST pass before `git add` on any modified workflow file.**
> Required gate, same status as `cargo build` and `cargo clippy --workspace --all-targets -- -D warnings`. Skipped twice despite the rule existing — escalated to AGENTS.md after the second occurrence.
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

Search: `ast-index` first (see [`.claude/rules/ast-index.md`](.claude/rules/ast-index.md)); fall back to `rg <pattern> --type rust [-l | -C 3]` when `ast-index` returns empty.

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

See [`ai-docs/api-naming.md` → The _unchecked AXIOM](ai-docs/api-naming.md#the-_unchecked-axiom) for the `_unchecked` AXIOM + naming rules.

## Code Style

- **Source files:** Rust-only (`.rs`) under `src/`; max 100 cols (rustfmt default); format via `cargo fmt`, never `rustfmt <file>` directly. See [`ai-docs/code-style.md` → Source files](ai-docs/code-style.md#source-files).
- **Linter posture:** strict clippy enforced (`-D warnings`); no blanket `#[allow]` without justification; workspace-wide lint policy lives in root `Cargo.toml` `[workspace.lints.rust]` + `[workspace.lints.rustdoc]` + `[workspace.lints.clippy]` (with `clippy.toml` for size-aware thresholds). See [`ai-docs/code-style.md` → Linter posture](ai-docs/code-style.md#linter-posture).
- **Rust idioms:** prefer Rust over literal ports; let chains valid (edition 2024); comparison helpers (`.min`/`.max`/`.clamp`/`Option::or`/`Option::filter`) over explicit `if`/`match`; **never cite GUI/UI frameworks** (Qt, GTK, WinForms, SwiftUI, …) as design justification. See [`ai-docs/code-style.md` → Rust idioms](ai-docs/code-style.md#rust-idioms).
- **Magic numbers:** numeric literals with semantic meaning → module-level `const SCREAMING_SNAKE_CASE`, not inline. Self-evident constants (`0`, `1`, `-1`, `2`) and test fixtures exempt. See [`ai-docs/code-style.md` → Magic numbers](ai-docs/code-style.md#magic-numbers).
- **Library safety idioms:** `parking_lot::Mutex` / `parking_lot::RwLock` are workspace default (non-poisoning, infallible `.lock()`/`.read()`/`.write()`); safe primitives (`OnceLock` / `Arc` / `Weak` / `AtomicBool`) over raw pointers + `unsafe`. `.unwrap_or_else(|e| e.into_inner())` survives **only** for rare FFI-imposed `std::sync::*Lock` retainees (none in-tree). See [`ai-docs/code-style.md` → Library safety idioms](ai-docs/code-style.md#library-safety-idioms).
- **Documentation:** workspace declares `missing_docs = "deny"` + `rustdoc::broken_intra_doc_links = "deny"` + `clippy::undocumented_unsafe_blocks = "deny"` in `[workspace.lints.*]`; each crate opts in via `[lints] workspace = true`; every public item has at least one-line `///`; `# Examples` block on new public items with single-line docs; doc-style conventions live in [`ai-docs/doc-convention.md`](ai-docs/doc-convention.md). See [`ai-docs/code-style.md` → Documentation](ai-docs/code-style.md#documentation).
- **Error types:** `thiserror` for new error enum/struct; hand-rolled `Display`/`Error` reserved for cases the derive cannot express. See [`ai-docs/code-style.md` → Error types](ai-docs/code-style.md#error-types).
- **Tracing:** `*_span!` guard wrapping the body of any function that meaningfully mutates application state; `debug_span!` for lifecycle, `trace_span!` for supplementary; high-frequency paths gated behind `verbose-tracing` cargo feature. See [`ai-docs/code-style.md` → Tracing](ai-docs/code-style.md#tracing).
- **`#[inline]` and the `_Simple._` doc tag:** mark every recursively-simple fn (no branches/loops, ≤ 1 non-simple call) with the marker matching its shape — `#[inline]` (concrete fn **or** method inside `impl Trait for ConcreteFoo` — concrete-impl trait method needs `#[inline]` for cross-crate inlining without LTO; `// _Simple._` is *not* a substitute), `/// _Simple._` (generic free fn / inherent generic method / trait method declaration whose every conforming impl is required to be simple), `// _Simple._` (method inside `impl<T> Trait for Foo<T>` — avoids overriding trait-inherited rustdoc); strip + cascade re-test of callers when an edit makes a previously-simple fn non-simple. See [`ai-docs/code-style.md` → `#[inline]` and the `_Simple._` doc tag](ai-docs/code-style.md#inline-and-the-_simple_-doc-tag).
- **Generic-fn split for binary size:** public fn with conversion-style generic param (`impl Into<T>` / `impl AsRef<T>` / `impl ToString`) and > 3 line body extracts the body into a nested `fn inner(...)` (NOT a sibling `<outer>_inner` impl method); outer carries `_Simple._`; **if the inner ends up simple, unwrap it** — `#[inline]` simple inner is dead weight. See [`ai-docs/code-style.md` → Generic-fn split for binary size](ai-docs/code-style.md#generic-fn-split-for-binary-size).
- **File size:** target 200–400 lines per `.rs` file excluding `#[cfg(test)]`; soft 500/800; hard 1000/1500 (refactor before merge unless exempt — auto-generated, single state machine / `match`, `macro_rules!`); per-fn `clippy::too_many_lines` (>100); counter-rule against over-splitting (one-struct-per-file is not Rust idiom). See [`ai-docs/code-style.md` → File size](ai-docs/code-style.md#file-size).

See [`ai-docs/code-style.md`](ai-docs/code-style.md) for the canonical reference.

## Dependency Versions

> **AXIOM — Query live state BEFORE asserting any claim about an external dep or the project's own dep graph. Memory is stale.**
> Three dimensions of "I remember X is the case" have each landed wrong claims in this repo:
> 1. **Version of an external dep** — `criterion = 0.5` when live is `0.8`; `actions/deploy-pages@v4` when live is `@v5`.
> 2. **Behaviour of a third-party Action** — "the action sets `RUSTC_WRAPPER` by default" when `src/setup.ts` shows it does not (PR #179 sccache).
> 3. **Presence of a dep in the current project** — "would add `parking_lot` as a new dep" when `cargo tree --invert parking_lot` shows it's already there (issue #440).
>
> See [`ai-docs/dependency-versions.md`](ai-docs/dependency-versions.md) for the per-dimension lookup recipes. Apply the pinning rule (below) to the **observed** version, never the remembered one.
>
> | If you're about to write... | Verify first with |
> |---|---|
> | A specific version of crate `X` | `curl -sS "https://crates.io/api/v1/crates/X" \| jq -r '.crate.max_stable_version'` |
> | A claim that Action `X` sets / exports / defaults to `Y` | Read `action.yml` + `src/setup.ts` / `src/main.ts` per the recipe |
> | A claim that `X` is / isn't / would-be-added-as a dep in this project | `grep -r '<X>' --include='Cargo.toml' .` AND `cargo tree --invert <X>` (the latter catches transitive presence) |
>
> If your draft contains substrings like *"would add"*, *"introduce X as a dep"*, *"pull in X"*, *"avoid X as a dep"*, *"X is not currently a dependency"* — STOP, run the grep + cargo-tree check, and either rewrite with the actual trade-off (perf / feature-gate / test-prod parity / binary-size) or drop the claim.

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

- Merge PRs via merge commit (`gh pr merge --merge`); never squash/rebase-merge. → [§ Merge strategy](ai-docs/workflow.md#merge-strategy)
- Run `cargo build` before commit so `Cargo.lock` refreshes. → [§ Cargo.lock refresh before commit](ai-docs/workflow.md#cargolock-refresh-before-commit)
- Stage explicitly; **Never** `git add -A` / `.`. → [§ Explicit-file staging](ai-docs/workflow.md#explicit-file-staging)
- **Before every `git commit` during a PR task**, stage `ai-docs/learnings.md` with related code. **After every push**, give a post-push learning entry its own commit. → [§ Staging learnings.md during PR commits](ai-docs/workflow.md#staging-learningsmd-during-pr-commits)
- **Never** `git commit --no-verify` (or any hook-skip flag) — fix the hook. → [§ No --no-verify](ai-docs/workflow.md#no---no-verify)
- **CI-fix commits get self-review too.** Spawn `self-review` before pushing any CI-fix commit. → [§ CI-fix commit self-review (parent rule)](ai-docs/workflow.md#ci-fix-commit-self-review-parent-rule) + [§ Self-review checklist for CI-fix commits](ai-docs/workflow.md#self-review-checklist-for-ci-fix-commits)
- **No "too simple" step-skip in `/task`.** Steps 6 / 7 / 10 are MANDATORY; user authorisation is the only bypass. → [§ "Too simple" step-skip rule (parent rule)](ai-docs/workflow.md#too-simple-step-skip-rule-parent-rule)
- **NEVER** `git reset --hard` — discards uncommitted work. → [§ Recovery from destructive-git-commands](ai-docs/workflow.md#recovery-from-destructive-git-commands)
- Plan first. Tests before prod code (TDD). Lint changed files. → [§ TDD + lint-changed-files](ai-docs/workflow.md#tdd--lint-changed-files)
- Files with ~50+ lines of substantial logic MUST have a `#[cfg(test)] mod tests` block (exceptions: `examples/`, `benches/` with `harness = false`). → [§ #[cfg(test)] requirement for substantial logic](ai-docs/workflow.md#cfgtest-requirement-for-substantial-logic)
- `.gitignore` (not `.arcignore`).
- After generating/moving a markdown file with relative links, trace one link via `realpath` before committing. → [§ Markdown link tracing after generate/move](ai-docs/workflow.md#markdown-link-tracing-after-generatemove)
- **PR review comment resolution:** Resolve only comments fixed by code; objections stay open for the reviewer. → [§ PR review comment resolution](ai-docs/workflow.md#pr-review-comment-resolution)

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
> See [`ai-docs/workflow.md` → PR body vs. tracking-issue body](ai-docs/workflow.md#pr-body-vs-tracking-issue-body) for the issue-vs-PR-body distinction.

> **AXIOM — Every code-producing commit on a feature branch with an open PR (or about-to-be-opened PR) must pass `self-review` before `git push`.**
> The per-skill rules already exist (`/task` Step 10, `/pr-commented` Step 5, `/pr-ci-failed` Step 5, `/master-ci-failed` Step 5, `/bugfix` Step 6). This AXIOM names them as instances of a single workspace rule, so the next surface that doesn't yet have its own per-skill step still falls under the rule.
>
> | If the commit is... | Action |
> |---|---|
> | Initial implementation in `/task` | `/task` Step 10 — spawn `self-review` |
> | Reviewer-comment fix in `/pr-commented` | `/pr-commented` Step 5 — spawn `self-review` |
> | CI-failure fix in `/pr-ci-failed` / `/master-ci-failed` | per-skill `self-review` step |
> | Bugfix in `/bugfix` (standalone or detoured from `/task`) | `/bugfix` Step 6 — spawn `self-review` |
> | Ad-hoc / out-of-skill fix on a feature branch with an open PR | Spawn `self-review` manually over `git diff <merge-base>..HEAD` before `git push` |
> | Docs-only / instruction-file-only commit (no `.rs` diff) | Self-review optional; still required if the diff touches any user-facing artefact |
>
> APPROVE = push. REJECT = fix on the same branch and re-run; after 3 REJECTs in a row, surface and stop without pushing.

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
> | `AGENTS.md` (rule add / exemption) | Run `grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ .claude/rules/ AGENTS.md ai-docs/agent-writing-style.md` and apply the same change to every match. **For new pre-resolved rules** (the kind that should never reach a question): also add a corresponding entry to the Rule-5 substring blacklist in `.claude/agents/spec-writer.md` so the spec-writer subagent enforces it mechanically. |
> | `AGENTS.md` "Learning Log" section (Boundary rules 1 / 2, entry format incl. `Kind:`, `Escalated?` semantics, 🌱 verdict from `/ai-audit`) | `.claude/agents/self-improve.md` AND `.claude/agents/learnings-escalation-audit.md` (Learning-Log group — the two agents that read/write `learnings.md` must match the rules they enforce) |
> | `.claude/skills/task/SKILL.md` (Steps 6–8 design phase contract) | `.claude/agents/design.md` AND `.claude/agents/design-review.md` AND `.claude/skills/context-reset/SKILL.md` (Task/Design group — design's artefact format, design-review's verdict format incl. GO-with-notes round-trip, task SKILL Step 8's every-group `/context-reset` handoff contract, and context-reset's own trigger / `allowed-tools` / write-contract wording all co-evolve) |
> | `.claude/agents/design.md` | See *Task/Design group* anchor row above (`.claude/skills/task/SKILL.md`). |
> | `.claude/agents/design-review.md` | See *Task/Design group* anchor row above (`.claude/skills/task/SKILL.md`). |
> | `.claude/skills/context-reset/SKILL.md` | See *Task/Design group* anchor row above (`.claude/skills/task/SKILL.md`). |
> | `.claude/skills/task/SKILL.md` Step 7 *Spec Amendment recipe* (or any rule about spec-amendment → design → design-review re-entry) | `.claude/skills/pr-commented/SKILL.md` AND `.claude/skills/pr-ci-failed/SKILL.md` AND `.claude/skills/master-ci-failed/SKILL.md` (Spec-Amendment group — every downstream "fix" skill whose round can touch `ai-docs/plans/*.spec.md` carries the same recipe; the rule's mechanical detection trigger — *"the fix commit's diff includes a `.spec.md` file"* — applies identically in each surface) |
> | `quartzite-widgets/tests/support/mod.rs` | `quartzite-style/tests/support/mod.rs` (Snapshot-helper group) |
> | `quartzite-style/tests/support/mod.rs` | `quartzite-widgets/tests/support/mod.rs` (Snapshot-helper group) |
> | `ai-docs/agent-writing-style.md` (new fail-loud pattern entry under `## Patterns`) | See [`ai-docs/agent-writing-style.md` § *Propagation rule for new patterns*](ai-docs/agent-writing-style.md#propagation-rule-for-new-patterns). |
> | `ai-docs/skill-size-exemptions.md` | `.claude/skills/ai-audit/reference.md` (Checklist K item 1 anchor + cited `wc -l` numbers MUST stay synchronised; deferred `scripts/check-instruction-file-sizes.sh` (#383) reads the same index once landed) (Size-exemption-index group) |
> | Any other instruction file | Run the same grep — the Procedure (below) catches lingering references |

**Procedure:**
1. Before closing the edit, `grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ .claude/rules/ AGENTS.md ai-docs/agent-writing-style.md` for any file that references the same rule, exemption, or terminology.
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
| `ai-docs/workflow.md` | Extracted § Workflow narrative (PR-review-comment recipe) |
| `ai-docs/corrections-log.md` | Extracted § Learning Log carve-outs + field glossary |
| `ai-docs/key-decisions.md` | Key Design Decisions detail bodies from context.md |
| `ai-docs/plans-summary.md` | Maintenance-plans (cross-cutting) detail bodies from context.md |
| `ai-docs/dependency-versions.md` | Live Cargo / GitHub Action version lookup + behaviour recipes |
| `ai-docs/agent-writing-style.md` | Binary-rule writing style for dual-model readability |
| `ai-docs/agent-docs-index.md` | Verbose bodies of `§ Agent Docs` rows. Read on demand. |
| `ai-docs/api-naming.md` | `_unchecked` AXIOM + naming rules. Read on demand. |
| `ai-docs/skill-size-exemptions.md` | Audited list of `.claude/skills/*/SKILL.md` files exempted from the 200-line soft target; consumed by `/ai-audit` Checklist K item 1 + deferred `scripts/check-instruction-file-sizes.sh`. |
| `ai-docs/templates/` | Shared templates consumed by multiple skills / agents |
| `ai-docs/templates/progress-format.md` | Canonical `.progress.md` format spec (template + lifecycle) |
| `ai-docs/plans/INDEX.md` | Plan index — statuses and dependency order |
| `ai-docs/plans/*.spec.md` | Active task spec + acceptance criteria |
| `ai-docs/plans/*.design.md` | Active task design documents |
| `ai-docs/plans/*.progress.md` | Active task progress / handoff state — local-only (gitignored) |
| `ai-docs/pr-comments/pr-<N>.progress.md` | Fallback progress file for `/pr-commented` on non-`/task` PRs (gitignored) |
| `ai-docs/triage/triage-YYYY-MM-DD.progress.md` | `/triage` resume state for multi-turn runs (gitignored) |
| `ai-docs/plans/done/` | Completed plans (spec + design, implemented) |
| `ai-docs/plans/deferred/` | Blocked or future plans |
| `ai-docs/deferred/_inbox.md` | triage queue — rows from completed specs awaiting `/triage` |
| `ai-docs/bugfix/trace-*.md` | Bugfix trace + durable-state surface — deleted on resolution |
| `ai-docs/learnings.md` | Corrections log — feed for `/improve` |
| `.claude/agents/spec-writer.md` | Spec-writer subagent — drafts task spec one round per call |
| `.claude/skills/triage/SKILL.md` + `.claude/agents/triage-runner.md` | `/triage` skill — batched promotion of deferred rows to gh issues |
| `.claude/skills/pr-commented/SKILL.md` | `/pr-commented` skill — one round of reviewer-comment response |
| `.claude/skills/pr-ci-failed/SKILL.md` | `/pr-ci-failed` skill — one round of CI-failure response on PR |
| `.claude/skills/master-ci-failed/SKILL.md` | `/master-ci-failed` skill — one round of post-merge red-master fix |
| `.claude/skills/ui-design/SKILL.md` | `/ui-design` skill — pointer to `design-system/` (Read manifest + visual rules on demand) |
| `.claude/rules/ast-index.md` | On-demand code-search rules — `ast-index` mandatory-search + read-outline rules, plus the verbatim block subagents inherit (see also `§ Build & Test` Search line). |

See [`ai-docs/agent-docs-index.md` → Agent doc rows](ai-docs/agent-docs-index.md#agent-doc-rows) for the verbose body of each row (writers, lifecycle, special cases).

## Learning Log

On **ANY** instruction violation, of any kind, write a new entry to `ai-docs/learnings.md` — there is no "obvious", "minor", "trivial", "already-known", or "duplicate" disposition. The history (including recurrences and superseded entries) is the artefact `/improve` audits to decide escalation fan-out. See [`ai-docs/corrections-log.md` → FORBIDDEN reasoning for skipping a `learnings.md` write](ai-docs/corrections-log.md#forbidden-reasoning-for-skipping-a-learningsmd-write) for the enumerated list of skip-reasons that have been used in violation of this rule and are therefore explicitly disallowed. **Read the two boundary rules below before you write — both have been violated multiple times.**

### Boundary rule 1 — `ai-docs/learnings.md` is APPEND-ONLY

> **NEVER** edit, rewrite, reorder, summarise, or delete an existing entry in `ai-docs/learnings.md`. Only append new entries at the end of the file. This applies even when:
> - a newer correction supersedes an older one — write a NEW entry that says so, leave the old one intact
> - an entry turns out to be wrong, redundant, or poorly worded — write a NEW entry that corrects it
> - you are tempted to "tidy up" or "consolidate" the file
>
> The history of corrections (including superseded and wrong ones) is itself the artefact `/improve` audits. Editing past entries destroys that history.
>
> **Exception — `Escalated?` and `Superseded by:` fields, agent-driven only.** Both fields MAY be updated in-place by the `self-improve` agent (`/improve`) and the `learnings-escalation-audit` agent (`/ai-audit` Phase 1). See [`ai-docs/corrections-log.md` → Boundary rule 1 Exception](ai-docs/corrections-log.md#boundary-rule-1-exception) for the per-agent contract. All other lines of an entry remain immutable.
>
> **One-off carve-out — 2026-05-19 compaction-recovery-protocol entry.** Retro-tagged `**Kind:** validation` via PR #492 Phase 1 with `Superseded by:` line as audit trail. **Named, narrow, NOT a precedent.** Schema migrations require their own named carve-out. See [`ai-docs/corrections-log.md` → Boundary rule 1 Exception](ai-docs/corrections-log.md#boundary-rule-1-exception).

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
> **Exception — `/improve` and `/ai-audit` workflows.** `self-improve` (via `/improve`) + `learnings-escalation-audit` (via `/ai-audit` Phase 1) MAY update `Escalated?` / `Superseded by:` on existing entries alongside instruction-file edits. Existing-entry updates ONLY — NEW learning entries STILL cannot be appended in the same turn as instruction-file edits (Rule 2's main protection stays intact). See [`ai-docs/corrections-log.md` → Boundary rule 2 Exception](ai-docs/corrections-log.md#boundary-rule-2-exception).
>
> **Exception — in-flow learning capture during `/task` Steps 8–12.** A NEW learning entry MAY be appended in the same turn as an instruction-file edit when ALL hold: (a) running skill is `/task` Steps 8–12 (incl. sub-skills `/bugfix`, `/context-reset`); (b) entry documents an in-task insight (not pre-emptive escalation); (c) marked `Escalated? no`. See [`ai-docs/corrections-log.md` → Boundary rule 2 Exception](ai-docs/corrections-log.md#boundary-rule-2-exception) for the full body.

### Entry format

```
### YYYY-MM-DD — [category] — [short description]
**What happened:** [quote or paraphrase]
**Rule:** [what to do instead, or what to keep doing]
**Kind:** correction | validation    (optional; defaults to `correction` when omitted)
**Escalated?** no | AGENTS.md | skill:[name] | hook | settings | agent:[name] | doc-convention | code-style (comma-separate multiple)
**Superseded by:** [ref] — [one-line reason]    (optional; omitted when not applicable)
```

`Kind:` defaults to `correction` when omitted — existing entries need NO rewrite. Write `Kind: validation` for entries that document a working protocol / pattern the agent should keep doing (carrot signal); write `Kind: correction` (or omit) for entries that document a violation to stop doing (stick signal).

See [`ai-docs/corrections-log.md` → Entry format — field glossary](ai-docs/corrections-log.md#entry-format--field-glossary) for the semantics of each field (`Kind:` values, `Escalated?` values, `doc-convention` vs `code-style`, `Superseded by:` reference format).

Categories: `code-style` | `process` | `architecture` | `testing` | `documentation` | `tooling` | `search` | `other`

Run `/improve` when **≥3 unescalated correction entries**, **≥2 unescalated validation entries**, or a `🌱 Stale-validation` flag from `/ai-audit` accumulates.

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
