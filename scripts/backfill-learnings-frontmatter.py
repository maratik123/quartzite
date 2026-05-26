#!/usr/bin/env python3
"""
Backfill YAML frontmatter on `ai-docs/learnings.md`.

Authorising spec/design:
- `ai-docs/plans/2026-05-26-frontmatter-long-running-files.spec.md`
- `ai-docs/plans/2026-05-26-frontmatter-long-running-files.design.md`
- Tracking issue: #575

Behaviour summary
-----------------
- Emits file-level frontmatter (a `---` … `---` block) at lines 1-4 if not
  already present:

      ---
      schema_version: 1
      kind: learnings
      ---

- For every `### YYYY-MM-DD — …` entry heading, emits a fenced
  ```yaml … ``` block IMMEDIATELY above the heading (no blank line between
  the closing fence and the `###`) carrying:

      escalated: "<rhs of **Escalated?** body line, whitespace-trimmed>"
      kind:      "<rhs of **Kind:** body line, whitespace-trimmed>"
      superseded_by: "<rhs of **Superseded by:** body line, trimmed>"   # only when present

  Defaults when the bold-key body line is absent:
      escalated: "no"      (mirrors AGENTS.md default-when-omitted)
      kind:      "correction"  (mirrors AGENTS.md default-when-omitted)

  YAML values are emitted as double-quoted scalars; embedded `"` is escaped
  as `\\"`. Backticks pass through verbatim (YAML double-quoted scalars
  treat them literally).

Idempotence (two independent layers)
------------------------------------
1. File-level FM: emitted only when lines 1-4 are NOT already a valid
   ``---`` / ``schema_version`` / ``kind`` / ``---`` block whose YAML parses
   to a dict carrying both `schema_version` AND `kind`.
2. Per-entry FM: emitted only above headings that don't already have a
   fenced ```yaml`` block immediately above whose body parses to a dict
   carrying BOTH `escalated:` AND `kind:` keys.

The two layers are checked independently — a partial-migration recovery
(e.g. file-level FM already present but a freshly-appended entry lacks
its preamble; or vice versa) completes the missing layer without
rewriting the present one.

Fail-loud posture
-----------------
Every emitted fenced block is re-fed through `yaml.safe_load`. Any parse
failure or schema-floor deviation (missing `escalated:` or `kind:` key in
the parsed dict, or values not being strings) raises SystemExit(1) with a
diagnostic naming the offending heading.

CLI
---
  python3 scripts/backfill-learnings-frontmatter.py            # apply changes
  python3 scripts/backfill-learnings-frontmatter.py --check    # dry-run; exit 0 iff zero changes

Stdlib only, plus PyYAML (hard requirement; the script exits 1 with a
`pip install PyYAML` hint if `import yaml` fails).
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - environment guard
    sys.stderr.write(
        "ERROR: PyYAML is required.\n"
        "Install it with: pip install PyYAML\n"
    )
    raise SystemExit(1)


LEARNINGS_PATH = Path("ai-docs/learnings.md")

FILE_LEVEL_FM = "---\nschema_version: 1\nkind: learnings\n---\n\n"

HEADING_RE = re.compile(r"^### (\d{4}-\d{2}-\d{2}) — ")
ESCALATED_RE = re.compile(r"^\*\*Escalated\?\*\*\s*(.*)$")
KIND_RE = re.compile(r"^\*\*Kind:\*\*\s*(.*)$")
SUPERSEDED_RE = re.compile(r"^\*\*Superseded by:\*\*\s*(.*)$")

# Detect an existing top-of-file YAML FM block delimited by `---` lines.
FILE_FM_RE = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)

# Detect a fenced ```yaml ... ``` block that sits IMMEDIATELY above a
# `### YYYY-MM-DD — …` heading (no blank line between the closing fence
# and the heading). The closing fence may end the file iff a heading
# follows on the next line; this matches the spec's "no blank line"
# requirement.
ENTRY_FM_RE = re.compile(
    r"```yaml\n(?P<body>.*?)\n```\n(?=### \d{4}-\d{2}-\d{2} — )",
    re.DOTALL,
)


def yaml_quote(value: str) -> str:
    """Render a Python string as a double-quoted YAML scalar.

    Escapes embedded `"` as `\\"` and embedded `\\` as `\\\\`. Backticks,
    parentheses, commas, and whitespace pass through verbatim — YAML
    double-quoted scalars treat them literally.
    """
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def parse_entries(raw: str) -> list[dict]:
    """Walk `raw` markdown body, return one record per `### YYYY-MM-DD —` heading.

    Each record:
      - 'heading_line':       1-based line number of the `###` line
      - 'heading_offset':     byte offset of the `###` line's first char
      - 'date':               'YYYY-MM-DD'
      - 'escalated':          string (default "no" when bold-key line absent)
      - 'kind':               string (default "correction" when bold-key line absent)
      - 'superseded_by':      string OR None (None when bold-key line absent)
      - 'has_preamble':       bool — fenced ```yaml block exists immediately above
                              this heading AND its parse carries BOTH
                              `escalated:` AND `kind:` keys
    """
    lines = raw.splitlines(keepends=True)
    n_lines = len(lines)

    # Precompute byte offsets of each line start.
    offsets = [0] * (n_lines + 1)
    for i, line in enumerate(lines):
        offsets[i + 1] = offsets[i] + len(line)

    # Index heading line numbers (1-based).
    heading_indices: list[int] = []
    for idx, line in enumerate(lines):
        if HEADING_RE.match(line):
            heading_indices.append(idx)

    entries: list[dict] = []
    for h_idx, line_idx in enumerate(heading_indices):
        heading_line = lines[line_idx]
        match = HEADING_RE.match(heading_line)
        assert match is not None
        date = match.group(1)

        # Body window: from the line AFTER this heading to the line BEFORE
        # the next heading (or end of file).
        body_start = line_idx + 1
        body_end = heading_indices[h_idx + 1] if h_idx + 1 < len(heading_indices) else n_lines

        escalated_value: str | None = None
        kind_value: str | None = None
        superseded_value: str | None = None
        for body_idx in range(body_start, body_end):
            body_line = lines[body_idx].rstrip("\n")
            if escalated_value is None:
                m_esc = ESCALATED_RE.match(body_line)
                if m_esc is not None:
                    escalated_value = m_esc.group(1).strip()
                    continue
            if kind_value is None:
                m_kind = KIND_RE.match(body_line)
                if m_kind is not None:
                    kind_value = m_kind.group(1).strip()
                    continue
            if superseded_value is None:
                m_sup = SUPERSEDED_RE.match(body_line)
                if m_sup is not None:
                    superseded_value = m_sup.group(1).strip()
                    continue

        # Detect existing preamble: a fenced ```yaml ... ``` block whose
        # closing ``` is immediately followed by the `###` heading line.
        # We walk back from line_idx-1 looking for a closing ``` fence,
        # then back to its matching opening ```yaml.
        has_preamble = False
        if line_idx >= 2:
            close_idx = line_idx - 1
            if lines[close_idx].rstrip("\n") == "```":
                # Find opening fence ```yaml.
                open_idx = None
                for back in range(close_idx - 1, max(-1, close_idx - 200), -1):
                    candidate = lines[back].rstrip("\n")
                    if candidate == "```yaml":
                        open_idx = back
                        break
                    if candidate == "```":
                        # Found a different closing fence first → bail.
                        break
                if open_idx is not None:
                    body_text = "".join(lines[open_idx + 1:close_idx])
                    try:
                        parsed = yaml.safe_load(body_text)
                    except yaml.YAMLError:
                        parsed = None
                    if (
                        isinstance(parsed, dict)
                        and "escalated" in parsed
                        and "kind" in parsed
                    ):
                        has_preamble = True

        entries.append({
            "heading_line": line_idx + 1,
            "heading_offset": offsets[line_idx],
            "date": date,
            "escalated": escalated_value if escalated_value is not None else "no",
            "kind": kind_value if kind_value is not None else "correction",
            "superseded_by": superseded_value,
            "has_preamble": has_preamble,
        })

    return entries


