# Learnings

### 2026-05-07 — code-style — if the `fn inner` of a generic-fn split is simple, unwrap it (don't `#[inline]` it)

**What happened:** Applied the "Generic-fn split for binary size" pattern to four fns. Two of the four inner fns (`ObjectFactory::register`'s inner — single `HashMap::insert`; `ObjectBase::named`'s inner — single `ObjectBase::new()` call + struct literal) are concrete and simple by the recursive definition. The first reaction was to add `#[inline]` to those two simple inners, matching the concrete-row marker rule. User then pointed out that `#[inline]` on a simple inner causes the compiler to inline it back into the (per-`T`-monomorphized) outer fn, which **defeats the split entirely** — the body ends up duplicated per concrete `T` anyway, just with extra source indirection. The right answer is to unwrap a simple inner, not to mark it.

**Rule:** Run the recursive simplicity test on every `fn inner` body of a generic-fn split. If the inner qualifies as simple (no branches/loops, ≤ 1 non-simple call), **delete the wrapper and put the body directly in the outer fn**. Then re-evaluate the outer's `_Simple._` marker per the marker-maintenance rule — unwrapping moves the inner's non-simple calls (≤ 1) into the outer, which can flip the outer from simple to non-simple, requiring the tag to be stripped. Keep the split only when `inner` is non-simple, which is the case the rule was designed for. An `#[inline]` simple `fn inner` inside a generic-fn split is a code smell: either the split was unnecessary (unwrap), or the inner is mis-marked.

**Why:** The split's value is exactly to keep the body out of the per-`T` monomorphization. `#[inline]` on a simple inner re-inlines it back into the outer, so the body lands in every `T` instantiation — same binary cost as never-having-split, but with extra source noise. A simple inner is by definition ≤ ~2 lines, so unwrapping never re-triggers the "> 3 lines → split" threshold.

**How to apply:** After writing a `fn inner(...)`, run the recursive simplicity check on its body. If simple → unwrap (delete the inner, hoist the body into the outer). If non-simple → keep the split, no marker on the inner. Do NOT add `#[inline]` to a simple inner inside a split — the split is the wrong tool for that body.

**Escalated?** AGENTS.md, agent:review-findings, agent:self-review

### 2026-05-07 — code-style — methods inside `impl<T> Trait for Foo<T>` are generic-shaped and need `_Simple._`, not `#[inline]`; and use `// _Simple._` (not `///`) on trait-impl methods

**What happened:** PR #120 (issue #115 implementation) added `#[inline]` to `Signal<Args>::default`, `ObjectRef<T>::clone`/`eq`, `WeakRef<T>::clone`/`eq`. All five methods sit inside `impl<T> Trait for Foo<T>` blocks and have no own type parameters, but `Self` is parametrised by the impl block's generics. The original carve-out wording was ambiguous enough to be read as covering this case. The first fix replaced `#[inline]` with `/// _Simple._`, but rendered rustdoc revealed `///` *overrides* the trait-inherited docstring — `Clone::clone`'s actual doc ("Returns a copy of the value...") was wiped out and replaced by just `_Simple._` on the rustdoc page.

**Rule (combined):**
1. From the Rust compiler's point of view, methods inside `impl<T> ...` / `impl<T> Trait for Foo<T>` blocks are generic-shaped — the body is monomorphized per concrete `T` even when the method declares no extra type parameters. They go to the **generic** row → use `_Simple._`. The carve-out for default trait methods applies *only* to default methods inside a `pub trait` body (where the body lives in the trait declaration, a single source location).
2. **Marker form by position:** `/// _Simple._` (doc line) for generic free fns, inherent generic methods, and trait method declarations — these need their own docs. `// _Simple._` (regular line comment) for methods inside `impl<T> Trait for Foo<T>` blocks — these inherit the trait's docs and a `///` line would silently replace them on the rustdoc page. Audit grep `rg '_Simple\._'` matches both forms.

**Why:** `#[inline]` on a generic `impl<T>` method is redundant (monomorphization already makes the body cross-crate-available); the marker should be `_Simple._`. But `///` on a trait-impl method is a documentation regression (overrides inherited rustdoc). `//` keeps the marker as a human-only signal in source while leaving the trait-inherited rustdoc intact. Verified against rendered HTML in `target/doc/`.

**How to apply:** When deciding the marker for a method, look at the surrounding `impl` block's generic parameter list, not just the method's own. If the impl block introduces generics that parametrise `Self`, use `_Simple._`. If you're inside a trait-impl block (`impl<T> Trait for Foo<T> { fn ... }`), use `//` not `///`. Reviewer/self-reviewer rejects `#[inline]` on generic-shape methods AND rejects `///` on trait-impl methods.

**Escalated?** AGENTS.md, doc-convention, agent:review-findings, agent:self-review

### 2026-05-07 — code-style — strip `#[inline]` / `_Simple._` when a fn stops being simple, and cascade to callers

**What happened:** During design discussion of the recursive `#[inline]` rule, the user surfaced a maintenance gap: when a previously-simple fn grows branches/loops or a second non-simple call, its marker (`#[inline]` or `_Simple._`) becomes a lie, and any caller that was simple only because this callee was treated as free may itself no longer qualify. The original rule covered when to *add* the marker; it said nothing about when to *strip* it.

**Rule:** When an edit makes a previously-simple fn non-simple, strip its `#[inline]` attribute or `_Simple._` doc tag in the same edit. Then `rg <fn-name>` for callers and re-test each by the recursive simple definition; de-mark any caller that no longer qualifies. Cascade until quiescent. For trait methods: if a new conforming impl (hand-written or codegen) cannot be simple, strip `_Simple._` from the trait *declaration*, not the impl — the contract is what changed.

**Why:** A stale `_Simple._` is actively misleading because callers count calls into the tagged fn as "free" in the recursive budget rule, so a stale tag silently mis-classifies callers as simple too. The cascade is cheap in practice — the simple class is small (leaves and trivial wrappers) and rarely goes more than 2 levels deep.

**How to apply:** When editing any fn body, re-run the recursive simple test on it before committing. If the marker no longer matches, strip it and walk callers with one `rg`. Reviewer/self-reviewer flags stale markers as REJECT/`major` because the misleading effect propagates.

**Escalated?** AGENTS.md, doc-convention, agent:review-findings, agent:self-review

### 2026-05-06 — code-style — `#[inline]` rule is recursive; three markers by fn shape

**What happened:** `ObjectExt::id` (and other simple wrappers, including generic methods on `Signal<Args>` / `ObjectRef<T>` / `WeakRef<T>` and codegen-driven trait methods such as `AsObject::object_base`) was left without `#[inline]` because the rule (a) counted source-level calls flatly with a "≤ 1 fn call" budget that did not see through other `#[inline]` callees, and (b) blanket-excluded generic / blanket-impl fns as "monomorphized, no benefit". Even after relaxing those, generic simple fns and trait method declarations had no visible marker, so the recursive budget rule was not auditable from a fn or trait surface.

