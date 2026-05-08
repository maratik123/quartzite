# Agent Writing Style

**Audience:** anyone editing instruction files (`AGENTS.md`,
`.claude/skills/**`, `.claude/agents/**`, `ai-docs/code-style.md`,
`ai-docs/doc-convention.md`).

**Goal:** write binary rules so that both Opus 4.7 and Sonnet 4.6 reading the
same paragraph land on the same interpretation.

> **AXIOM — Apply this style only to binary rules.**
> Visual fail-loud emphasis is a tool for unambiguous rules where misreads
> have caused real failures. Applying it to ordinary prose devalues every
> emphasis token in the file — readers tune out the YELLED RULES because
> they're everywhere.
>
> | If a paragraph is... | Style |
> |---|---|
> | A binary rule (do X / never do Y / exactly one of A or B) | Fail-loud (patterns below) |
> | A documented misread spot (`Escalated?` target in `learnings.md`) | Fail-loud |
> | A multi-axis decision tree | Fail-loud + action table |
> | Rationale, context, examples, links | Ordinary prose |

## Patterns

### 1. AXIOM blockquote (top-of-section)

For sections governed by one binary rule:

> **AXIOM — \<name\>.**
> \<single-sentence rule.\> \<optional why-it-matters sentence.\>
>
> | If you see... | Action |
> |---|---|
> | \<condition\> | \<action\> |

Place immediately after the `## heading`, before any nuanced rules. The
action table is what makes the rule mechanical.

Example: `ai-docs/code-style.md § #[inline] and _Simple._`.

### 2. Fail-loud verbs (bold uppercase)

| Use... | For |
|---|---|
| **NEVER** / **MUST NOT** / **FORBIDDEN** | Prohibitions |
| **ALWAYS** / **MUST** | Required actions |
| **STOP** | Halt-the-workflow signals |
| **REJECT** | Code-review verdicts |
| **REMOVE** / **REPLACE** / **DELETE** | Corrective actions |

At most one bold-uppercase verb per paragraph. If a paragraph carries three,
it is no longer a rule — it is noise.

### 3. Action tables ("if you see X → do Y")

For mechanical decisions:

| If you see... | Action |
|---|---|
| `#[inline]` AND `_Simple._` on the same fn | **REMOVE** `_Simple._`, keep `#[inline]` |

Left column lists conditions an editor or reviewer would recognise (visible
markers, file paths, error messages). Right column lists the action verbatim.
Do not bury the action in prose around the table.

### 4. Explicit file lists, never globs

In fail-loud blocks, enumerate each path:

> When writing to `ai-docs/learnings.md`, you **MUST NOT** also edit any of
> these files in the same turn:
>
> - `AGENTS.md`
> - `CLAUDE.md`
> - `.claude/skills/**` (any file under this directory)
> - `.claude/agents/**` (any file under this directory)
> - `.claude/settings.json`
> - `ai-docs/code-style.md`
> - `ai-docs/doc-convention.md`

A glob inside one bullet (with parenthetical "any file under this directory")
is fine. A glob *as the entire list* is not — readers expand globs
differently.

### 5. Numbered enumeration of triggers

For "this fires when..." or "this is authorised by...":

> Project-level escalation happens only when:
>
> 1. The user runs `/improve`, OR
> 2. The user explicitly asks ("escalate this", "update AGENTS.md", etc.).

State the OR/AND connector at the end of each item, consistently. Consistent
placement defeats nesting ambiguity.

### 6. Concrete do/not examples

For non-trivial rules, show both shapes:

> **Do this:**
> ```rust
> let g = mutex.lock().unwrap_or_else(|e| e.into_inner());
> ```
>
> **NOT this:**
> ```rust
> let g = mutex.lock().expect("mutex poisoned");  // .expect() panics on poison
> ```

Examples remove ambiguity faster than prose.

## Writing checklist

Before submitting a rule paragraph, check:

- [ ] Single declarative sentence? (Three nested clauses → split.)
- [ ] Verb at the start? ("Append to..." beats "It is the case that...")
- [ ] Every condition explicit? (No "usually" / "the obvious case".)
- [ ] File paths spelled out? (Not "the usual file".)
- [ ] Says what to DO, not only what NOT to do?
- [ ] Action verb present? (Not just "this is forbidden".)

## Anti-patterns

| Anti-pattern | Why bad | Fix |
|---|---|---|
| Every paragraph in caps | Emphasis devalued | Reserve fail-loud for binary rules |
| AXIOM blockquote without action table | States rule but not application | Always include "if X → do Y" table |
| Implicit conditions ("usually") | Models infer differently | State explicitly |
| Globs as the entire fail-loud list | Reader expansion varies | Enumerate files |
| Rule + rationale + example + edge case in one paragraph | Rule lost in prose | Split: rule (fail-loud), rationale (prose), example (do/not), edge case (sub-bullet) |
| Negative-only rules ("don't X") | Reader doesn't know correct action | Pair with "do Y instead" |

## Citation in PRs

When a PR adds or modifies a fail-loud section, cite this doc in the body:

> Per `ai-docs/agent-writing-style.md` § Pattern 1, this PR adds an AXIOM
> blockquote to `<file> § <section>`.

Citing this doc grounds the stylistic choice in the agreed convention rather
than per-PR taste.

## Out of scope

This file does not govern:

- Files for Opus-only readers: agents with `model: opus` frontmatter
  (`design`, `design-review`, `learnings-escalation-audit`, `self-improve`)
  and Opus-mode skills (`/ai-audit`, `/improve`)
- Rust source code documentation (covered by `ai-docs/doc-convention.md`)
- Project context (covered by `ai-docs/context.md`)

For those readers, AXIOM/fail-loud styling is optional — the reader has the
context to disambiguate without it.
