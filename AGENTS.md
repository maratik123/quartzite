# Rust OSS Agent Rules

**CRITICALLY**
1) English for all output. Other language only on explicit user request.
2) Minimise thinking output shown to user.

## Project

> Read `ai-docs/context.md` for project purpose, entities, architecture, and design decisions — on demand.

## Permissions

Enforced by `settings.json`. Intent:

**ALLOW:** project files, `git`, `cargo`, `gh`, `~/.claude`
**DENY:** `.env`, `secrets*`, `git push --force`, files outside project
**ASK:** any tool not listed above; if access denied — suggest an alternative

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
```

Search: `rg <pattern> --type rust [-l | -C 3]`

## Code Style

- **All source files must be Rust (`.rs`).** No other languages in `src/`.
- Max line: 100 (rustfmt default).
- Strict clippy (enforced).
- Prefer Rust idioms over literal C++ ports. When in doubt, ask.

## Workflow

- Plan first. Tests before prod code (TDD). Lint changed files.
- Every `codegen.rs` must have a `#[cfg(test)] mod tests` block. Use `parse()` + `codegen().to_string().contains(...)` to cover each emit helper and its key branches.
- `.gitignore` (not `.arcignore`).

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
**Escalated?** no | AGENTS.md | skill:[name] | hook
```

Categories: `code-style` | `process` | `architecture` | `testing` | `search` | `other`

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