**Rule:** "Simple" is recursive: no branches/loops AND ≤ 1 call to a non-simple fn (apply transitively to each callee). Mark every simple fn / trait method by shape:
- concrete simple fn → `#[inline]` attribute (the existing marker);
- generic simple fn (or simple-bodied default method on a generic blanket impl) → `_Simple._` doc tag, italic line below the summary, before any `#` heading;
- trait method whose every conforming impl is required to be simple (typically codegen-driven — `AsObject::*`, `Object::{meta_object, connect_signal}`) → `_Simple._` doc tag on the trait method declaration. Hand-written impls of a tagged trait method MUST be simple; non-simple impl violates the trait contract.

Markers are mutually exclusive by shape. Codegen mirrors the rule: emit `#[inline]` for concrete generated fns; emit `/// _Simple._` for generic generated fns and for generated trait-method docs whose impls are always simple.

**Why:** The "≤ 1 fn call" budget was too tight because it counted calls to other `#[inline]` fns, even though those calls collapse during inlining. Without recursion, simple wrappers around simple wrappers were missed. The blanket "exclude generic / blanket-impl" exclusion missed an entire class of simple fns whose bodies LLVM can absolutely inline. The doc tag exists because `#[inline]` on a generic fn is redundant (monomorphization already makes the body cross-crate-available), so there was no source-visible marker for "this generic call is free in the budget rule" until now.

**How to apply:** When writing or reviewing a new fn, run the recursive simple test on it; if it passes, add the marker that matches its shape. When writing a trait method, ask "must every conforming impl be simple?" — if yes (because every impl is generated by a derive in this workspace, or because the contract demands it), tag the declaration. Do not tag a trait method whose contract admits non-simple impls (e.g. `Object::read_property`, which has a `match`).

**Escalated?** AGENTS.md, doc-convention, agent:review-findings, agent:self-review

### 2026-05-06 — code-style — extract body of `impl Into<T>` shells into a nested `fn inner`

**What happened:** `ObjectTree::rename(impl Into<String>)` (`quartzite-runtime/src/object_tree.rs:318`) ships its full ~30-line body in one copy per concrete `T` callers pass — `&str`, `String`, `&String`, `Cow<'_, str>`, etc. The same pattern repeats in `ObjectFactory::register`, `Timer::named`, `ObjectBase::named`. Lifting the body out is the standard-library idiom (see `Path::new`, `PathBuf::from`, `OsStr::new`), but a sibling `*_inner` impl method pollutes the type namespace.

**Rule:** When a public fn takes `impl Into<T>` / `impl AsRef<T>` / `impl ToString` and the body is > 3 lines, place the body in a nested `fn inner(...)` defined inside the outer fn (NOT as a sibling method); the outer fn carries the `_Simple._` doc tag and only does the conversion + forwards to `inner(...)`. Promote to a sibling private fn only when a second caller actually needs it. Skip when the type parameter is used throughout the body (genuine generic logic — e.g. `F: FnOnce(...)` invoked inside the body).

**Why:** Only the trivial conversion shell is monomorphized; the body ships once. Nesting keeps the helper out of the impl namespace, makes it impossible to misuse from elsewhere, and matches the `std` idiom. A nested `fn` cannot capture `self`; pass it explicitly as `this: &mut Self`.

**How to apply:** When adding or reviewing a public fn with a conversion-style generic param, count the body lines; if > 3, extract into a nested `fn inner(...)`. Reviewer: flag missing extraction as `nit` (binary-size cost may be accepted with rationale).

**Escalated?** AGENTS.md, agent:review-findings, agent:self-review

### 2026-05-06 — process — do not object to "breaking API change" on an unpublished crate

**What happened:** Code review finding #2 (dynamic `SingleShot` slot leak) was objected to with "breaking API change" as the reason. The user rejected this immediately — the project has not been published to crates.io and AGENTS.md explicitly allows freely changing the public API before the first `cargo publish`.

**Rule:** "Breaking API change" is not a valid objection reason on this codebase until after the first `cargo publish`. API can be freely renamed, removed, or restructured. When a finding is technically fixable but would require an API change, implement the change rather than deferring.

**How to apply:** Before objecting to any finding citing API stability, check AGENTS.md § "API Stability". If the crate is unpublished, remove that objection and implement the fix.

**Escalated?** AGENTS.md

### 2026-05-03 — process — /interview has no "too small for a spec" off-ramp; flag and ask instead of silently switching to implementation

**What happened:** `/interview errors should iml Error` was started. Mid-interview the user redirected: "ok, add the two impl blocks under the same cfg." Instead of completing the interview (producing a spec) or explicitly flagging the pivot, execution switched silently to direct implementation. The `/interview` skill produced code changes rather than a spec.

**Rule:** When a task arrives via `/interview` (or `/task`) but turns out to be trivially small during the interview, do not silently abandon the spec and implement. Flag explicitly: "This is small enough to implement directly — want me to skip the spec and just make the change?" Wait for the user to confirm the mode switch before writing any code.

**How to apply:** If the interview answers reveal the entire change is < ~20 lines with no design decisions, pause and offer to skip the spec rather than silently pivoting. The user stays in control of when the workflow gets bypassed.

**Escalated?** skill:interview

### 2026-05-03 — process — commit and push without waiting for explicit approval when inside a /task workflow

**What happened:** All rename changes were complete, verified, and ready to push, but no commit or push was made until the user explicitly asked. The stated reason was the global system instruction "only create commits when requested." The user clarified that this instruction does not override the /task workflow, which already authorizes commits at Steps 8 and 12.

**Rule:** Within a `/task` workflow, committing after each subtask (Step 8) and committing + pushing at Step 12 are part of the authorized workflow — no additional user prompt is needed. The global "ask before committing" instruction applies to ad-hoc work outside a task, not to steps explicitly prescribed by the task skill.

**How to apply:** If currently executing a `/task` (progress file exists or task was just completed), follow the step instructions for committing and pushing. Only pause to ask if something is ambiguous or risky beyond what the task step covers.

**Escalated?** skill:task

### 2026-05-03 — process — include ai-docs/learnings.md in the PR commit when it changes during the task

**What happened:** `ai-docs/learnings.md` was modified while working on a PR but not staged or committed — leaving learnings outside the PR diff and not reviewed alongside the code change that prompted them.

**Rule:** If `ai-docs/learnings.md` is modified during work on a PR (Step 8–11 or a follow-up push), stage and commit it together with the related code changes. Learnings are part of the task deliverable and should be visible in the PR for review.

