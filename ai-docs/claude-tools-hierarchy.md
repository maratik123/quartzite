# Claude Code — Tools, Subagents, Skills, Hooks (this project)

Snapshot of what is available to Claude in the `quartzite` working copy, marked by origin.

**Legend**: 🟦 embedded · 🟪 marketplace plugin · 🟩 project-defined (`.claude/`) · 🟧 user-global (`~/.claude/`)

Marketplace plugins enabled in `~/.claude/settings.json`:
- **`rust-analyzer-lsp@claude-plugins-official`** (v1.0.0) — provides the `LSP` tool
- **`ast-index@ast-index-marketplace`** (v3.42.0) — provides the `ast-index:*` skills

---

<!-- stable-anchor: #1-tools -->
<a id="1-tools"></a>
## 1. Tools

### 1a. Loaded tools

| Tool | Origin | Purpose | Parameters |
|------|--------|---------|------------|
| `Read` | 🟦 | Read a file (text / PDF / notebook / image) | `file_path`, `offset`, `limit`, `pages` |
| `Write` | 🟦 | Create or overwrite a file | `file_path`, `content` |
| `Edit` | 🟦 | Exact-string replacement | `file_path`, `old_string`, `new_string`, `replace_all` |
| `Bash` | 🟦 | Run shell command | `command`, `description`, `timeout` (max 600 000 ms), `run_in_background`, `dangerouslyDisableSandbox` |
| `Agent` | 🟦 | Spawn a subagent | `description`, `prompt`, `subagent_type`, `model`, `isolation` (`worktree`), `run_in_background` |
| `Skill` | 🟦 | Invoke a slash-command skill | `skill`, `args` |
| `AskUserQuestion` | 🟦 | Structured multi-choice prompt | `questions` (1–4 items), `annotations`, `metadata` |
| `ScheduleWakeup` | 🟦 | Schedule next `/loop` wake | `delaySeconds`, `reason`, `prompt` |
| `ShareOnboardingGuide` | 🟦 | Upload ONBOARDING.md | `mode`, `short_code` |
| `ToolSearch` | 🟦 | Load deferred tool schemas | `query`, `max_results` |
| `Grep` | 🟦 | Regex search across files | `pattern`, `path`, `glob`, `-i`, `-n`, `-l`, `-A`/`-B`/`-C`, `output_mode`, `type`, `head_limit`, `multiline` |
| `Glob` | 🟦 | Fast filename pattern matching | `pattern`, `path` |

### 1b. Deferred tools (load schema via `ToolSearch` first)

| Tool | Origin |
|------|--------|
| `CronCreate`, `CronDelete`, `CronList` | 🟦 |
| `EnterPlanMode`, `ExitPlanMode` | 🟦 |
| `EnterWorktree`, `ExitWorktree` | 🟦 |
| `LSP` | 🟪 `rust-analyzer-lsp` |
| `Monitor` | 🟦 |
| `NotebookEdit` | 🟦 |
| `PushNotification` | 🟦 |
| `RemoteTrigger` | 🟦 |
| `TaskCreate`, `TaskGet`, `TaskList`, `TaskOutput`, `TaskStop`, `TaskUpdate` | 🟦 |
| `WebFetch`, `WebSearch` | 🟦 |

---

<!-- stable-anchor: #2-subagents -->
<a id="2-subagents"></a>
## 2. Subagents (via `Agent` Tool)

> **Nested-spawn restriction.** The harness elides `Agent` from a Subagent's runtime tool list — **a Subagent cannot spawn another Subagent**, regardless of what its frontmatter `tools:` field declares. This applies to embedded Subagents (visible in §2a — `Explore` / `Plan` rows explicitly say "All except `Agent`") AND to project-defined Subagents (declaring `Agent` in `.claude/agents/<name>.md` frontmatter does NOT grant the primitive at runtime). All Subagent spawns originate from the orchestrator (main thread) or from a Skill. Empirically confirmed by [`ai-docs/learnings.md`](learnings.md) 2026-05-15 (`self-improve` runtime lacks `Agent`) and a 2026-05-23 `design-review` probe on PR #550.

### 2a. 🟦 Embedded

| Subagent | Purpose | Tools |
|----------|---------|-------|
| `claude` | Catch-all | `*` |
| `claude-code-guide` | Q&A about Claude Code / SDK / API | `Bash`, `Read`, `WebFetch`, `WebSearch` |
| `Explore` | Read-only code search | All except `Agent`, `Edit`, `Write`, `NotebookEdit`, `ExitPlanMode` |
| `general-purpose` | Open-ended research | `*` |
| `Plan` | Implementation strategy / architecture | All except `Agent`, `Edit`, `Write`, `NotebookEdit`, `ExitPlanMode` |
| `statusline-setup` | Configure status line | `Read`, `Edit` |

