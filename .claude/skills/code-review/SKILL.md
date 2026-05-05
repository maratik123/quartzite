---
name: code-review
description: "Whole-codebase review on the current branch (or branch given as argument). Reads all source files and done plans, runs fix loop and self-review loop until APPROVE, then commits."
disable-model-invocation: true
argument-hint: "[branch-name]"
allowed-tools: Bash(cargo build) Bash(cargo test *) Bash(cargo clippy *) Bash(cargo fmt *) Bash(cargo doc *) Bash(git diff *) Bash(git rev-parse *) Bash(git checkout *) Bash(git branch *) Bash(git log *) Bash(git add *) Bash(git commit *)
---

Whole-codebase review workflow. Steps execute **strictly in sequence**.

## ⚡ First: check for active review

```bash
ls ai-docs/plans/*.progress.md 2>/dev/null
```

**If found → RESUME:**
1. Read the `.progress.md` file
2. Jump to `## Next action`
3. Tell user: "Found active review, resuming from [next action]"

---

### Step 1: Determine branch

- If `$ARGUMENTS` is non-empty: confirm the user wants to review that branch, then `git checkout $ARGUMENTS`.
- Otherwise: use current branch (`git branch --show-current`).

Record `base_commit`:
```bash
git rev-parse HEAD
```

### Step 2: Spawn review agent

Create the progress file path: `ai-docs/plans/YYYY-MM-DD-code-review.progress.md` (use today's date).

```
Agent(subagent_type="general-purpose", prompt="
  Read .claude/agents/review-findings.md and follow it exactly.
  Branch: [branch name]
  base_commit: [base_commit]
  Write progress file to: ai-docs/plans/YYYY-MM-DD-code-review.progress.md
")
```

After the agent completes: read the progress file and report finding count and severity breakdown to the user.

### Step 3: Fix loop

For each `⬜ Open` finding in the `## AC Status` table (top-to-bottom):

- **Fix it** → implement the change, mark `✅ Fixed` in the progress file.
- **Object to it** (finding is wrong or intentionally out of scope):
  - `nit` / `minor`: may object autonomously — write reason, mark `⚠️ Objected: <reason>`.
  - `major` / `blocker`: **surface to user first** before objecting. User must approve.

After every 3 fixes (or when all findings in a subtask are resolved):
1. `cargo build` — must compile
2. `cargo test` — all green
3. `cargo clippy -- -D warnings` — clean
4. `cargo fmt`
5. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — clean
6. Update `## Files touched` and mark subtask `[x]` in progress file

**Context handoff rule:** if the finding count is ≥ 10 and more than half remain open, spawn a sub-agent per subtask rather than working inline — pass the progress file path so it can resume.

### Step 4: Final verify

1. `cargo build` — PASS
2. `cargo test` — all green
3. `cargo clippy -- -D warnings` — clean
4. `cargo fmt -- --check` — clean
5. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — clean (matches CI)
6. **Doc convention conformance.** For every changed `pub` item, verify it conforms to [`ai-docs/doc-convention.md`](../../../ai-docs/doc-convention.md) (summary tense, `# Parameters` on fns with ≥1 non-receiver arg, strict section order, `# Errors` / `# Panics` / `# Safety` where applicable). Methods inside `impl Trait for Type {}` blocks are exempt; the trait *definition* is not. Mechanical heading scan on changed files: `rg '^\s*///\s*#\s*(Parameters|Returns|Type parameters|Lifetimes|Errors|Panics|Safety|Examples|See also)\b' <file>`.
7. Update progress file: `**Last build:** PASS`

### Step 5: Self-review loop (max 3 rounds)

```
Agent(subagent_type="general-purpose", prompt="
  Read .claude/agents/self-review.md and follow it.
  Progress: ai-docs/plans/YYYY-MM-DD-code-review.progress.md
  base_commit is recorded in the progress file.
  There is no spec or design doc — this is a review-driven task.
  Treat the findings table in ## AC Status as the acceptance criteria.
")
```

**On APPROVE:**
1. `cargo fmt` (final pass)
2. Commit all changes (see commit rules below)
3. Delete `ai-docs/plans/YYYY-MM-DD-code-review.progress.md`
4. Done.

**On REJECT:**
- Fix each `⬜ Open` finding from the self-review section (same fix/object rules as Step 3)
- Return to Step 5 (loop)

**After round 3 with REJECT:** surface all remaining `⬜ Open` findings to the user and ask how to proceed. Do not delete `.progress.md` until resolved.

### Commit rules

```bash
git add <all changed files — list them explicitly, no -A>
git commit -m "$(cat <<'EOF'
[brief summary of what the review fixed]

Review findings addressed:
- #N: description (severity)
- ...

Deferred:
- #N: description — reason
EOF
)"
```

## Gate checklist

| Before | Check |
|---|---|
| Step 2 | branch confirmed? base_commit recorded? |
| Step 3 | build green after every 3 fixes? |
| Step 4 | all six checks pass (build, test, clippy, fmt, doc, doc convention)? |
| Step 5 | self-review APPROVE before commit? |
| Commit | `major`/`blocker` objections user-approved? progress file deleted? |