**How to apply:** Before any `git commit` during a PR task, check `git diff --name-only ai-docs/learnings.md`. If it shows changes, include the file in the commit being staged.

**Escalated?** AGENTS.md

### 2026-05-03 — process — update PR title and body after commits that change public API or scope

**What happened:** After renaming `emit_unchecked`/`emit_checked` → `emit`/`emit_unless_blocked`, the PR title and body still referenced the old names. The user had to prompt explicitly to update them.

**Rule:** After any `git push` that changes public API names, scope, or acceptance criteria, immediately update the PR title and body with `gh pr edit` before reporting done. The PR description is part of the deliverable — it must stay in sync with the actual implementation.

**How to apply:** At the end of Step 12 (or any follow-up push), check whether the PR title/body mentions any symbol, AC, or scope item that the new commits changed. If yes, run `gh pr edit --title "..." --body "..."` before posting the PR URL to the user.

**Escalated?** skill:task

### 2026-05-03 — code-style — `_unchecked` is reserved for `unsafe` fns; default name is the safe variant

**What happened:** Renamed `Signal::emit` → `Signal::emit_unchecked` and added `Signal::emit_checked` (which consults the `blocked` flag). Neither function is `unsafe`. This conflicts with `std` ecosystem convention: `_unchecked` is reserved for `unsafe` fns whose invariants the caller must uphold to avoid UB (e.g. `slice::get_unchecked`, `str::from_utf8_unchecked`). The natural unsuffixed name should be the safe, ergonomic default — using `_unchecked` for "skips an unrelated runtime check" misleads readers and reviewers and removes the ergonomic reward for the safe path.

**Rule:** API naming follows `std`:
- Default `do_something()` is safe/checked. `do_something_unchecked()` is the `unsafe` companion (must include a `# Safety` doc section listing invariants).
- If the safe default panics, add `try_do_something()` (preferred) or `do_something_checked()` returning `Result`/`Option`.
- For non-safety "with/without runtime behavior X" variants, pick descriptive names — do not co-opt `_unchecked`/`_checked`.

**Escalated?** AGENTS.md, agent:review-findings, agent:self-review

### 2026-05-03 — process — breaking public API changes are allowed before first crates.io release

**What happened:** Suggested keeping `Signal::emit` for backward compatibility. User clarified: the project has no downstream clients yet; API can be freely broken until the first release to crates.io.

**Rule:** Do not add backward-compat shims, deprecation layers, or keep old names "for compatibility" while the crate has not been published to crates.io. Rename, remove, or restructure public API freely. Note the release milestone in decisions when it matters.

**Escalated?** AGENTS.md

### 2026-05-02 — code-style — let chains are allowed and formattable in edition 2024

**What happened:** During the macros task, rustfmt errored on a let chain with "let chains are only allowed in Rust 2024 or later". The workspace uses `edition = "2024"`, Rust 1.95, and rustfmt 1.9.0 — all of which support let chains. The error was caused by running rustfmt without `--edition 2024` explicitly, or against a stale binary. The response was to replace let chains wholesale with match expressions as a blanket rule.

**Rule:** Let chains (`if let A = x && let B = y { ... }`) are valid in this codebase. Do not avoid them. Run `rustfmt` via `cargo fmt` (which picks up the workspace edition automatically) rather than invoking `rustfmt <file>` directly.

**Escalated?** AGENTS.md

### 2026-05-02 — process — do not touch IDE files unless explicitly asked

**What happened:** `.idea/quartzite.iml` had an uncommitted modification. Without being asked, it was added to `.gitignore` and removed from tracking.

**Rule:** Never add, remove, modify, or `.gitignore` IDE files (`.idea/`, `*.iml`, `.vscode/`, etc.) unless the user explicitly asks. Treat them as the user's domain.

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — "submit to PR" means push to remote, not merge

**What happened:** User said "submit to pr". Interpreted as merging the PR via `gh pr merge`. User meant pushing the local commits to the remote branch so they appear in the open PR.

**Rule:** "Submit to PR" (and similar: "push to PR", "add to PR") means `git push` the branch to remote. It does not mean merging. Only merge when the user explicitly says "merge" or "merge the PR".

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — "wtf" signals that the previous action was wrong

**What happened:** User said "add ide files". Interpreted as adding IDE files to `.gitignore`. User meant commit and track them. User responded "wtf?" to signal the action was wrong.

**Rule:** "wtf" (and similar expressions of surprise/frustration) means the last action was the opposite of what the user wanted. Stop immediately, ask what went wrong, and do not proceed until the intent is clarified.

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — never use git reset --hard; use soft reset, stash, cherry-pick, or backup branch

**What happened:** `git reset --hard origin/master` was used to move commits off local master to a feature branch. This discarded uncommitted changes to `ai-docs/learnings.md` that had not been staged.

**Rule:** Never use `git reset --hard`. Use one of these instead:
- `git reset --soft HEAD~N` — moves commits back to staged, preserves working tree
- `git stash` — saves uncommitted changes before switching branches
- `git cherry-pick` — moves specific commits to another branch
- Backup branch — `git checkout -b backup/...` before any destructive operation

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — always create a feature branch before committing; never commit directly to master

**What happened:** When the user said "submit PR", commits were already on local master. Instead of creating a feature branch first, `git push` was run directly against master — pushing the commits to origin/master. `master` is branch-protected (no force push), so the commits could not be removed after the fact and a proper PR became impossible.

**Rule:** When work is intended for a PR, create a feature branch (`git checkout -b feat/...`) *before* making any commits. Never commit to local master with the intention of later turning it into a PR.

Recovery — if commits were accidentally made on local master and not yet pushed (full procedure in `AGENTS.md`):
1. `git stash` — save any uncommitted changes
2. `git checkout -b feat/...` — branch off from current HEAD (carries the commits)
3. `git checkout master && git reset --soft origin/master && git restore --staged .` — soft-rewind local master to remote state without discarding work
4. Push the feature branch and open the PR from it; pop the stash on the feature branch if needed
5. **Never push master** — not even as an intermediate step. **Never use `git reset --hard`** — see the dedicated rule above.

Before any `git push`: run `git branch --show-current` and confirm it is **not** `master`. If it is master — stop, do not push, apply the recovery procedure above.

If "submit PR" is requested and commits are already pushed to origin/master: stop and tell the user — there is no recovery without a force push, which branch protection may block.

**Escalated?** AGENTS.md

### 2026-05-02 — process — create feature branch before committing at the start of Step 8

**What happened:** The auto-connection task completed all implementation steps on the working tree without ever committing. Only after the user asked "why didn't you create a PR?" was the branch created. The changes had to be recovered by checking out a feature branch from the unstaged state.

**Rule:** At the start of Step 8 (Implementation), immediately create a feature branch (`git checkout -b feat/...`) before writing any code. Record the branch name in the progress file. Do not wait until after self-review to create the branch.