### 2b. 🟩 Project-defined (`.claude/agents/`)

| Subagent | Purpose | Tools |
|----------|---------|-------|
| `design` | Produce Design Document | All |
| `design-review` | GO / ITERATE / STOP verdict on design doc | `Read`, `Grep`, `Glob`, `Bash` |
| `learnings-escalation-audit` | Audit `Escalated?` / `Superseded by:` fields | All |
| `review-findings` | Whole-branch findings table | All |
| `self-improve` | Propose escalation diffs from `learnings.md` | All |
| `self-review` | APPROVE / REJECT diff vs spec/design | All |
| `spec-writer` | Draft task spec one round at a time | `Read`, `Write`, `Edit`, `Grep`, `Glob`, `Bash` |
| `triage-runner` | Promote deferred rows → gh issues | All |

---

<!-- stable-anchor: #3-skills -->
<a id="3-skills"></a>
## 3. Skills (via `Skill` Tool or typed `/<name>`)

### 3a. 🟦 Embedded

| Skill | Purpose |
|-------|---------|
| `code-review` | Diff review (low / medium / high / max); `--comment` posts inline |
| `run` | Launch / drive the project app |
| `review` | Review a PR |
| `security-review` | Security review of current branch |
| `init` | Initialize new `CLAUDE.md` |
| `update-config` | Edit `settings.json` (hooks, perms, env) |
| `keybindings-help` | Customize `~/.claude/keybindings.json` |
| `fewer-permission-prompts` | Generate allowlist from transcripts |
| `loop` | Run prompt / slash command on interval |
| `schedule` | Cron-scheduled remote agents |
| `claude-api` | Build / debug Claude API + SDK apps |
| `verify` | Verify that a code change actually does what it's supposed to by running the app and observing behavior. Use when asked to verify a PR, confirm a fix works, test a change manually, check that a feature works, or validate local changes before pushing. |

### 3b. 🟪 Marketplace (`ast-index-marketplace`)

| Skill | Purpose |
|-------|---------|
| `ast-index:ast-index` | Code search (symbol / class / usages / callers / impls / refs / outline) |
| `ast-index:initialize-android` | Init for Android / Kotlin / Java |
| `ast-index:initialize-web` | Init for TS / JS / React / Vue / Svelte |
| `ast-index:initialize-ios` | Init for iOS / Swift / ObjC |
| `ast-index:initialize-rust` | Init for Rust |
| `ast-index:initialize-csharp` | Init for C# / .NET |
| `ast-index:initialize-ruby` | Init for Ruby / Rails |

### 3c. 🟩 Project-defined (`.claude/skills/`)

| Skill | Purpose |
|-------|---------|
| `interview` | Spec interview → `ai-docs/plans/` + gh issue |
| `task` | Full task lifecycle (Steps 1–12) orchestrator |
| `bugfix` | Trace → root-cause → failing test → fix |
| `context-reset` | Handoff for large tasks / post-compaction re-entry |
| `verify-change` | Run `cargo test [filter]` to verify a change passes tests |
| `pr-commented` | One round of reviewer-comment response |
| `pr-ci-failed` | One CI-failure round on PR branch |
| `pr-merged` | Post-merge cleanup |
| `master-ci-failed` | CI-failure round on master (new branch + new PR) |
| `next` | Pick next task from deferred queue |
| `triage` | Promote deferred rows → gh issues |
| `improve` | Audit `learnings.md`; spawn `self-improve` |
| `ai-audit` | Repo-wide instruction-file audit |
| `project-review` | Whole-branch review (spawns `review-findings` + `self-review`) |
| `ui-design` | Quartzite design-system pointer |

---

<!-- stable-anchor: #4-hooks -->
<a id="4-hooks"></a>
## 4. Hooks

Hooks are not separately invocable — they fire automatically on harness events. All current hooks live in 🟩 `.claude/settings.json` (project-defined).

### 4a. `SessionStart` 🟩 — fire at every session start

| # | Status message | Effect |
|---|----------------|--------|
| 1 | "Syncing ast-index..." | Runs `ast-index update` (or `rebuild` if no index); injects `additionalContext: "ast-index: index updated"` |
| 2 | "Read session rules..." | Injects `additionalContext: "REQUIRED SESSION START to read rules from CLAUDE.md. Show user a summary of these rules"` |

### 4b. `PreToolUse` 🟩 — fire **before** a matching tool call

