# CI-failed skills — `/pr-ci-failed` + `/master-ci-failed`

**Source:** issue #357
**Date:** 2026-05-15
**Tracked in:** #357

## Scope

Two coordinated skills are authored in this task. Both own **one round of CI-failure response per invocation** and share the bulk of their per-invocation flow; they differ in preconditions and fix-landing path. The shared flow is factored out first; per-skill differences follow.

### 1a — Shared per-invocation flow (both skills)

1. Identify the relevant failing run on the in-scope ref (a feature branch's open PR for `/pr-ci-failed`; the latest / `$ARGUMENTS`-specified master commit for `/master-ci-failed`).
2. Resolve the failing run-id + job-id and fetch the failing-step log window (default ~200 lines) via `gh run view <run-id> --log-failed`, with `gh api repos/:owner/:repo/actions/runs/<run-id>/logs` as the documented fallback when the flag is unavailable.
3. Classify the failure into one of: `fmt` | `clippy` | `test` | `doc` | `actionlint` | `build` | `coverage` | `other`. Each class maps to a known local reproducer command (see *Technical constraints* table).
4. Run the mapped reproducer locally. If it does **not** reproduce, **STOP** and report — likely flake / runner-specific / dirty cache. User decides next step.
5. Diagnose root cause; apply edits; re-run the reproducer until green.
6. Spawn the `self-review` agent on the diff (mandatory — enforces AGENTS.md `## Workflow` *CI-fix commits get self-review too* axiom).
7. Stage explicitly, commit (heredoc message citing the failing run URL).
8. Push (and, for `/master-ci-failed` only, open a fresh PR — see 1b vs 1c).

### 1b — `/pr-ci-failed` (renamed from `/pr-failed`)

Per-invocation differences from the shared flow:

- **Preconditions.** `git branch --show-current` ≠ `master`; `git status --porcelain` empty; `gh pr view --json number,state,headRefName` matches current branch and `state == OPEN`; `gh pr checks <N>` has at least one failing required check. If all green: print *No failing checks on PR #<N>; exiting.* and exit cleanly.
- **Fix-landing path.** Commit + push to the existing feature branch of the open PR. After `git push`, the skill runs `gh pr view <N> --json title,body` (AGENTS.md AXIOM 2) and edits the PR body only when the new commit contradicts the description (rare for CI-fix commits).
- **Optional step 9.** Re-poll `gh pr checks <N>` until the previously-failing check turns green or a new failure surfaces; if a new failure surfaces, the user re-invokes `/pr-ci-failed`.
- **Master-only / closed-PR bail.** If the branch is `master`, or the matched PR is closed/merged, the skill bails with a pointer to `/master-ci-failed` (master case) or a no-op (closed/merged case).

### 1c — `/master-ci-failed` (new)

Per-invocation differences from the shared flow:

- **Preconditions.** `git branch --show-current` == `master`; `git status --porcelain` empty; the latest master push (or the `$ARGUMENTS`-specified master commit SHA) has at least one failing required check. NO open PR is required on the failing commit. If there **is** an open PR pointing at the failing commit (unusual but possible — e.g. a hotfix branch was force-merged), bail with *This looks like a `/pr-ci-failed` case — switch to that skill.*
- **`$ARGUMENTS` semantics.** Default: scan the latest master commit's CI run. Optional argument: a specific master commit SHA (for retroactive fixes to an older red run that is still red on master HEAD). Discovery semantics for "latest failing master run" are deferred to design (see *Open questions*).
- **Fix-landing path (no direct commit on master).**
  1. Create a new feature branch off the failing master commit (or current master HEAD if it has progressed but the failure is still red): `git checkout -b fix/master-ci-<run-id>`.
  2. Apply the fix; run the local reproducer to green.
  3. Spawn `self-review` (mandatory, same axiom).
  4. Commit with a heredoc message citing the failing master run URL.
  5. `git push -u origin <branch>`.
  6. `gh pr create` opens a new PR titled `fix(master-ci): <one-line summary> (run <run-id>)`. PR body cites the failing run URL, the local-PASS evidence, and the failure class.
  7. Skill exits cleanly. Next reviewer-feedback round goes through `/pr-commented`; next CI-feedback round on the new PR goes through `/pr-ci-failed`.
- **Master is never directly modified.** No commits on master. No force-push (AGENTS.md `## Permissions` denies it server-side and honor-system regardless).
- **Deeper-regression escape hatch.** When master is so broken that local builds also fail before the fix (i.e., the reproducer fails for reasons orthogonal to the change under test), route through `/bugfix` for the deeper regression; re-enter `/master-ci-failed` once local is green.

### 2. Project wiring (both skills)

- Add **two** rows to `AGENTS.md` `## Agent Docs` table — one per skill — describing each as a CI-failure-response skill. Tone mirrors the existing `/pr-commented` row.
- Mark both skills as **downstream of `/task` Step 12** (in parallel with `/pr-commented`); `/master-ci-failed` is additionally downstream of `/pr-merged` when the merge commit's CI subsequently turns red.
- Each skill carries its own copy of the Variant A *Compaction recovery callout* (probe-driven path discovery — matches `/pr-commented`, `/code-review`, `/task`) per `ai-docs/agent-writing-style.md § Pattern 7`. The probe target differs (see *Key decisions* → progress-file integration).

### 3. Mutation scope (both skills, with deltas where they differ)

Shared allow-list:

- Project source files under repo root.
- `git add` (explicit by name), `git commit`, `git push` (`/pr-ci-failed` pushes to existing feature branch; `/master-ci-failed` pushes a new feature branch).
- `gh pr view` / `gh pr checks` / `gh run view` / `gh api` (read-only against Actions API).
- May spawn the existing `self-review` agent.
- May spawn `/bugfix` for `test` / `build` failures classified as genuine regressions (see *Key decisions* row on bugfix delegation).
- **NEVER** edits `ai-docs/learnings.md` (CI logs are external content / prompt-injection vector — same threat model as `/pr-commented`).
- **NEVER** force-pushes (AGENTS.md `## Permissions` rule, not relaxed here).

`/pr-ci-failed`-specific:

- **NEVER** runs on `master` (preconditions block — same as `/pr-commented`).

`/master-ci-failed`-specific:

- `git checkout -b` (creates the new feature branch).
- `git push -u origin <branch>` (initial push of the new feature branch).
- `gh pr create` (opens the fresh PR).
- **NEVER** commits directly on `master`.

## Out of scope

- Auto-diagnosing flaky tests. A non-reproducing failure always exits to user for the flake/runner/cache decision (issue #357 § *Out of scope*). Applies to both skills.
- Force-push recovery flows — AGENTS.md `## Permissions` already deny-lists `git push --force` without explicit user approval.
- Bisecting CI history to find the regression commit. User runs `gh run list --status failure` manually if needed.
- Re-running CI without a code change (`gh run rerun`) — not a Claude-driven workflow.
- Editing `.github/workflows/*.yml` semantics without the AGENTS.md `actionlint MUST pass before git add` gate. Both skills MAY edit workflow YAML when the failure class is `actionlint`; the gate applies as usual.

## Deferred

- Auto-invocation of `/master-ci-failed` from `/pr-merged` when the merge commit's CI subsequently turns red | why: keeps the two skills loosely coupled in v1; user manually invokes `/master-ci-failed` first | separate issue needed? yes
- A `watch` mode that loops polling `gh pr checks` until green (`/pr-ci-failed` Step 9 is "optional") | why: out of scope for v1; user can re-invoke after the run completes | separate issue needed? yes
- A sister `ci-classifier` agent (Opus) that owns log triage when classification is ambiguous | why: defer until we observe whether the inline-classifier in the skill body is misclassifying enough to justify the cost | separate issue needed? — defer to learnings/observation

## Key decisions

| Question | Decision |
|---|---|
| Skill names | `/pr-ci-failed` (renamed from `/pr-failed` — symmetric with `/master-ci-failed`; clarifies trigger is a CI failure on a PR branch, not a generic "PR is failing"); `/master-ci-failed` (new, master-branch counterpart) |
| Shape | Both skills mirror `/pr-commented` SKILL.md: YAML frontmatter (`name`, `description`, `disable-model-invocation: false`, `allowed-tools` list) → commit-authorisation banner → Scope (In/Out) → Compaction recovery callout (Variant A) → Preconditions table → Step-by-step Workflow → Edge cases → Anti-patterns → Gate checklist |
| Skill model invocation | `disable-model-invocation: false` for both (model MAY auto-invoke when watching CI, in addition to user invocation) — diverges from `/pr-commented`'s `true`. Rationale: a CI failure has a deterministic remediation path (the per-class reproducer table) and the skill body already enforces self-review before any push (AGENTS.md `## Workflow` *"CI-fix commits get self-review too"* axiom, loop cap 3), so the autonomous-push concern is bounded. Reviewer direction on PR #361 (`fix(pr-361): address review round 1` commit). |
| Sister agent? | **No** for either skill. Mirror `/pr-commented` (no sister agent). The skill body owns classification + diagnosis; only `self-review` (existing) and optionally `/bugfix` (existing) are spawned. Consequence: no new Propagation Rule rows. |
| Progress-file integration — `/pr-ci-failed` | Extend the existing `/task` progress file at `ai-docs/plans/<spec-base>.progress.md` with a new `## CI-fix cycle round M` section (symmetric with `/pr-commented`'s `## Comment cycle round M`). Fallback path for PRs not produced by `/task`: `ai-docs/ci-fixes/pr-<N>.progress.md` (gitignored, deleted by `/pr-merged`). |
| Progress-file integration — `/master-ci-failed` | No `/task` progress file is linked by PR-number (no PR exists yet at skill entry). Fallback path: `ai-docs/master-ci/<run-id>.progress.md` (gitignored). Deleted by `/pr-merged` after the fresh PR's eventual merge — the new PR's `Tracked in` line references this run-id so `/pr-merged`'s cleanup probe can find it. Design agent may pick a simpler shape if the per-run file shows no benefit. |
| Per-class reproducer mapping | (Shared) `fmt` → `cargo fmt -- --check`; `clippy` → `cargo clippy --workspace -- -D warnings`; `test` → `cargo test <substring or path>`; `doc` → `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`; `actionlint` → `actionlint <changed-workflow-file>`; `build` → `cargo build` (or `cargo build -p quartzite --no-default-features --features libm` for the no_std gate); `coverage` → the coverage workflow's local equivalent (recorded in skill body); `other` → pause and surface logs to user. |
| Bugfix delegation threshold | (Shared) Inline-fix classes: `fmt`, `clippy`, `doc`, `actionlint` (mechanical / lint-shaped). Delegate to `/bugfix`: `test`, `build` when the failure is a genuine regression (root cause unclear from logs alone). `coverage` / `other`: pause and surface to user — they may map to either path. For `/master-ci-failed`, when local builds also fail orthogonally to the change, route through `/bugfix` first and re-enter once green. |
| No-reproduce path | (Shared) Terminal exit. The skill prints the failing-step log excerpt, the classifier's pick, the reproducer command run, and the local-PASS evidence. User decides: cache-clear retry, `gh run rerun`, defer, or re-invoke after investigation. |
| Self-review boundary | (Shared) Mandatory before every `git push` from either skill — even one-liner fixes. Enforces the AGENTS.md `## Workflow` `**CI-fix commits get self-review too.**` bullet. Loop cap **3** REJECT attempts per round (same as `/pr-commented` Step 5); on the 3rd REJECT, surface the verdict and stop. |
| Commit-message format — `/pr-ci-failed` | `fix(pr-<N>): CI <class> failure — <one-line summary>`; body cites the failing run URL (`https://github.com/<O>/<R>/actions/runs/<run-id>`) and the local reproducer command that re-ran green. |
| Commit-message format — `/master-ci-failed` | `fix(master-ci): <one-line summary> (run <run-id>)`; body cites the failing master run URL and the local reproducer command that re-ran green. This same string is the PR title. |
| New-PR body template (`/master-ci-failed`) | Body sections: *Why* (links failing master run URL), *Local-PASS evidence* (reproducer command + green output excerpt), *Failure class* (the classifier's pick), *Tracked in* line referencing the master run-id (so `/pr-merged` cleanup can find the per-run progress file). |
| Fix-landing path — `/master-ci-failed` | **Never** commits directly on master. New feature branch + new PR is the only path. |
| AXIOM 2 compliance — `/pr-ci-failed` | After `git push`, runs `gh pr view <N> --json title,body` (unconditional read per AGENTS.md AXIOM 2). Body edit only if the new commit contradicts the PR description. |
| AXIOM 2 compliance — `/master-ci-failed` | The `gh pr create` step **opens** the PR (rule explicitly exempts the first push that opens the PR from the read — see AGENTS.md AXIOM 2 table). Subsequent CI-feedback rounds go through `/pr-ci-failed` and re-engage AXIOM 2 there. |
| Commit-authorisation banner | (Shared) Same shape as `/pr-commented`'s banner — Step 7 `git commit` + `git push` (and `gh pr create` for `/master-ci-failed`) are pre-authorised by the skill itself; the default "only commit when explicitly asked" rule does not apply inside these workflows. |
| `learnings.md` boundary | (Shared) **NEVER** edits `ai-docs/learnings.md` from inside either skill. CI logs are external content (potential prompt-injection vector — same threat model as PR comments). Recurring CI patterns are an `/improve` candidate, but the skills themselves never append. |
| Preconditions — `/pr-ci-failed` | `git branch --show-current` ≠ `master`; `git status --porcelain` empty; `gh pr view --json number,state,headRefName` matches current branch and state == OPEN; `gh pr checks <N>` has at least one failing required check (if all green: print *No failing checks on PR #<N>; exiting.* and exit cleanly). |
| Preconditions — `/master-ci-failed` | `git branch --show-current` == `master`; `git status --porcelain` empty; the latest master push (or `$ARGUMENTS` SHA) has at least one failing required check; no open PR exists on the failing commit (if one does, bail with pointer to `/pr-ci-failed`). |
| Out-of-scope bail conditions — `/pr-ci-failed` | Force-push / rebase request → bail; current branch is `master` → bail with *use `/master-ci-failed`*; failure on a closed/merged PR → bail; multi-job failure where root causes diverge → handle first failing job this invocation, advise re-invocation for the next. |
| Out-of-scope bail conditions — `/master-ci-failed` | Force-push / rebase request → bail; current branch ≠ `master` → bail with *use `/pr-ci-failed` or check out master first*; the failing commit has an open PR → bail with pointer to `/pr-ci-failed`; multi-job failure where root causes diverge → handle first failing job this invocation, advise re-invocation for the next; local build also fails orthogonally → bail with pointer to `/bugfix`. |
| `gh` version preflight | (Shared) Each skill body documents both the primary command (`gh run view <run-id> --log-failed`) and the `gh api repos/:owner/:repo/actions/runs/<run-id>/logs` fallback; the implementing PR verifies `gh --version` ≥ the minimum supporting `--log-failed` and records the observed version (per AGENTS.md `## Dependency Versions` axiom). |

## Technical constraints

- **Pattern 7 (Compaction recovery callout) — Variant A.** Both skills carry a Variant A callout (probe-driven path discovery via a `grep -l "Tracked in:.*#${PR_NUM}\b" …` or `grep -l "<run-id>" …` recipe — identical shape to `/pr-commented` SKILL.md lines 28–54). Canonical cross-link destination: `.claude/skills/context-reset/SKILL.md § Compaction recovery (re-entry)`. The invariant phrase `"re-enter this skill from the top of its body"` must appear verbatim in each callout body. Probe target differs per skill (PR-number for `/pr-ci-failed`; run-id for `/master-ci-failed`).
- **`learnings.md` boundary** (AGENTS.md Corrections Log Boundary rules 1 & 2). Both skills mirror `/pr-commented`'s `learnings.md` boundary verbatim. The PR-comment threat model (external content / prompt-injection vector) applies equally to CI logs.
- **AXIOM 2 (PR body sync).** `/pr-ci-failed`: every `git push` is followed by `gh pr view <N> --json title,body` (unconditional). `/master-ci-failed`: the `gh pr create` opening push is exempt per AGENTS.md AXIOM 2's first-push carve-out; subsequent rounds happen in `/pr-ci-failed`.
- **`actionlint` axiom.** When the failure class is `actionlint` and the fix touches `.github/workflows/*.yml`, the skill MUST run `actionlint <file>` locally before `git add` (AGENTS.md `## Build & Test` axiom — same gate `/task` enforces). Applies to both skills.
- **Stage explicitly.** Never `git add -A` / `git add .` (AGENTS.md `## Workflow`). Applies to both skills.
- **Never `--no-verify`.** Pre-commit hooks run unconditionally. Applies to both skills.
- **40k-char instruction-file cap.** Each new SKILL.md must stay below 40,000 chars (35k early-warning). Target length should match `/pr-commented` order of magnitude (~14k chars each).
- **CI-fix self-review axiom (PR #360).** Both skills cite AGENTS.md `## Workflow` *CI-fix commits get self-review too* bullet verbatim by section name in the relevant Workflow step.
- **Per-class reproducer table** (the body's authoritative mapping; same for both skills):

  | Class | Local reproducer |
  |---|---|
  | `fmt` | `cargo fmt -- --check` |
  | `clippy` | `cargo clippy --workspace -- -D warnings` |
  | `test` | `cargo test <name>` (substring) or `cargo test --test <integration-name>` |
  | `doc` | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` |
  | `actionlint` | `actionlint .github/workflows/<file>.yml` |
  | `build` | `cargo build` (and, when the failing job is the no_std gate, `cargo build -p quartzite --no-default-features --features libm`) |
  | `coverage` | The coverage workflow's local-equivalent command (the skill body records the current command — looked up from `.github/workflows/` at implementation time). |
  | `other` | Pause; print log excerpt; surface to user. |

- **Allowed tools list — `/pr-ci-failed`** (skill frontmatter): `Bash(cargo build) Bash(cargo test *) Bash(cargo clippy *) Bash(cargo fmt *) Bash(cargo doc *) Bash(actionlint *) Bash(git diff *) Bash(git status *) Bash(git log *) Bash(git rev-parse *) Bash(git branch *) Bash(git checkout *) Bash(git add *) Bash(git commit *) Bash(git push *) Bash(git merge-base *) Bash(gh pr view *) Bash(gh pr checks *) Bash(gh run view *) Bash(gh run list *) Bash(gh api *) Bash(gh pr edit *)`.
- **Allowed tools list — `/master-ci-failed`** (skill frontmatter): superset of the above plus `Bash(gh pr create *)` (creates the fresh PR) and `Bash(git push -u origin *)` (initial push of the new feature branch).
- **No new dependencies.** Both skills consume existing tooling only (`gh`, `cargo`, `actionlint`, `self-review`, optionally `/bugfix`). No `Cargo.toml` edits.
- **Workflow YAML edits.** When a fix changes `.github/workflows/*.yml`, the AGENTS.md AXIOM table rows still apply (actionlint gate; PR safety-net check for master-only triggers).

## Acceptance Criteria

### Applies to both skills

| # | Criterion |
|---|-----------|
| AC1 | Two new files exist: `.claude/skills/pr-ci-failed/SKILL.md` and `.claude/skills/master-ci-failed/SKILL.md`. Each has YAML frontmatter (`name: pr-ci-failed` / `name: master-ci-failed`, `description: …`, `disable-model-invocation: false`, `allowed-tools: …` matching the *Technical constraints* allow-list for that skill). |
| AC2 | Each SKILL.md body carries: commit-authorisation banner, *Scope* (In / Out), Variant A *Compaction recovery callout* (with the canonical cross-link to `.claude/skills/context-reset/SKILL.md § Compaction recovery (re-entry)` and the invariant phrase `"re-enter this skill from the top of its body"`), *Preconditions* table, numbered Workflow steps covering identify → fetch logs → classify → reproduce → diagnose/fix → self-review → commit → push (+ `gh pr create` for `/master-ci-failed`), *Edge cases* table, *Anti-patterns* list, *Gate checklist*. Shape and ordering match `/pr-commented` SKILL.md. |
| AC3 | The Workflow in each SKILL.md contains the explicit per-class reproducer table from *Technical constraints*. |
| AC4 | Each skill body explicitly states it **never** edits `ai-docs/learnings.md` (mirroring `/pr-commented` line 26 wording). The threat model (external content / prompt-injection) is named verbatim. |
| AC5 | Self-review is spawned before every `git push` from either skill — each skill body cites the AGENTS.md `## Workflow` *CI-fix commits get self-review too* bullet by section name. Self-review loop cap = 3 REJECT attempts per round; on the 3rd REJECT the skill surfaces the verdict and stops without pushing. |
| AC6 | Each skill body documents both the primary log-fetch command (`gh run view <run-id> --log-failed`) and the `gh api repos/:owner/:repo/actions/runs/<run-id>/logs` fallback; the implementing PR also records the observed `gh --version` (per AGENTS.md `## Dependency Versions` axiom) in each skill body or a `## Pre-task verification` comment. |
| AC7 | `AGENTS.md § Agent Docs` table gains **two** new rows — one for `/pr-ci-failed`, one for `/master-ci-failed` — each describing the skill as a CI-failure-response skill downstream of `/task` Step 12 (and, for `/master-ci-failed`, also downstream of `/pr-merged`), with the `learnings.md` boundary called out. Row tone matches the existing `/pr-commented` row. |
| AC8 | Each new SKILL.md is below 40,000 chars (`wc -c …` < 40000); ideally < 20,000 chars (order-of-magnitude parity with `/pr-commented`). |
| AC9 | Neither skill body introduces a sister agent (decided in *Key decisions*). If the implementing PR introduces one for either skill, it adds the corresponding Propagation Rule row in the same PR. |
| AC10 | The implementing PR makes no edits to `ai-docs/learnings.md` as part of the same diff (the spec-implementation itself is not a learning-capture event). |
| AC11 | Both skills cite the AGENTS.md `## Workflow` *CI-fix commits get self-review too* bullet verbatim by section name in the Workflow step that spawns `self-review` (per *Technical constraints* row). |

### `/pr-ci-failed` only

| # | Criterion |
|---|-----------|
| AC-P1 | Preconditions bail when: branch is `master`, working tree is dirty, no open PR matches the branch, or no failing required check exists on the PR. The "no failing check" exit prints `No failing checks on PR #<N>; exiting.` and exits cleanly (not an error). The `master`-branch bail names `/master-ci-failed` as the alternative. |
| AC-P2 | Skill body documents the progress-file integration: extends `/task` progress file at `ai-docs/plans/<spec-base>.progress.md` with `## CI-fix cycle round M`; fallback `ai-docs/ci-fixes/pr-<N>.progress.md` for non-`/task` PRs. Both paths gitignored (the implementing PR extends `.gitignore` if the fallback directory is not yet covered) and deleted by `/pr-merged`. |
| AC-P3 | Commit-message format per *Key decisions* (`fix(pr-<N>): CI <class> failure — <one-line summary>`); body cites the failing run URL and the local reproducer command that re-ran green. AXIOM-2 read step (`gh pr view <N> --json title,body`) is present verbatim after `git push`. |
| AC-P4 | Step 9 (optional re-poll of `gh pr checks <N>`) is documented as optional and surfaces a new failure path that points the user back to `/pr-ci-failed` re-invocation. |

### `/master-ci-failed` only

| # | Criterion |
|---|-----------|
| AC-M1 | Preconditions bail when: branch ≠ `master`, working tree is dirty, the latest master push (or `$ARGUMENTS` SHA) has no failing required check, or the failing commit has an open PR. The branch ≠ `master` bail names `/pr-ci-failed` as the alternative; the "has open PR" bail also names `/pr-ci-failed`. |
| AC-M2 | Fix-landing path is documented as: new feature branch (`git checkout -b fix/master-ci-<run-id>`) off the failing commit (or master HEAD) → fix → reproducer green → self-review → commit → `git push -u origin <branch>` → `gh pr create` with the title/body template from *Key decisions*. Master is **never** directly committed to. |
| AC-M3 | `$ARGUMENTS` semantics documented: default = latest master commit; argument = specific master commit SHA. The "latest failing master run" discovery rule is referenced from *Open questions* (design agent finalises). |
| AC-M4 | Skill body documents the deeper-regression escape hatch: when local build also fails orthogonally, route through `/bugfix`, then re-enter `/master-ci-failed` once local is green. |
| AC-M5 | New PR title is `fix(master-ci): <one-line summary> (run <run-id>)`. PR body has the four sections from *Key decisions*: *Why* (run URL), *Local-PASS evidence*, *Failure class*, *Tracked in* line referencing the master run-id so `/pr-merged` cleanup can find the per-run progress file. |
| AC-M6 | Progress-file integration uses `ai-docs/master-ci/<run-id>.progress.md` (gitignored; the implementing PR extends `.gitignore` if the directory is not yet covered; deleted by `/pr-merged` via the `Tracked in` run-id probe). The design agent may pick a simpler shape if a per-run file shows no benefit; AC is satisfied by either shape provided the cleanup path remains coherent. |

## Open questions

- **Bugfix delegation hand-off shape.** When either skill classifies a `test` failure as a regression and delegates to `/bugfix`, does control return to the calling skill for the self-review + push step, or does `/bugfix` own the push and the calling skill exits? Default for the design phase: `/bugfix` owns the full Trace → Root cause → Fix → self-review → push loop; the calling skill exits after delegation. Revisit if this leaves a gap in the audit trail. (Applies to both skills.)
- **Coverage failure reproducer.** The `coverage` class needs a concrete local-reproducer command. Design phase looks up the current command from `.github/workflows/` at implementation time; if no clean local equivalent exists, `coverage` falls back to the `other` (pause-and-surface) path. (Applies to both skills.)
- **Auto-invocation of `/master-ci-failed` from `/pr-merged`.** Deferred to a separate issue (recorded in `## Deferred`); revisit once `/master-ci-failed` lands and we observe how often merge commits turn the master CI red.
- **`gh --version` minimum.** Pinned at implementation time per the AGENTS.md `## Dependency Versions` axiom. Pre-task verification step in the implementing PR confirms `--log-failed` is the right flag in the live `gh` version.
- **"Latest failing master run" discovery for `/master-ci-failed`.** When `$ARGUMENTS` is empty, does the skill look at the latest master push's CI run only, or scan the last N master pushes for any still-failing run? Design's call. Default for the design phase: scan the latest master push only; if it's green but an older still-red commit is suspected, the user passes the SHA explicitly via `$ARGUMENTS`.