def render_preamble(entry: dict) -> str:
    """Render a fenced ```yaml ... ``` block for one entry.

    Returned text ends with `\n` after the closing fence so it slots
    immediately above the `###` heading with no blank line in between.
    """
    lines = [
        "```yaml",
        f"escalated: {yaml_quote(entry['escalated'])}",
        f"kind: {yaml_quote(entry['kind'])}",
    ]
    if entry["superseded_by"] is not None:
        lines.append(f"superseded_by: {yaml_quote(entry['superseded_by'])}")
    lines.append("```")
    return "\n".join(lines) + "\n"


def verify_block_parses(block_text: str, heading_label: str) -> None:
    """Re-feed an emitted block through yaml.safe_load and check the floor.

    Exits 1 with a diagnostic naming `heading_label` on any parse failure
    or schema-floor deviation.
    """
    # The block we emit is the FULL fenced wrapper; extract the body for
    # safe_load.
    match = re.match(r"```yaml\n(.*?)\n```\n\Z", block_text, re.DOTALL)
    if match is None:
        sys.stderr.write(
            f"ERROR: emitted block for entry '{heading_label}' is not a "
            f"valid fenced ```yaml … ``` wrapper:\n{block_text!r}\n"
        )
        raise SystemExit(1)
    body = match.group(1)
    try:
        parsed = yaml.safe_load(body)
    except yaml.YAMLError as exc:
        sys.stderr.write(
            f"ERROR: emitted YAML block for entry '{heading_label}' failed "
            f"to parse: {exc}\n--- block body ---\n{body}\n"
        )
        raise SystemExit(1)
    if not isinstance(parsed, dict):
        sys.stderr.write(
            f"ERROR: emitted YAML block for entry '{heading_label}' did not "
            f"parse to a dict (got {type(parsed).__name__}).\n--- body ---\n{body}\n"
        )
        raise SystemExit(1)
    for required in ("escalated", "kind"):
        if required not in parsed:
            sys.stderr.write(
                f"ERROR: emitted YAML block for entry '{heading_label}' is "
                f"missing required key '{required}'.\n--- body ---\n{body}\n"
            )
            raise SystemExit(1)
        if not isinstance(parsed[required], str):
            sys.stderr.write(
                f"ERROR: emitted YAML block for entry '{heading_label}' key "
                f"'{required}' is not a string (got "
                f"{type(parsed[required]).__name__}).\n--- body ---\n{body}\n"
            )
            raise SystemExit(1)