**Matcher: `Bash`**

| # | Status message | Trigger | Effect |
|---|----------------|---------|--------|
| 1 | "Checking branch before commit..." | `git commit` while on `master` | **Blocks** with `exit 2` and a recovery recipe (stash → branch → soft-reset master → pop stash) |
| 2 | "Regenerating + staging ROADMAP.md..." | Any `git commit` | Runs `scripts/gen-roadmap.sh`; blocks commit if it fails; otherwise stages `ROADMAP.md` |
| 3 | "Refreshing ast-index..." | Any `git commit` | Runs `scripts/update-ast-index.sh` in background (`exit 0` regardless) |

### 4c. `PostToolUse` 🟩 — fire **after** a matching tool call

**Matcher: `Write|Edit`**

| # | Status message | Trigger | Effect |
|---|----------------|---------|--------|
| 1 | "Formatting Rust..." | Edited file is `*.rs` | Runs `cargo fmt --quiet` workspace-wide |
| 2 | "Checking for panicking calls..." | `*.rs` not under `/tests/` or `/examples/` | Warns to stderr if `.unwrap()` / `.expect(` / `panic!(` appears outside `#[cfg(test)]` blocks |
| 3 | "Checking ROADMAP regeneration..." | Edited `ai-docs/plans/INDEX.md` | Reminds to run `scripts/gen-roadmap.sh` + stage `ROADMAP.md` in same commit |

**Matcher: `Bash`**

| # | Status message | Trigger | Effect |
|---|----------------|---------|--------|
| 4 | "Checking PR body sync..." | `git push` (no `gh pr create` in same line) on non-`master` branch with an OPEN PR | Warns to stderr to re-read PR body via `gh pr view <N>` (AXIOM 2) |
| 5 | "Checking ROADMAP staging..." | `git add ai-docs/plans/INDEX.md` without `ROADMAP.md` also staged | Warns to stderr to regenerate ROADMAP.md and stage both together |

---

## 5. Permissions (also from `settings.json`)

### Project `allow` 🟩

- `Edit/Write` under `./**`, `~/.claude/**`, `.claude/**`
- `Bash(cargo *)`, `Bash(git *)`, `Bash(gh *)`, `Bash(ast-index *)`, `Bash(grep *)`, `Bash(rg *)`

### Project `deny` 🟩

- `.idea/**` (Read / Edit / Write)
- `**/.env`, `**/.env.*` (Read / Edit / Write)
- `**/secrets*`, `**/.secrets*` (Read / Edit / Write)

### Project-local 🟩 `.claude/settings.local.json`

Extensive per-command allowlist (~600+ entries) for specific `Bash`, `WebFetch`, `Read`, `Skill` invocations the user has approved historically.

### User-global 🟧 `~/.claude/settings.json`

- `enabledPlugins`: `rust-analyzer-lsp@claude-plugins-official`, `ast-index@ast-index-marketplace`
- `extraKnownMarketplaces`: `ast-index-marketplace` (github: defendend/Claude-ast-index-search)
- `statusLine`: `sh ~/.claude/statusline-command.sh`
- `theme`: `dark`
- `includeCoAuthoredBy`: `false`, `attribution` cleared

---

<!-- stable-anchor: #mental-model -->
<a id="mental-model"></a>
## Mental model

```
Harness events (SessionStart, PreToolUse, PostToolUse, …)
  └── Hooks 🟩 (shell commands)  ─ fire automatically; can block tool calls or inject context

Tools (this turn)
  ├── Agent  ─►  Subagents  🟦 embedded | 🟩 project
  └── Skill  ─►  Skills     🟦 embedded | 🟪 plugin | 🟩 project

Plugins (~/.claude/plugins/)
  ├── rust-analyzer-lsp  ─►  LSP Tool
  └── ast-index          ─►  ast-index:* skills
```

---

<!-- stable-anchor: #stable-anchors -->
<a id="stable-anchors"></a>
## Stable anchors

The following anchors are load-bearing — other files in this project deep-link to them. Future PRs that edit this file MUST preserve every listed anchor OR update every inbound deep-link in the same PR (Propagation Rule).

- `#1-tools`
- `#2-subagents`
- `#3-skills`
- `#4-hooks`
- `#mental-model`
- `#stable-anchors`

Each load-bearing heading carries two pre-heading lines, in this order, with no blank line between them and the heading:

1. `<!-- stable-anchor: #<slug> -->` — diff-visible marker for editors.
2. `<a id="<slug>"></a>` — explicit HTML anchor element so inbound deep-links resolve regardless of how the GFM auto-slug for the heading evolves.
