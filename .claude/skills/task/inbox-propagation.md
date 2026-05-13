# Inbox propagation — parser rules and per-row mapping

Reference for `.claude/skills/task/SKILL.md` Step 12 sub-step 4. Walks one or more spec/design files, locates each of the three target headings (`## Out of scope` / `## Deferred` / `## Open questions`, exact h2 match anchored on `^## <Heading>$`), classifies the body shape using the six ordered rules below, and emits one row to `ai-docs/deferred/_inbox.md` per parsed item. The rules are ordered — the first match wins; rule ordering matters (pipe-bullet and bold-bullet shapes take precedence over plain bullets). For mixed-shape bodies (rare; a single section containing two shapes), classify per line using the same ordering — every line goes through the same six rules.

**Shape rule 1 — NONE sentinel.** Body matches one of: `_None._` / `None.` / `None`; `(none — …)` / `(none -- …)` / `_(none)_`; `None — …` / `None - …` (the word `None` followed by an em-dash or hyphen and prose); `None at spec time.` / `None blocking …`. Matcher: collapse whitespace, lower-case, strip wrapping `_` and `()`, then test `^none\b.*$` with no `- ` bullet lines and no `|` table pipes. **Emit zero rows; emit zero warnings.** The only silent case.

**Shape rule 2 — TABLE.** Section body's first non-blank line starts with `|` and is followed by a `|---|...|` separator line. Per-row extraction: skip header + separator; for each data row `| C1 | C2 | C3? | ... |`, `Item` = `C1` (verbatim, including markdown formatting); if C2 is non-empty, append ` — <C2>` to `Item` (carries rationale into the inbox row). Subsequent columns are ignored. Terminate row collection on first blank line or first `^## ` heading.

**Shape rule 3 — PIPEBULLET3.** Bullet lines match `^- (.+?) \| (.+?) \| (.+?)$` — three `|`-separated fields. Per-row extraction: `Item` = field 1 + ` — ` + field 2 (the third field is metadata such as "Separate issue needed?" and is dropped).

**Shape rule 4 — PIPEBULLET2.** Bullet lines match `^- (.+?) \| (.+?)$` — two `|`-separated fields. Per-row extraction: `Item` = field 1 + ` — ` + field 2.

**Shape rule 5 — BOLDBULLET.** Bullet lines match `^- \*\*([^*]+)\*\*\s*[—-]\s*(.+)$` — a bolded leading term followed by an em-dash or hyphen and prose. Per-row extraction: `Item` = `**<term>**` + ` — ` + `<prose>` (the leading bold survives into the inbox so the term/explanation split is preserved).

**Shape rule 6 — PLAINBULLET.** Bullet lines match `^- (.+)$` and do NOT match rules 3 / 4 / 5 above. Per-row extraction: `Item` = the entire bullet text, verbatim. **Multi-line continuation:** lines starting with 2+ spaces of indentation and no leading `- ` are joined into the preceding bullet with a single space — common in wrapped real-corpus bullets.

**Unrecognised shape.** A section with a non-blank body that matches none of the six rules emits one stdout warning and zero rows:

```
WARN: <spec-path> :: <section heading> — unrecognised body shape, no rows emitted
```

Step 12 continues normally; `_inbox.md` is unchanged for that section.

**Per-row mapping.** Each parsed item becomes a 4-cell `_inbox.md` row:

```
| <Item> | [<source-label>](<source-path>) | <section-key> | — |
```

- **`<Item>`** — value produced by the matched shape rule. If `<Item>` contains a literal `|`, escape it as `\|` (markdown table convention).
- **`<source-label>`** — derived from the source filename: strip the `YYYY-MM-DD-` date prefix and the `.spec.md` / `.design.md` suffix; append ` spec` or ` design` accordingly. Examples: `2026-05-09-paint-style.spec.md` → `paint-style spec`; `2026-05-01-auto-connection.design.md` → `auto-connection design`.
- **`<source-path>`** — `../plans/done/<filename>` (relative to `_inbox.md`'s location in `ai-docs/deferred/`).
- **`<section-key>`** — one of three literal tokens: `## Out of scope` → `out-of-scope`; `## Deferred` → `deferred`; `## Open questions` → `open-question` (singular `open-question`, NOT `open-questions`, matches the AGENTS.md AXIOM table cell text).
- **`—`** — the literal em-dash, identical to the un-triaged marker used in the 8 thematic files' `Tracked` column. Cell-4-`Tracked` invariant honoured.

**Dedupe rule (file-level).** Before running the parser, build the set `H` = every `Source`-cell relative path appearing in any `^|` row of the 8 thematic files (`ai-docs/deferred/{signals-slots,properties,macros-codegen,object-tree,threading-runtime,future-crates,ci-docs-workflow,python}.md`). Normalisation: strip trailing `/`, strip `#...` anchor fragments, strip leading `./`, strip leading/trailing whitespace; preserve case. For each candidate row, if its `Source` path is in `H`, skip the **entire file** (all of its sections) — file-level dedupe avoids re-harvesting any portion of a file whose other sections were already drained into thematic files by the manual extraction passes. `widget-backlog.md` is NOT in `H` (its rows are tracked via the `Notes` cell, not via thematic-file membership).