**Escalated?** skill:task, hook (PreToolUse on `git commit` blocks master)

### 2026-05-02 — testing — any sufficiently large file requires unit tests

**What happened:** Three codegen files (`object/codegen.rs`, `object_impl/codegen.rs`, `meta_enum/codegen.rs`) were written without `#[cfg(test)]` modules. Gaps were caught in review and by the user. The original rule was codegen-specific, but the user generalised it: any file with substantial logic needs tests.

**Rule:** Any file with ~50+ lines of non-trivial code must have a `#[cfg(test)] mod tests` block. This applies equally to codegen, parse, util, and any other module — not just files named `codegen.rs`.

**Escalated?** AGENTS.md

### 2026-05-02 — process — propagate skill/agent fixes to all related files in the same operation

**What happened:** A fix to `self-review.md` was applied in isolation; `codebase-review.md` was only updated after the user pointed it out. Similarly, `/task` and `/task-issue` fixes were done together, but the code-review family was handled piecemeal.

**Rule:** When fixing a skill or agent, immediately propagate the change to all files in the same sync group before reporting done. Two sync groups:
- **Task group:** `skills/task/SKILL.md` ↔ `skills/task-issue/SKILL.md`
- **Review group:** `skills/code-review/SKILL.md` (workflow orchestrator) ↔ `agents/review-findings.md` (findings producer) ↔ `agents/self-review.md` (fix validator)

Note: `code-review` is a **skill** (user-facing workflow); `review-findings` and `self-review` are **agents** spawned by it. Do not refer to any of these as "code-review agent" — that conflates the skill with an agent. (A `diff-review` agent existed historically but was removed as orphan; do not reintroduce it without wiring it into a skill.)

**Escalated?** AGENTS.md, memory

### 2026-05-02 — process — self-review must not re-run cargo fmt or cargo clippy

**What happened:** The self-review agent checked `cargo fmt -- --check` and raised REJECT findings for formatting drift, even though both `cargo fmt` and `cargo clippy -- -D warnings` are already mandated after every subtask during Implementation (Step 8 of /task and /task-issue). This caused a spurious round-trip.

**Rule:** Self-review must not run or re-check `cargo fmt`, `cargo clippy`, `cargo build`/`check`, or `cargo test`. These are all guaranteed by the Implementation and Verify steps before self-review runs. Self-review scope: spec conformance, design conformance, test coverage, safety/correctness, style (Rust file conventions, allow attributes) — not build tooling.

**Escalated?** agent:self-review, agent:review-findings

### 2026-05-02 — process — propagate rule exemptions to agent/skill files in same task

**What happened:** When adding `quartzite-examples` exemptions to `AGENTS.md` (no `#![deny(missing_docs)]`, no `#[cfg(test)]`), the corresponding checks in `.claude/agents/self-review.md` and `.claude/agents/review-findings.md` were not updated. Future reviews would have incorrectly flagged the examples crate.

**Rule:** When a rule exemption is added to `AGENTS.md`, immediately propagate it to every agent/skill/settings file that enforces that rule. Check with `grep` across `.claude/agents/` and `.claude/skills/` before closing the task.

**Escalated?** AGENTS.md

### 2026-05-02 — process — check current branch before committing, not only before pushing

**What happened:** For the `docs/learnings-and-skill-fix` branch, commits were made directly to local master without checking `git branch --show-current` first. The error was caught at push time (branch protection rejected the push), not at commit time. The rule in AGENTS.md mentions checking before `git push`, but the correct mental model is: verify branch before `git commit`.

**Rule:** Run `git branch --show-current` and confirm it is **not** `master` before any `git commit` that is intended for a PR. A pre-push check is a last resort, not the primary safeguard. The commit should never happen on master — the push check only exists as a final gate.

**Escalated?** hook, skill:task

### 2026-05-02 — process — run cargo fmt --all after every code change, including post-self-review fixes

**What happened:** `cargo fmt --all -- --check` was run once during Step 9 (Verify). A self-review finding then triggered a code fix (Step 11). The fix was committed and pushed without re-running `cargo fmt --all`. CI failed on the formatting drift introduced by that fix.

**Rule:** Run `cargo fmt --all` (and re-check with `cargo fmt --all -- --check`) after *every* code change — including fixes made after self-review. The verify step (Step 9) is not a one-time gate; it must be re-run after any subsequent edit before committing. Never commit without a clean `cargo fmt --all -- --check` immediately before the commit.

**Escalated?** hook

### 2026-05-03 — code-style — use stdlib methods instead of explicit branching for comparisons

**What happened:** In `Rect::united` / `RectF::united`, explicit `if` branches were written to compute min/max of edge coordinates. The review comment pointed out `.min()`/`.max()` should be used instead.

**Rule:** Use `.min()`, `.max()`, `.clamp()`, and other stdlib methods instead of `if`/`match` branching for simple value comparisons. This applies equally to all numeric types (`i32`, `f32`, etc.). Explicit branches are harder to read and error-prone.

**Escalated?** AGENTS.md

### 2026-05-03 — architecture — do not reference other frameworks as justification for design choices

**What happened:** Design decisions for geometry types were described as "consistent with Qt's Qt::NoButton = 0" in analysis. The user explicitly said not to copy or reference other frameworks; we are writing a Rust framework from scratch using language strengths.

**Rule:** Never cite other frameworks (Qt, GTK, WinForms, etc.) as justification for design choices. Design based on Rust idioms, crate ecosystem norms, and explicit reasoning about the problem. "Other library does X" is not a valid argument.

**Escalated?** AGENTS.md

### 2026-05-03 — process — delete progress file immediately on self-review APPROVE, before Step 12

**What happened:** Self-review Round 2 returned APPROVE. Proceeded directly to Step 12 (finalize INDEX.md, commit, PR) without deleting the progress file first. The file was left as an untracked artifact after the PR was created.

**Rule:** On self-review APPROVE, delete `.progress.md` as the *first action* before starting Step 12. The task skill is explicit: "On APPROVE: delete `.progress.md` → proceed to Step 12." The file is transient handoff state; it must not outlive the task.

**Escalated?** skill:task

### 2026-05-03 — process — do not ask about backward compatibility; AGENTS.md already prohibits compat shims

**What happened:** During an interview, asked "should `#[object_meta]` be kept as a deprecated no-op alias (soft removal) or fully deleted?" — despite AGENTS.md explicitly stating "The project has not yet been published to crates.io and has no downstream clients. Public API may be freely renamed, removed, or restructured without backward-compat shims or deprecation layers." The same lesson was already in learnings.md (2026-05-03 "breaking public API changes are allowed") and escalated to AGENTS.md.