def has_valid_file_level_fm(raw: str) -> bool:
    """Detect a valid top-of-file FM block carrying schema_version + kind."""
    match = FILE_FM_RE.match(raw)
    if match is None:
        return False
    try:
        parsed = yaml.safe_load(match.group(1))
    except yaml.YAMLError:
        return False
    return (
        isinstance(parsed, dict)
        and "schema_version" in parsed
        and "kind" in parsed
    )


def apply_migration(raw: str) -> tuple[str, int, int]:
    """Return (new_raw, file_fm_emitted, per_entry_blocks_emitted).

    file_fm_emitted is 0 or 1.
    per_entry_blocks_emitted counts headings that received a fresh preamble.
    """
    file_fm_emitted = 0
    new_raw = raw

    if not has_valid_file_level_fm(new_raw):
        new_raw = FILE_LEVEL_FM + new_raw
        file_fm_emitted = 1

    # Re-parse entries against the (possibly file-FM-prefixed) text so the
    # heading_offset values are correct for splicing.
    entries = parse_entries(new_raw)

    # Build the new text by walking the headings in order and emitting a
    # preamble before each one that lacks `has_preamble`.
    chunks: list[str] = []
    cursor = 0
    per_entry_emitted = 0
    for entry in entries:
        if entry["has_preamble"]:
            continue
        chunks.append(new_raw[cursor:entry["heading_offset"]])
        block = render_preamble(entry)
        verify_block_parses(block, f"{entry['date']} (line {entry['heading_line']})")
        chunks.append(block)
        cursor = entry["heading_offset"]
        per_entry_emitted += 1
    chunks.append(new_raw[cursor:])
    new_raw = "".join(chunks)

    return new_raw, file_fm_emitted, per_entry_emitted


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Backfill YAML frontmatter on ai-docs/learnings.md. "
            "Idempotent at file-level + per-entry layers (checked independently)."
        ),
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help=(
            "Dry-run mode: write no changes. Exit 0 iff zero changes would be "
            "emitted (i.e. the file is already fully migrated)."
        ),
    )
    parser.add_argument(
        "--path",
        default=str(LEARNINGS_PATH),
        help=(
            "Path to learnings.md (default: ai-docs/learnings.md, relative "
            "to the current working directory)."
        ),
    )
    args = parser.parse_args(argv)

    path = Path(args.path)
    if not path.is_file():
        sys.stderr.write(f"ERROR: file not found: {path}\n")
        return 1

    raw = path.read_text(encoding="utf-8")

    new_raw, file_fm_emitted, per_entry_emitted = apply_migration(raw)
    total_changes = file_fm_emitted + per_entry_emitted

    if args.check:
        if total_changes == 0:
            sys.stdout.write(
                f"OK: {path} is already fully migrated "
                f"(0 file-level FM emits + 0 per-entry FM emits).\n"
            )
            return 0
        sys.stdout.write(
            f"CHANGES NEEDED: {path} would receive "
            f"{file_fm_emitted} file-level FM emit + "
            f"{per_entry_emitted} per-entry FM emit(s).\n"
        )
        return 1

    if total_changes == 0:
        sys.stdout.write(
            f"OK: {path} is already fully migrated; no changes written.\n"
        )
        return 0

    path.write_text(new_raw, encoding="utf-8")
    sys.stdout.write(
        f"WROTE: {path} ({file_fm_emitted} file-level FM emit + "
        f"{per_entry_emitted} per-entry FM emit(s)).\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
