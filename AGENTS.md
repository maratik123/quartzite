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

- **All source files must be Rust (`.rs`).** No other languages in `src/`.
- Max line: 100 (rustfmt default).
- Strict clippy (enforced).
- Prefer Rust idioms over literal ports from other languages. When in doubt, ask. **Never cite other GUI/UI frameworks (Qt, GTK, WinForms, SwiftUI, etc.) as justification for design choices** in specs, designs, commit messages, or PR bodies. Justify decisions from Rust idioms, the crate ecosystem, and explicit reasoning about the problem.
- Let chains (`if let A = x && let B = y { ... }`) are valid in this codebase (edition 2024). Do not avoid them. Always format via `cargo fmt`, never `rustfmt <file>` directly.
- **Documentation:** Every crate must have `#![deny(missing_docs)]` and `#![warn(clippy::undocumented_unsafe_blocks)]` in its `lib.rs`. Every public item must have at least a one-line `///` doc comment. Every new public item with only a single-line doc must include a `# Examples` block. Proc-macro examples use `no_run`; runtime items needing an event loop use `no_run`; pure library types use compiling doctests.
- **Documentation Conventions:** See [`ai-docs/doc-convention.md`](ai-docs/doc-convention.md) for the canonical workspace doc style. Mandatory rules in brief: third-person-present-indicative summary line; `# Parameters` on every public fn with ≥1 arg; strict section order (`Parameters → Returns → Type parameters → Lifetimes → Errors → Panics → Safety → Examples → See also`); `# Errors` / `# Panics` / `# Safety` whenever applicable (clippy-enforced); intra-doc links over inline URLs; American English. Methods inside `impl Trait for Type {}` blocks are exempt — the trait definition documents the contract.
- **`#[inline]` and the `_Simple._` doc tag.** Mark every **simple** function so callers and reviewers can recognise it as inline-equivalent.

  **"Simple" (recursive definition):**
  - No branches or loops in the body, AND
  - At most one call to a **non-simple** function.

  Calls to other simple functions are "free" — apply the test transitively to each callee. After inlining, the chain collapses, so the binary cost is bounded regardless of how many simple-into-simple delegations the source contains.

  **Three visible markers, mutually exclusive by fn shape:**

  | Function shape | Marker | Why this marker |
  |---|---|---|
  | Concrete (no own type parameters AND `Self` is concrete — i.e. the surrounding `impl` block introduces no generics) | **`#[inline]` attribute** above the `fn` line | Needed for cross-crate inlining; visible in source. |
  | Generic — either the fn declares its own `<T>` / `<F: ...>` / `impl Trait` parameter, **or** it sits inside an `impl<T> ...` / `impl<T> Trait for Foo<T> { … }` block where `Self` is parametrised by the impl block's generics | **`_Simple._` doc tag** under the summary (see [`ai-docs/doc-convention.md`](ai-docs/doc-convention.md)) | Body is monomorphized per concrete `T`, so the compiler already has it cross-crate and `#[inline]` is redundant; the tag is the human signal that calls into this fn count as "free" in the recursive budget rule. |
  | Trait method declaration whose every conforming impl is required to be simple (typically because the impl is generated by a derive / proc-macro in this workspace, or because the trait's contract makes only simple impls valid) | **`_Simple._` doc tag** on the trait method declaration | Trait declarations carry no body; the tag publishes the contract so callers, reviewers, and hand-written-impl authors all see the same signal. A hand-written impl whose body is non-simple violates the trait contract. |

  Generic simple fns and tagged trait methods get the doc tag *instead of* `#[inline]`, not in addition to it — pick one based on shape.

  **Carve-out: default trait methods inside a `pub trait` body** (e.g. `ObjectExt::{id, name, is_on_current_thread}`) stay in the **concrete** row when their own bodies declare no type parameters. The body lives in a single source location (the trait declaration); only `Self` varies. Use `#[inline]`. This carve-out does **not** apply to methods inside an `impl<T> Trait for Foo<T>` block — those go to the **generic** row, even when the method itself declares no extra type parameters, because `Self = Foo<T>` is parametrised by the impl block's generics and the body is monomorphized per concrete `T`.

  **Where to place `_Simple._` for methods inside an `impl<T> Trait for Foo<T>` block.** Add the line `// _Simple._` (regular line comment, **not** a `///` doc comment) directly above the `fn` keyword inside the impl block. Rustdoc treats a `///` line on a trait-impl method as overriding the trait's inherited docstring — `/// _Simple._` on `ObjectRef<T>::clone` would replace `Clone::clone`'s actual docstring with just `_Simple._` on the rustdoc page. A regular `//` comment is invisible to rustdoc, stays a human-only marker visible in source, and preserves the trait-inherited docs. Audit grep `rg '_Simple\._'` matches both forms. Example: `Signal<Args>::default`, `ObjectRef<T>::clone`, `WeakRef<T>::eq`.

  **Marker-form decision tree:**

  | Position | Marker form |
  |---|---|
  | Concrete fn (no own type params, concrete `Self`) | `#[inline]` attribute |
  | Generic free fn / inherent generic method (`impl<T> Foo<T> { fn ... }`) | `/// _Simple._` doc line |
  | Trait method declaration (default method or method decl in a `pub trait` body) | `/// _Simple._` doc line — becomes part of the trait's docs and is inherited by all impls |
  | Method inside an `impl<T> Trait for Foo<T>` block (inherits docs from the trait) | `// _Simple._` line comment — avoids overriding inherited rustdoc |

  **Decision rule for tagging a trait method:** tag only when *every* conforming impl is required to be simple. If some valid impls are non-simple (different by-value shape, branches, loops), do **not** tag — callers cannot rely on it.

  **Trait-tag examples in this codebase:** `AsObject::{object_base, object_base_mut, as_any, as_any_mut}` (codegen always emits a single field/`self` access); `Object::{meta_object, connect_signal}` (simple by codegen contract). `Object::{read_property, write_property, invoke_method}` are **not** tagged — they contain `match` branches and are non-simple.

  **Typical targets** for `#[inline]` (concrete row): field getters (`self.field`), trivial wrappers (`.as_deref()`, single delegation call), `Default::default()` that calls `Self::new()`, `const fn` struct-literal constructors, wrappers whose body is one call into another simple fn (e.g. `ObjectExt::id` → `self.object_base().id()`).

  **Codegen mirroring:** emit `#[inline]` before each generated simple **concrete** `fn`; emit a `/// _Simple._` doc line before each generated simple **generic** `fn` and before each generated trait method whose conforming impls are always simple.

  **Marker maintenance.** When an edit makes a previously-simple fn non-simple (gains branches or loops, or > 1 call to a non-simple fn), strip its `#[inline]` attribute or `_Simple._` doc tag in the same edit — leaving a stale marker turns it into a lie (and a `_Simple._` tag is actively misleading, since callers count calls into the fn as "free" in the recursive budget). Then `rg <fn-name>` for callers and re-evaluate each: every caller whose simplicity depended on this callee being free must be re-tested by the recursive definition and de-marked if it no longer qualifies. Cascade until quiescent. The propagation cost is bounded — the simple class is small in practice (leaves and trivial wrappers) and the cascade rarely goes more than 2 levels deep. For trait methods: if a new conforming impl (hand-written or codegen) cannot be simple, strip `_Simple._` from the **trait declaration**, not the impl — the contract is what changed.
- **Generic-fn split for binary size.** When a public function takes a conversion-style generic parameter (`impl Into<T>`, `impl AsRef<T>`, `impl ToString`, …) and the body is more than ~3 lines, **place the body in a nested non-generic `fn inner(...)` defined inside the outer fn body**, and let the generic outer fn perform only the conversion. The outer fn carries the `_Simple._` doc tag (it is generic and simple by the recursive definition); the nested `inner` carries `#[inline]` only if it is itself simple, which it usually won't be. Only the trivial conversion shell is monomorphized; the body ships once.

  **Do not name the helper `<outer>_inner` and do not place it as a sibling impl method.** Nesting keeps the helper out of the type's namespace, makes it impossible to misuse from elsewhere, and matches the `std` idiom (`Path::new`, `PathBuf::from`, `OsStr::new`, …). Promote the helper to a sibling private fn **only if** a second caller actually needs it.

  **Does not apply** when the type parameter is used throughout the body (genuine generic logic — e.g. `F: FnOnce(...)` invoked inside the body): there is no non-generic remainder to extract.

  Pattern:
  ```rust
  /// Renames the object `id` to `new_name`, updating the name index.
  ///
  /// _Simple._
  ///
  /// # Examples
  /// ...
  pub fn rename(&mut self, id: ObjectId, new_name: impl Into<String>) {
      fn inner(this: &mut ObjectTree, id: ObjectId, new_name: String) {
          // body
      }
      inner(self, id, new_name.into())
  }
  ```

  A nested `fn` cannot capture `self`; pass it explicitly as `this: &mut Self` (or use a closure when capture matters; the closure form is acceptable but less greppable).
- **Comparison helpers.** Use `.min()`, `.max()`, `.clamp()`, `Option::or` / `Option::filter`, etc. instead of explicit `if`/`match` for simple value selection. Applies to all numeric types and any `Ord`/`PartialOrd` types. Reach for the stdlib method first; fall back to branching only when the comparison logic is genuinely non-trivial.
- **Library safety idioms.** Concrete forms of the "non-panicking APIs for libraries" rule (see *API Naming*):
  - **Mutex locks in `Option`/`Result`-returning fns:** use `mutex.lock().ok()?` (or `.unwrap_or_else(|e| e.into_inner())` to recover the inner value). Reserve `.lock().unwrap()` for cases where poisoning genuinely indicates an unrecoverable broken global invariant — and even then prefer `.expect("reason")`.
  - **Prefer safe primitives over raw pointers.** If a `OnceLock` / `Arc` / `Weak` already in scope holds the value, an `AtomicBool` flag is enough to track liveness — do not reach for `AtomicPtr` + `unsafe`. Reserve `unsafe` for cases where no safe construct expresses the semantic.
- **Error types.** Use `thiserror` for any new error enum/struct in this workspace — it eliminates boilerplate `Display` / `std::error::Error` impls and keeps error definitions concise. Hand-rolled `Display` / `Error` impls are reserved for cases where `thiserror`'s derive cannot express the required behaviour.
- **Trace significant state mutations.** Functions that make a meaningful, traceable change to application state should open with a `*_span!` guard (`let _span = …_span!(…).entered();`) wrapping the entire function body. Threshold: "would someone debugging a live system want to see this in a log?" Skip trivial getters/setters.
  - Use **`debug_span!`** for significant mutations — object lifecycle (insert, destroy, reparent), timer lifecycle (start/stop), event loop/pool lifecycle.
  - Use **`trace_span!`** for supplementary or lower-significance mutations — name changes, connection bookkeeping, posting closures.
  - High-frequency paths (e.g. signal emit or event loops) must gate the span behind the `verbose-tracing` cargo feature rather than always-on.
  - When adding a sibling to an existing traced function, apply the same treatment so the log story stays consistent.
- **File size.** Target **200–400 lines per `.rs` file excluding `#[cfg(test)]`** (readability sweet spot — fits in mental RAM, supports cohesive grouping of a struct + its `impl` blocks + related errors).
  - **Soft limit:** 500 lines excl. tests / 800 incl. tests. Trigger a split-by-responsibility check (e.g. `models.rs` / `db.rs` / `handlers.rs`) — do **not** split mechanically by line count.
  - **Hard limit:** 1000 lines excl. tests / 1500 incl. tests. Refactor before merge unless an exemption applies.
  - **Exemptions:** auto-generated / codegen output (build scripts, proc-macro emission targets); a single state machine or `match` where splitting would obscure the control flow; `macro_rules!` definitions.
  - **Counter-rule — do not over-split.** One-struct-per-file (Java / C# habit) is not Rust idiom and bloats the `mod` tree. Prefer one cohesive 300-line file over three 100-line fragments.
  - **Per-function:** Clippy's `too_many_lines` (>100) is the canonical fn-level signal — keep functions under it. Small functions naturally yield small files.

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
**Escalated?** no | AGENTS.md | skill:[name] | hook | settings | agent:[name] | doc-convention | memory (comma-separate multiple)

> `memory` = saved to global memory only. `/improve` treats it as unescalated — the entry remains a candidate for project-level escalation (AGENTS.md / skill / agent / hook / settings / doc-convention). `settings.local` does NOT count as project-level.
> `doc-convention` = the rule landed in `ai-docs/doc-convention.md`. Use only for documentation-style rules that genuinely belong in the workspace doc-convention reference rather than in AGENTS.md or a skill.
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
