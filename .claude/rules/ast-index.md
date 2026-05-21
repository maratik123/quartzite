# ast-index Rules

Teach Claude Code to prefer `ast-index` over `grep` / bulk `Read`, outline
before reading large files, and pass the same instructions to any subagent
it spawns for code search.

## Index hygiene

The session-start hook in `.claude/settings.json` runs `ast-index update`
(or `rebuild` if no index exists) every time a session opens, and the
post-`git commit` hook refreshes the index after each commit. **Manual
`ast-index update` is rarely needed** — only run it explicitly after a
large external file change you want indexed before the next search (e.g.
right after `git pull` or a checkout, mid-session).

The project lives in a single workspace root — no `add-root` configuration
is required.

## Mandatory search rules

1. **ALWAYS use `ast-index` FIRST** for any code-search task. This matches
   the workspace-wide search-tool preference `ast-index > rg > grep`.
2. **NEVER duplicate results** — if `ast-index` returned hits, that IS the
   complete answer. Do not re-run `grep` / `rg` to "double-check".
3. Use the `Grep` tool **only when** `ast-index` returned empty, or for
   regex / string-literal patterns that are not symbol names.

## Mandatory read rules

1. **Before `Read`-ing any file over 500 lines, run `ast-index outline
   <file>` first.**
2. Use the outline to locate the specific symbol / line range you need,
   then `Read` that slice via `offset` / `limit`.
3. Never bulk-read large files without an outline — it wastes context and
   produces worse answers.

## Rules for subagents

When you spawn a subagent for code search (via the `Agent` tool), the
subagent does **not** inherit this file. Include the block below verbatim
in the subagent's prompt:

```
Use `ast-index` via Bash for code search (NOT grep / the Grep tool):
  ast-index search "query"           — universal search
  ast-index file "Name"              — find a file by name fragment
  ast-index symbol "Name"            — find a symbol definition
  ast-index class "Name"             — find a class / trait / struct / enum
  ast-index usages "Name"            — every usage of a symbol
  ast-index callers "func"           — functions that call this one
  ast-index implementations "Trait"  — concrete implementors of a trait
  ast-index refs "Name"              — cross-references (defs + imports + usages)
Use Grep ONLY if ast-index returned empty.

Before Read-ing any file over 500 lines, FIRST run
  ast-index outline <file>
to get its structure, then Read only the targeted slice via offset/limit.
Never bulk-read large files.
```

The `Explore` and `general-purpose` agents are the primary search agents in
this project — give them the block above whenever they're tasked with
locating code.

## Command cheat sheet

Grouped by intent. Full list and flags: `ast-index --help`.

- **Search:** `search`, `file`, `symbol`, `class`
- **Usages & flow:** `usages`, `callers`, `call-tree`, `refs`
- **Hierarchy:** `implementations`, `hierarchy`, `extensions`
- **Modules / deps:** `module`, `deps`, `dependents`, `api`, `unused-deps`
- **Files:** `outline`, `imports`, `changed`
- **Quality:** `todo`, `deprecated`, `unused-symbols`
- **Index mgmt:** `rebuild`, `update`, `stats`

## Common use cases

Real Quartzite symbols — the agent makes better choices when it has
concrete precedents to pattern-match against.

- `ast-index usages "Palette"` — every place `Palette` is referenced
  (constructors, fields, `&Palette` params, tests).
- `ast-index implementations "Style"` — every concrete type that
  implements the `Style` trait (e.g. `DefaultStyle`).
- `ast-index callers "paint"` — who calls a `paint` method, without the
  noise of definition lines.
- `ast-index call-tree "draw" -d 3` — transitive caller tree up to depth
  3. Use when tracing a paint bug back to its widget entry point.
- `ast-index symbol "ColorRole"` — definition site of the `ColorRole`
  enum (`quartzite-style-types/src/color_role.rs`).
- `ast-index deps "quartzite-style"` — what `quartzite-style` depends on.
- `ast-index dependents "quartzite-paint-api"` — what depends on
  `quartzite-paint-api` (useful before a breaking change).
- `ast-index changed` — symbols modified in your current branch vs
  `master`. Great for "what am I actually changing?" PR-description
  summaries.
- `ast-index outline quartzite-style/src/default_style.rs` — structure of
  a single file before reading it.
- `ast-index todo` — all TODO / FIXME / HACK comments, grouped.
- `ast-index deprecated` — every use of `#[deprecated]` across the
  workspace.

## Scoping searches

All symbol-returning commands accept scope filters — use them to kill
noise across the workspace:

```bash
ast-index usages "Palette" --module quartzite-style    # within one crate
ast-index search "paint" --in-file default_style.rs    # within one file
ast-index symbol "Style" --type class                  # class-kind only
```

## When `ast-index` returns empty

Legitimate reasons:

- Symbol genuinely doesn't exist in the workspace.
- Index is stale — run `ast-index update` and retry (rare; the
  session-start + post-commit hooks normally keep it fresh).
- Symbol is generated by a macro (`derive`, `tracing::*_span!`,
  `thiserror`, `rstest`, etc.) — `ast-index` does not expand macros. Fall
  back to `rg` / `Grep` for the macro invocation.
- You're searching for a string literal, not a symbol — use `rg` /
  `Grep`.

Do **not** fall back to bulk `Read` of files in these cases. Use `rg` /
`Grep` with a specific pattern.