**Rule:** Never ask the user about backward compatibility, deprecation layers, or keeping old names for compatibility. Read AGENTS.md before formulating interview questions. If a rule is already documented there, apply it silently — do not re-ask.

**How to apply:** Before asking any interview question, check whether AGENTS.md already answers it. Questions about API stability, compat shims, deprecation, and release timing are all answered there.

**Escalated?** skill:interview

### 2026-05-02 — process — verify relative markdown links before committing

**What happened:** Files generated in `ai-docs/deferred/` used `../../plans/done/` as the relative path to `ai-docs/plans/done/`. The correct path is `../plans/done/` — one level up from `ai-docs/deferred/` reaches `ai-docs/`, then `plans/done/` is a sibling of `deferred/`. GitHub rendered the broken paths as `/plans/done/` (repo root), which does not exist.

**Rule:** After generating files with relative links, verify at least one link manually: trace the path on disk (`realpath` or mental directory traversal) before committing. From `ai-docs/deferred/file.md`, one `..` reaches `ai-docs/`; two `../..` reaches the repo root.

**Escalated?** AGENTS.md

### 2026-05-05 — code-style — `clippy::doc_markdown` allowlist scope + heuristic limits

**What happened:** Initial doc-convention seed populated `clippy.toml`'s `doc-valid-idents` with ~60 entries: every project type (`MouseEvent`, `ObjectBase`, `BitFlags`, …), third-party types (`IndexMap`, `RwLock`), and `no_std`. The reasoning was "avoid noise from `clippy::doc_markdown` false positives during the audit." On review the user pointed out this was the wrong default — Rust identifiers in prose should always be backticked; the allowlist is for *non-code* tokens only. Shrinking to just `GPU` surfaced only **3 violations** workspace-wide. **A follow-up empirical test then showed that even `GPU` was dead code:** the `doc_markdown` heuristic flags `CamelCase` identifiers (`HelloWorld` → warning) but does NOT flag pure all-caps acronyms regardless of length (`GPU`, `REALLYLONGACRONYM` → no warning). The entire `clippy.toml` was deleted as a result.

**Rule:** `doc-valid-idents` entries must be genuine non-code tokens **AND** must be `CamelCase` / mixed-case names (the only shape the heuristic catches). **Never add Rust type names, function names, module names, third-party crate types, or build-config tokens like `no_std`** to the allowlist — backtick them inline. **Never add pure all-caps acronyms** like `GPU`, `JSON`, `URL` — the heuristic already ignores them. Realistic candidates are `CamelCase` brand names or proper nouns the project genuinely uses in prose without intending them as code (e.g. a product name).

**How to apply:** Default response to a `doc_markdown` warning is "add backticks", not "allowlist the name". Before adding any entry, verify the token actually fires the lint by removing it temporarily and re-running clippy with `--all-targets`. The workspace currently ships **no `clippy.toml`** — only resurrect it if a real non-code `CamelCase` token surfaces.

**Escalated?** doc-convention (`ai-docs/doc-convention.md` *Linking and code references* + *Lints* sections)

### 2026-05-05 — architecture — prefer AtomicBool + safe OnceLock over AtomicPtr + unsafe for process-global accessors

**What happened:** The `global_tree` process-global accessor used `static TREE_PTR: AtomicPtr<Mutex<ObjectTree>>` with an `unsafe` dereference inside `try_with_tree`. The user pointed out that the `AtomicPtr` + `unsafe` design is unnecessary: since `APP: OnceLock<Arc<ApplicationInner>>` already holds the tree for the process lifetime, a simple `AtomicBool TREE_LIVE` flag (set by `Application::new`, cleared by `Drop`) is sufficient. `try_with_tree` then checks the bool, calls `APP.get()?`, and locks safely — no raw pointers, no unsafe blocks.

**Rule:** When implementing a process-global accessor that tracks whether a singleton is "live", prefer `AtomicBool` (or a similar safe primitive) over `AtomicPtr` to raw memory. Reach for `unsafe` only when there is no safe alternative. An `OnceLock` or `Weak<T>` already in scope can often provide the pointer without raw pointer manipulation.

**How to apply:** Before writing an `AtomicPtr`-based global, ask "can I express the same semantic with a `bool` flag plus an already-existing `OnceLock`?" If yes, use that. Reserve `AtomicPtr` for cases where the pointee's lifetime genuinely cannot be tracked through existing safe constructs.

**Escalated?** AGENTS.md

### 2026-05-05 — process — context-reset and self-review agents must check `#[inline]` on every simple new fn

**What happened:** The context-reset subagent implementing Tasks 3–11 of the timer-object task wrote `impl AsObject` and `impl Object` methods (`object_base`, `object_base_mut`, `as_any`, `as_any_mut`, `meta_object`, `invoke_method`, `connect_signal`) without `#[inline]`. The self-review agent's Round 1 pass also missed the gap, only flagging it in Round 2. The user had to point it out explicitly before the PR commit.

**Rule:** Both the context-reset implementing agent and the self-review agent must check `#[inline]` on every new simple, non-generic function — not only on named groups like "getters" or "Default impls". Every `AsObject`/`Object` impl method, every trivial wrapper, every constructor that is just a struct literal, must be reviewed for this annotation before the task is reported done.

**Why:** The AGENTS.md `#[inline]` rule is unambiguous and applies broadly. Context-reset agents must apply it at write time. Self-review agents must validate it independently on every new public fn.

**How to apply:** At implementation time, add `#[inline]` immediately before writing the body of any simple fn. At self-review time, scan every `fn` in the diff for missing `#[inline]` before reporting APPROVE — not just fn groups the reviewer already knows about.

**Escalated?** agent:self-review

### 2026-05-05 — architecture — use existing derive macros for `AsObject`/`Object`; do not write manual impls

**What happened:** The context-reset subagent implementing the Timer type wrote manual `impl AsObject for Timer` and `impl Object for Timer` instead of using `#[derive(Extend, Object)]` from `quartzite-macros`. The design document explicitly specified the derive approach. Adding `quartzite-macros` as a dependency of `quartzite-runtime` has no circular dependency (quartzite-macros only depends on `syn`/`quote`/`proc-macro2`), so there was no obstacle. The manual impl caused a 2-round review cycle to surface the `#[inline]` gap the macro would have generated automatically.

**Rule:** When a type in `quartzite-runtime` (or any crate with access to `quartzite-macros`) needs to implement `AsObject` and `Object`, use `#[derive(Extend, Object)]` + `#[object_impl]`. Do not hand-write the impls. The derive macros generate correct `#[inline]` annotations, property dispatch, and MetaObject registration automatically.

**Why:** The derive macros exist precisely to eliminate boilerplate. Using them ensures consistency, correctness, and automatic compliance with all codegen conventions. Rolling a manual impl is reinventing the wheel and drifts from the generated pattern.

