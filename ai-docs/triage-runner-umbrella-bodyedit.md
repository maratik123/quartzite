# Triage Runner — umbrella body auto-edit procedure

> Extracted verbatim from `.claude/agents/triage-runner.md` Phase 6.5 / Phase 7 gate, numbered-pick branch **sub-step 4** (umbrella body auto-edit, per Tech #8). The pointer + sub-step label stay in `triage-runner.md`; the `gh`-view → anchor-scan → `gh`-edit machinery (sub-steps a–e) lives here. Referenced from `.claude/skills/triage/SKILL.md` § Design-work classification gate. Read on demand.

**Umbrella body auto-edit (per Tech #8).** Edit `#N`'s body in-place under the `## Child issues (blocked on this epic)` anchor. Read the current body into a shell variable, then `Write` it (modified) to the staging file `ai-docs/triage/umbrella-<N>.body.md` (inside the subagent's mutation scope; `ai-docs/triage/**` is gitignored) — **no `>` file-redirect**:

```bash
body=$(gh issue view <N> --json body --jq .body)
```

(`--jq` is `gh`'s own JSON extraction, not a shell pipe to `jq` and not a `>` redirect.) Apply sub-steps a–d to `$body`, then use the `Write` tool to write the modified body to `ai-docs/triage/umbrella-<N>.body.md`.

a. **Locate the anchor** — verbatim substring `## Child issues (blocked on this epic)` (case-sensitive; #539–#542 share verbatim).

b. **Idempotency check (BEFORE writing).** Scan from the anchor line forward to the END-of-section boundary (rule c) for substring `#<C> ` (**trailing-ASCII-space sentinel** prevents `#54` matching `#549`). If present, **no-op** the edit; log under Phase 8's per-umbrella summary as "already linked".

c. **END-of-section detection rule.** From the line immediately after the anchor, scan forward: a line beginning with `## ` (any next h2) is the **boundary** — insert the bullet on its own line immediately BEFORE that boundary, preserving section blank-line spacing. Reaching end-of-body without another `## ` makes the boundary **EOF** — append the bullet at EOF with a trailing newline.

d. **Compose the bullet** — exactly `- #<C> — <child-title>` (full child title, NOT the menu's 80-char truncation).

e. **Push back** — `gh issue edit <N> --body-file ai-docs/triage/umbrella-<N>.body.md`. Capture exit code. Clean up the staging file after the call returns: `rm -f ai-docs/triage/umbrella-<N>.body.md`.
