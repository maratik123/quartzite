---
name: bugfix
description: "Reactive bug-fixing workflow. Trace → Root cause → Failing test → Fix. Prevents the fix-break cycle."
when_to_use: "Activate on: 'not working', 'broken', 'wrong', 'incorrect', 'doesn't show', unexpected panic/crash/compile error, failing test that should pass. Divergence signal: 'expected X got Y', 'should return X but returns Y'. During implementation: 'this is wrong', 'overengineered', 'not what I meant'. Regression: 'broke again', 'stopped working', 'worked before'. SKIP for general questions, codebase exploration, known/planned limitations."
allowed-tools: Bash(cargo test *) Bash(cargo clippy *) Bash(cargo build) Bash(cargo fmt *) Bash(git rm ai-docs/bugfix/*) Bash(rm -f ai-docs/bugfix/*)
---

Reactive bug-fixing workflow. **Fundamentally different from `/task`:**
- First step is analysis, NOT code
- Failing test is written BEFORE the fix, not after

> ⛔ **Do NOT open Edit, do NOT write code until Step 2 (Root Cause) is complete.**

---

## Step 1: Reproduce and Trace

**Goal:** understand the exact sequence of events — what happens now, step by step.

> **⛔ BEFORE ANY FIX: DATA FIRST, THEN CODE.**
> Code shows what CAN happen. Data shows what DID happen.
> Hypothesis without data = guess.

Using only read-only tools (`rg`, `Read`, logs, test output), trace the actual execution path and draw an ASCII sequence diagram:

```
[ACTUAL]
Caller -(method())-> ServiceA -(query())-> DaoB
DaoB -(N records)-> ServiceA
ServiceA: applies logic X, but expected logic Y
ServiceA -(wrong result)-> Caller

[EXPECTED]
Caller -(method())-> ServiceA
ServiceA -(correct result)-> Caller
```

Adapt the diagram to the actual components involved. Label the divergence point explicitly.

**Inner steps — in this exact order:**

1. Create artifact `ai-docs/bugfix/trace-YYYY-MM-DD-<short-name>.md` (use today's date, kebab-case name):

```markdown
# Bugfix Trace: <bug description>
Date: YYYY-MM-DD
Reporter: <quote from user message>

## Actual behaviour
<ASCII sequence diagram showing what DID happen, with divergence point labelled>

## Expected behaviour
<ASCII sequence diagram showing what SHOULD happen>

## Confirmed by user: ⏳ PENDING
```

2. Show the trace to the user and ask: **"Did I understand the behaviour correctly?"**
3. After the user confirms — update the artifact: `Confirmed by user: ✅ YES`
4. **Do NOT proceed to Step 2 until the user confirms the trace.**

> **Artifact is required.** Step 2 starts with `Read ai-docs/bugfix/trace-*.md`.

---

## Step 2: Root Cause

> **First action:** `Read ai-docs/bugfix/trace-*.md`. If file missing — go back to Step 1.

Based on the confirmed trace — find the single point of failure.

**Rule:** root cause is **one place** in code where behavior diverges from the component's contract. If there are multiple candidates — those are symptoms, dig deeper.

1. Read only files involved in the trace
2. Find the line where the contract is violated
3. State: `"Root cause: function_name() in src/foo.rs line N — does X instead of Y"`
4. Append to the trace artifact:
   ```markdown
   ## Root Cause
   `function_name()` in `src/foo.rs` line N — does X instead of Y
   Confirmed by user: ⏳ PENDING
   ```
5. Show root cause to user. After confirmation → `✅ YES`.

---

## Step 3: Failing Test (REQUIRED, before Edit)

> ⛔ **Step 4 (Fix) is BLOCKED until the test is red.**

**Checklist:**
- [ ] Test written with the same data as the bug report
- [ ] Test run: `cargo test test_name`
- [ ] Test is **RED** with the expected error (not compile error — specifically assertion fail)
- [ ] Only after a red test → Edit

**Test must:**
- Be in `#[cfg(test)]` module or `tests/` directory
- Verify an invariant (comment out the fix → test must fail)
- Be named as a behavior description: `returns_error_when_input_empty`

**If the test does NOT fail:**
- You didn't find the root cause → go back to Step 2
- OR assertion is too weak → rewrite the assertion
- ⛔ Do NOT proceed to fix while test is still green

---

## Step 4: Plan + Regression Check

Before Edit — make a plan:

1. What exactly to change (file, function, ~lines)
2. What might break (adjacent components)
3. Which existing tests cover adjacent code — run them

**Show plan to user if:**
- Change touches >1 file
- You're changing the same file for the second time in a row (loop signal)

---

## Step 5: Fix

Now open Edit.

**One-file rule:** if fix requires changes in >3 files — you're fixing a symptom, not the root cause. STOP, go back to Step 2.

**One-attempt rule:** if a new bug appeared in the same place after the fix — STOP. Draw a full system diagram, show it to the user.

---

## Step 6: Verify

1. Run the failing test from Step 3: `cargo test test_name` — must turn green
2. Run the full suite: `cargo test` — confirm nothing else broke
3. Run `cargo clippy -- -D warnings` for changed files
4. Run `cargo fmt`
5. **Delete the trace artifact:** `git rm ai-docs/bugfix/trace-*.md 2>/dev/null || rm -f ai-docs/bugfix/trace-*.md` (handles both tracked and untracked traces)

---

## ⛔ Anti-pattern: fix-break cycle

```
Bug report → Edit → regression → bug report → Edit → regression → ...
```

**Signs you're in a loop:**
- User describes a similar symptom for the second time
- You're changing the same file >2 times
- User asks "why did you decide to do it that way?"

**Breaking the loop:**
1. STOP — close all Edits
2. Draw a full system diagram: all components + data flow + interactions
3. Show to user
4. Wait for understanding confirmation
5. Only then continue