**How to apply:** When adding `#[derive(Extend, Object)]` to a crate that doesn't yet depend on `quartzite-macros`, add the dep first (`quartzite-macros = { path = "../quartzite-macros" }`). Proc-macro crates are never circular.

**Escalated?** no

### 2026-05-05 — code-style — use `.ok()?` not `.unwrap()` on `Mutex::lock` in library code

**What happened:** `try_with_tree` used `mutex.lock().unwrap()`, which panics on a poisoned mutex. Since AGENTS.md mandates non-panicking APIs for libraries, and mutex poisoning (another thread panicking while holding the lock) is not a broken global invariant from the caller's perspective, returning `None` is the correct behaviour. `.lock().ok()?` achieves this with no additional code.

**Rule:** In library code, use `mutex.lock().ok()?` (or `mutex.lock().unwrap_or_else(|e| e.into_inner())` when you want to recover the inner value) rather than `.unwrap()`. Panicking on mutex poisoning is not appropriate for a library — callers should decide how to handle a "tree unavailable" result.

**How to apply:** Any time you write `something.lock().unwrap()` in a function that returns `Option` or `Result`, replace with `.lock().ok()?`. Reserve `.unwrap()` for cases where poisoning truly indicates an unrecoverable program invariant failure.

**Escalated?** AGENTS.md

### 2026-05-05 — process — never ask whether a library API should panic for an avoidable error

**What happened:** During interview for #55 (parent/children accessors), asked "should `parent()` / `children()` panic or return a default when called outside an Application scope?" — AGENTS.md already answers this: "Prefer non-panicking APIs for libraries; panicking is acceptable only when a fundamental invariant is broken." Being outside an Application scope is a recoverable condition, not a broken global invariant, so `None`/empty is the correct answer by rule.

**Rule:** Never ask the user whether a library function should panic for an avoidable error condition. Read AGENTS.md first — the non-panicking default is already mandated. Only ask about panic behavior if the scenario involves a genuinely broken global invariant (e.g., internal data structure corruption, double-free).

**How to apply:** Before formulating interview questions, check whether AGENTS.md already resolves the question. "Should X panic or return None/Err?" is almost always answered by the non-panicking library API rule — apply it silently.

**Escalated?** skill:interview

### 2026-05-05 — process — filter to unresolved PR review threads before reading comments

**What happened:** When the user said "I've commented gh pr", all inline comments were fetched and read — including ones already resolved by the reviewer in a prior session. Time was wasted re-reading and re-resolving already-closed threads.

**Rule:** Always fetch PR review threads filtered to `isResolved: false` before reading comments. Use the GraphQL query:
```
reviewThreads(first: 20) { nodes { id isResolved comments(first:1) { nodes { databaseId body } } } }
```
then filter `isResolved == false` before reading any comment bodies.

**How to apply:** Start every "I've commented gh pr" workflow with the GraphQL unresolved-threads query, not the REST `/pulls/{N}/comments` endpoint (which returns all comments regardless of resolution state).

**Escalated?** no

### 2026-05-05 — process — always run the PR body check after every push, even if no edit seems needed

**What happened:** After pushing AGENTS.md + learnings.md changes to the open PR branch, the PR body check (`gh pr view <N>`) was skipped on the grounds that instruction-only commits can't affect code claims. The rule is unconditional: re-read first, then decide. Reasoning your way out of the check is the failure mode the rule prevents.

**Rule:** After every push to a branch with an open PR, always run `gh pr view <N>` and read the body. Only then decide whether an edit is needed. Never skip the read — only skip the edit if the body is still accurate.

**How to apply:** Post-push checklist: `gh pr view <N> --json title,body`. If the body matches reality → done. If not → `gh pr edit`. The cost is one command; the benefit is catching invisible drift.

**Escalated?** AGENTS.md, skill:task, hook (PostToolUse Bash pr-sync reminder in `.claude/settings.json`)

### 2026-05-05 — process — resolve fixed review comments; leave objected ones for the reviewer

**What happened:** After applying fixes and posting an objection reply to a PR review comment, neither type of comment was resolved on GitHub. The user clarified the correct rule: resolve only comments that were fixed; leave comments where an objection was posted so the reviewer can decide whether to accept the objection.

**Rule:** After pushing fixes to a PR:
- Comments addressed by a code fix → resolve on GitHub (`gh api … -X PUT … {"resolved": true}` or via the UI).
- Comments where a reply was posted explaining why no change was made (objection) → leave unresolved; it is the reviewer's call to accept or push back.

**How to apply:** After any `git push` that addresses review feedback, iterate over the closed-out comments and resolve only the ones that have a corresponding code change. Objection replies are the reviewer's domain to close.

**Escalated?** AGENTS.md

### 2026-05-05 — code-style — use thiserror for error types; apply undocumented_unsafe_blocks to every crate

**What happened:** `TreeAccessError` was initially hand-rolled with manual `Display` / `std::error::Error` impls. PR review requested `thiserror = "2"` be added. Separately, `#![warn(clippy::undocumented_unsafe_blocks)]` was added only to `quartzite-runtime` when first introduced; review comment pointed out it should be in every crate's `lib.rs`.

**Rule:** Use `thiserror` for any new error enum/struct in this workspace — avoids boilerplate `Display` / `Error` impls. Add `#![warn(clippy::undocumented_unsafe_blocks)]` to every crate `lib.rs` (next to the other clippy attributes), not just the crate where unsafe was first introduced.

**How to apply:** When adding a new crate-level lint attribute, immediately propagate it to all other crate `lib.rs` files in the same PR. When defining an error type, reach for `thiserror` first.

**Escalated?** AGENTS.md

### 2026-05-05 — process — keep PR description in sync after every push to an open PR

**What happened:** While iterating on PR #83 (doc-convention) after it was already open, we landed two follow-up commits — first tightening the "backtick every Rust identifier" rule and shrinking `clippy.toml` to one entry, then deleting `clippy.toml` entirely after an empirical test showed the heuristic ignores all-caps tokens. The original PR body still claimed "New workspace-root `clippy.toml` with a ~60-entry `doc-valid-idents` allowlist…" and the AC6 test-plan line still said "seeded; no growth needed". Neither was true after the follow-ups. The PR description was not synced until the user explicitly asked. The `/task` skill *does* spell out this rule in Step 11 ("If the fixes changed any public API name, scope, or AC referenced in the PR title/body (and the PR is already open), run `gh pr edit --title ... --body ...` to bring the PR description in sync before pushing"), but the rule applies to *any* push that invalidates a claim in the body — not only Step 11 review-fix commits.

**Rule:** **After every push to a branch with an open PR, re-read the PR body (`gh pr view <N>`) and ask "is anything here now wrong?".** If yes, sync via `gh pr edit <N> --title "..." --body "..."` *before* the next action. Sync triggers include: scope items added or dropped; renames or removals of files / types / fns named in the body; AC checkbox status flips; numbers cited in the body that drifted (test count, lint count, allowlist size, file count, etc.). Routine commits — typos, formatting, refactors strictly within already-described scope — do not need a body edit. The **upstream tracking issue's** title and body are the user's original problem statement and **must not be rewritten**; communicate scope changes via an issue comment instead, and rely on the PR's `Closes #N` line to surface the merging PR via GitHub's auto-link.

**How to apply:** Add a "re-read PR body" step to your post-push checklist whenever the open-PR-on-branch state holds. Cheapest shape: `gh pr view <N> --json title,body | rg -i '<thing-you-just-changed>'` — if any hit, the body needs an edit. The cost of an extra `gh pr edit` is minutes; the cost of a reviewer reading a body that contradicts the diff is wasted reviewer trust.

**Escalated?** AGENTS.md, skill:task, hook (PostToolUse Bash pr-sync reminder in `.claude/settings.json`)

### 2026-05-05 — process — `gh pr view` not needed immediately after `gh pr create`

**What happened:** After `gh pr create` for PR #85, ran `gh pr view 85` to "apply the new unconditional read rule". The body returned was exactly what was just authored a second earlier — the read was wasted work. The "PR body sync after every push" rule treats `gh pr create` as if it were a subsequent push that might have invalidated the body, but in fact the body and the push are authored together: there is nothing to discover.

**Rule:** The "PR body sync after every push" rule does not apply to the push that immediately precedes `gh pr create`. The body at that moment is what you just authored; nothing to discover. The rule starts firing on the *next* push to the branch.

**How to apply:** After `gh pr create` returns the URL, stop — do not re-read the body until at least one further `git push` happens. From the *second* push onward, the unconditional read rule applies.

**Escalated?** AGENTS.md

### 2026-05-05 — process — backward-compat question asked again despite skill:interview escalation

**What happened:** During `/interview` for issue #36, asked "should the current `Timer` construction/usage API stay roughly the same (just gaining object-tree integration), or is a full redesign expected?" — i.e., a backward-compat framing. The rule was already escalated to `skill:interview` on 2026-05-03 ("do not ask about backward compatibility; AGENTS.md already prohibits compat shims"). User rightly pushed back again.

**Rule:** Never frame an interview question around backward compatibility, keeping old APIs, or preserving existing behavior "for users". AGENTS.md is clear: no crates.io release, no downstream clients, free to rename/remove/restructure. Apply this silently before any question round.

**How to apply:** Before every interview round, re-read AGENTS.md § API Stability. If a candidate question touches compat, deprecation, or "keeping old X", discard it and apply the rule silently.

**Escalated?** skill:interview

### 2026-05-05 — tooling — use `0.x` version format for 0.x.y deps, not bare `0`

**What happened:** Added `tracing = "0"` and `itertools = "0"` to Cargo.toml. AGENTS.md rule is "use `0.x` for `0.x.y` versions — never pin the patch." The correct forms are `tracing = "0.1"` (tracing is 0.1.x) and `itertools = "0.14"` (itertools is 0.14.x). Bare `"0"` is overly broad; it would accept any 0.x.y release including incompatible minor versions.

**Rule:** Check the resolved minor version (from `cargo update` output or crates.io) and use `0.x` — e.g. `"0.1"`, `"0.14"`. `"0"` is wrong because Cargo treats 0.x as breaking per minor version; `"0.x"` pins to that minor line.

**How to apply:** When adding a new 0.x.y dep, look at the version that `cargo update` resolves (e.g., `tracing v0.1.44`) and write `"0.1"`, not `"0"`. Same applies to `{ version = "0.x", ... }` inline tables.

**Escalated?** no

### 2026-05-05 — process — switch to feature branch BEFORE editing files in `/improve` (and similar skills)

**What happened:** During `/improve`, all instruction-file edits were applied while on `master`. Only at commit time was the branch-switch made reactively (`git checkout -b chore/...`). AGENTS.md says "create a feature branch before any commits" — that was technically respected (no commits on master), but the spirit (don't accumulate work on master) was broken. `/task` already gates this at Step 8; `/improve` had no equivalent gate.

**Rule:** Skills that produce commits intended for a PR (currently: `/improve`; future: any new skill in the same shape) must check `git branch --show-current` and create a feature branch *before any file edit*, not just before commit.

**How to apply:** In `/improve` Step 5 (Apply), the first action — before any Edit/Write — is the branch check. If on `master`, `git checkout -b chore/YYYY-MM-DD-improve-<short-name>` carries the working tree over. Edit only after the branch is in place.

**Escalated?** AGENTS.md, agent:self-improve

### 2026-05-06 — code-style — never use .expect() on mutex locks or condvar waits in production code

**What happened:** All `.lock().expect("... poisoned")` and `condvar.wait().expect("...")` calls in `ThreadDriver`, `AppDriver`, and `PoolDriver` were written as panicking. Reviewer flagged all three sites asking why panicking behavior was there and whether it was avoidable.

**Rule:** Mutex poisoning in library code is recoverable. Use `.lock().unwrap_or_else(|e| e.into_inner())` (or `.ok()?` in Result-returning fns) instead of `.expect("... poisoned")`. The `AGENTS.md` library safety idioms section already states this explicitly. Also remove any `# Panics` doc sections from methods that no longer panic after the fix.

**How to apply:** On any new driver, scheduler, or shared-state type: whenever a `Mutex::lock()` or `Condvar::wait()` call appears in production code (i.e., not in tests), use `unwrap_or_else(|e| e.into_inner())` by default. Reserve `.expect("reason")` only for cases where poisoning genuinely means an unrecoverable invariant violation (document why in the reason string).

**Escalated?** hook (PostToolUse Edit/Write panic-gate in `.claude/settings.json`)

### 2026-05-06 — code-style — `.expect()` on mutex/condvar/Option is still a panic; checklist must catch it explicitly

**What happened:** `timer.rs` was implemented with `.expect("... mutex poisoned")` on every `Mutex::lock()`, `Condvar::wait()`, and `BinaryHeap::peek()`/`pop()` call in `ThreadDriver`, `AppDriver`, and `PoolDriver`. Two rounds of self-review (Rounds 2 and 3) both approved without catching these. The user had to flag all five sites via PR comments.

**Why the gate broke — implementor side:** AGENTS.md § *Library safety idioms* says "use `mutex.lock().ok()?` or `.unwrap_or_else(|e| e.into_inner())`". The implementor read the *general* test rule ("no `unwrap()` in production code; `expect("reason")` preferred") and concluded that spelling out a reason string was sufficient. The mutex-specific rule was applied silently only to the `Mutex<Signal>` from `quartzite-core` (which was correctly handled via `ok()?`) but not to the driver-owned mutexes.

**Why the gate broke — self-review agent side:** The reviewer checklist item was worded as "No `unwrap()` in production code (only in `.expect("reason")` form with justification)." Because `.expect(...)` is literally a different method name, the grep mental model (`grep unwrap`) missed it entirely. The agent never checked `.expect()` calls for panicking soundness — only for the presence of a justification string.

**Rule:** Both implementor and self-review agent must treat `.unwrap()`, `.expect()`, and `panic!()` as the same category — *unconditional panics* — and apply the same scrutiny to all three. For every occurrence in production code (outside `#[cfg(test)]`), ask: "Is there a non-panicking form?"

Concrete substitutions:
- `mutex.lock().expect(...)` → `mutex.lock().unwrap_or_else(|e| e.into_inner())`
- `condvar.wait(g).expect(...)` → `condvar.wait(g).unwrap_or_else(|e| e.into_inner())`
- `condvar.wait_timeout(g, d).expect(...)` → `condvar.wait_timeout(g, d).unwrap_or_else(|e| e.into_inner())`
- `option.expect("logically guaranteed")` → `let Some(x) = option else { continue/return; };`
- `result.expect("infallible")` → `result.unwrap_or_else(|e| /* handle or return */)`

**How to apply — self-review agent:** After checking for `unwrap`, also `grep -n '\.expect(' src/` (excluding test modules) and verify each call. Acceptable forms: `.expect()` in tests, `.expect()` where poisoning means a genuine broken global invariant with the reason string explaining *why* it is unrecoverable. Anything else is a finding.

**Escalated?** agent:self-review, hook (PostToolUse Edit/Write panic-gate in `.claude/settings.json`)

### 2026-05-06 — process — "add to learnings" means learnings only; do not propagate to agent/skill files

**What happened:** User said "add to learnings: unwrap/expect/panic should be avoided... with explanations how this quality gate is broken." I wrote the learnings entry, then also updated `.claude/agents/self-review.md` and `.claude/agents/review-findings.md` citing the Propagation Rule. The user flagged this as unauthorized — they asked for one action (learnings), not three.

**Rule:** "Add to learnings" is a request to write to `ai-docs/learnings.md` only. The `Escalated?` field in the learnings entry records that an issue *is a candidate* for escalation to an agent/skill/AGENTS.md — it is not authorization to make that escalation immediately. The Propagation Rule in AGENTS.md only triggers when you are already editing an instruction file (`.claude/agents/**`, `.claude/skills/**`, `AGENTS.md`, `.claude/settings.json`). Creating a learnings entry is not editing an instruction file, so no propagation is triggered.

**How to apply:** When the user says "add to learnings" (or "note this", "remember this"), write the entry and stop. If the learnings entry merits escalation to project instructions, mark `Escalated? no` and let the user trigger `/improve` when ready. Never escalate to agents/skills on the same turn as a learnings-only request.

**Escalated?** no

### 2026-05-06 — process — `/next` skill cannot see "Blocked by:" in issue bodies; use a GitHub label instead

**What happened:** `/next small` recommended issue #48 (BlockingQueued) as a runner-up even though its body says "Blocked by: Per-thread event loops (#51, still open)". The skill fetches issues via `gh issue list --json number,title,labels,updatedAt` — it gets labels but never reads issue bodies. The "Skip blocked items" selection rule in the skill refers to the plan index (🔴 flags) and does not cross-reference GitHub issue bodies.

**Rule:** When a GitHub issue has a "Blocked by:" dependency on another open issue, add a `blocked` label to it on GitHub. The `/next` skill can then filter it out by label without fetching issue bodies. Do not rely on the "Blocked by:" section in the body being visible to the skill.

**How to apply:** After opening or triaging a new issue that depends on another open issue, immediately run `gh issue edit <N> --add-label blocked`. When the blocker is resolved, remove the label. This keeps the `/next` recommendation list accurate without requiring body reads.

**Escalated?** skill:next

### 2026-05-06 — process — resolve fixed PR review comments via GraphQL after pushing the fix

**What happened:** After fixing the panicking mutex ops (commits 543bb4f, 2ddece7), I replied to each comment but did not resolve the conversations. I attempted `resolveReviewThread` via GraphQL but used a guessed thread ID (`PRRT_kwDOSR5chs5UHUwU`) that did not exist, got NOT_FOUND, printed "resolve via graphql not available", and moved on. AGENTS.md says: "After pushing fixes, resolve only the comments that were addressed by a code change" — this was not done.

**Rule:** When a PR review comment is fixed by a code change, the conversation must be resolved. The flow is:
1. Reply to the comment explaining the fix.
2. Query for the actual thread node IDs:
   ```bash
   gh api graphql -f query='{ repository(owner:"OWNER", name:"REPO") { pullRequest(number:N) { reviewThreads(first:20) { nodes { id isResolved path } } } } }'
   ```
3. For each unresolved thread whose comment was addressed, call:
   ```bash
   gh api graphql -f query='mutation { resolveReviewThread(input:{threadId:"<id>"}) { thread { isResolved } } }'
   ```
4. Verify `isResolved: true` in the response.

When a GraphQL mutation fails with NOT_FOUND, do not silently move on — investigate: the thread ID was wrong, so query for the correct one first.

**How to apply:** In the post-push step after every PR fix round: reply first, then query for thread IDs, then resolve. If resolution fails, diagnose before giving up.

**Escalated?** AGENTS.md (GraphQL recipe added to "PR review comment resolution" bullet)

### 2026-05-06 — code-style — significant state-mutating fns should open with a debug! trace

**What happened:** `ObjectTree::rename` had a `debug!` call but `clear_name` was added without one. The reviewer caught the inconsistency.

**Rule:** Any function that makes a meaningful, traceable change to application state should open with a `debug!` call. This is at the writer's discretion — the threshold is "would someone debugging a live system want to see this in a log?" Minor field setters and high-frequency paths (e.g. every signal emit) do not qualify; high-frequency tracing should be gated behind a dedicated feature flag (e.g. `debug` or `trace-emit`) rather than always-on.

**Why:** Consistent traceability — when debugging lifecycle or state issues, log output should cover all significant mutations. Omitting a trace on one sibling method while its peers are traced breaks the log story.

**How to apply:** When adding a function that mutates non-trivial application state (tree mutations, lifecycle transitions, index updates, config changes): add a `debug!` at the top. When adding a sibling to an existing traced function, check whether it warrants the same treatment. Skip for: trivial getters/setters, emit paths, and any path called at high frequency without a feature-flag guard. Canonical feature name for high-frequency tracing: `verbose-tracing`.

**Escalated?** AGENTS.md, agent:self-review, agent:review-findings
