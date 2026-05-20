# Design: Sync code to design-system colour-value updates

**Issue:** #514
**Date:** 2026-05-21

## Approach

The spec's per-slot audit table (light + dark, 11 roles each) records **zero
colour-value mismatches** between the Rust workspace and the post-refresh
`design-system/` folder. AC1 is therefore satisfied **on disk** by the spec
itself; no Rust source edit is required to assert it.

The only artefact that needs touching is a single stale path in a module-level
doc comment at `quartzite-style-types/src/dark_palette.rs:6`. The doc cites
`design-system/colors_and_type.css [data-theme="dark"]` — a file that no longer
exists in `design-system/` (`find design-system -name '*.css'` returns only
`preview/card-base.css` and `ui_kits/widgets/kit.css`). Per the spec's
**Key decisions** row, the replacement reference is
`design-system/README.md § Dark theme`, the canonical narration of the dark
palette that survived the refresh and lists every dark hex.

**Chosen solution:** delete line 6 of `dark_palette.rs` entirely (the stale
`design-system/colors_and_type.css ...` clause), collapsing the two-source list
the module-doc currently maintains to the single surviving source already on
line 5 (`design-system/README.md § *Dark theme*`). One-line deletion plus a
4-character strip on line 5 (remove the trailing ` and` connector so the
existing period on the README reference closes the sentence). No prose content
change beyond that connector strip; no semantic change.

**Interpretation of spec AC2 ("Replace `…colors_and_type.css …` with
`…README.md § Dark theme`."):** the surviving line 5 *is* the
`design-system/README.md § Dark theme` reference — already present in the
module-doc before this edit. AC2 is satisfied because, after the edit, the
doc-comment references **only** the canonical surviving source. A reader of
AC2 expecting a literal in-place text insertion will not see one in the diff;
the README reference was already there, and the work is to remove the stale
sibling that pointed at the deleted file. See § Rejected alternatives row 1
for why in-place duplication was rejected.

### Rejected alternatives

1. **Replace the stale path in place with the README path** — would leave two
   adjacent lines both pointing at the same artefact (`design-system/README.md
   § Dark theme`), which is pointless duplication. Delete is the cleaner shape.
2. **Cite `design-system/preview/dark-palette-compare.html` instead of the
   README** — the spec's *Open questions* row left this as a reviewer flip
   option. README is the better citation for "the values come from here" (it
   spells them out); the HTML is a rendered swatch. Spec's Key decisions
   already chose README; deferring to that.
3. **Land the audit observation in a new comment block on `palette.rs`** — the
   audit is documentation-only; the spec already records it. Adding a
   commentary block to source files would re-introduce design-system content
   into Rust, working against the pointer-only contract. Spec table is the
   right home.
4. **Bundle removal of the also-stale `colors_and_type.css` mention in
   `AGENTS.md § Design system` (line 13) and `.claude/agents/design.md` line 15
   and `.claude/agents/design-review.md` lines 26/34** — out of scope. The spec
   explicitly limits the fix to `dark_palette.rs:6`. The agent-prompt mentions
   are a separate concern that affects every `/task` invocation's design
   bootstrap and should be tracked as its own follow-up. Recorded under § Open
   questions, not folded into this design.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Remove the stale `design-system/colors_and_type.css [data-theme="dark"]` clause from `dark_palette.rs:6`; re-flow the module-doc so the surviving `design-system/README.md § *Dark theme*` reference reads cleanly. Then run the full local gate suite (build / test / clippy / fmt --check / doc with `-D warnings -D missing-docs` / no-default-features-with-libm build) and verify zero snapshot regen by inspecting `git status quartzite-style/tests/snapshots/`. | `quartzite-style-types/src/dark_palette.rs` | — |

**Subtask 1 — exact edit site (file:line + before/after text)**

File: `quartzite-style-types/src/dark_palette.rs`

Before (lines 1–6, inclusive):

```rust
//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose eleven
//! [`ColorRole`] slots are seeded from the dark-theme values defined in
//! `design-system/README.md` § *Dark theme* and
//! `design-system/colors_and_type.css` `[data-theme="dark"]`.
```

After (lines 1–5, inclusive — file shortens by one line):

```rust
//! Dark-theme [`Palette`] constant for quartzite.
//!
//! Exports [`DARK_PALETTE`], a compile-time [`Palette`] whose eleven
//! [`ColorRole`] slots are seeded from the dark-theme values defined in
//! `design-system/README.md` § *Dark theme*.
```

The sentence is grammatically complete after the deletion (the trailing
`and` clause that pointed at the deleted file is removed; the period on the
README reference closes the sentence). No other line in the file references
either path.

**Gate cadence for Subtask 1 (single group, single subtask):**

After the edit, run the full local gate suite before commit per AGENTS.md
§ Build & Test:

```
cargo build
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt -- --check
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features
cargo build -p quartzite --no-default-features --features libm
```

All must exit 0 (AC3). Then confirm `git status quartzite-style/tests/snapshots/`
is empty (AC4) and `git diff --stat` on the working tree shows exactly one file
changed in `quartzite-style-types/src/dark_palette.rs` (AC5 — proxy for "no
public-API change", since the net effect — line 6 removed plus a 4-character
strip on line 5 to re-close the sentence — is confined to a module-level `//!`
doc comment, which cannot reach `pub` API).

## Handoff plan

`M = 1`. One group, terminal.

- **Group A:** subtask 1 — terminal group (1 subtask; within the 1..=3 range).
  Entry into Group A spawns `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § *Compaction recovery (re-entry)* — the single group completes Step 8 in its
  own `/context-reset` subagent. No handoff between groups (there are no later
  groups).

## Risks

- **Risk:** `cargo doc` with `-D warnings -D rustdoc::broken_intra_doc_links`
  rejects the edited module-doc.
  **Mitigation:** the deleted clause is a free-text mention (a Markdown
  inline-code path), never an intra-doc link, so removing it cannot break a
  link target. The surviving line 5 reference is also free-text. `cargo doc`
  gate run in Subtask 1 confirms.
- **Risk:** an editor (rustfmt) re-wraps the doc comment in an unexpected way.
  **Mitigation:** `cargo fmt` does not reflow doc-comment prose (it touches
  whitespace inside Rust syntax, not the contents of `///` or `//!` lines).
  The `cargo fmt -- --check` gate confirms no unintended reformat.
- **Risk:** subsequent reader thinks "only README cited" implies the swatch
  HTML demos are not authoritative — minor.
  **Mitigation:** README is canonical narration per spec § Key decisions; the
  swatch HTML is rendered demo, not a separate source of truth. Spec records
  the rationale; no in-code annotation needed.
- **Risk:** A future palette refresh re-introduces `colors_and_type.css` (or a
  similarly-named CSS source-of-truth file) and the doc-comment is now
  one-source where it should be two.
  **Mitigation:** acceptable. If a future refresh adds a new authoritative
  source, that PR will update `dark_palette.rs:5` to enumerate both. Today's
  tree has exactly one extant source; the doc-comment should reflect that.
- **Risk:** unrelated stale `colors_and_type.css` mentions in `AGENTS.md` /
  `.claude/agents/design.md` / `.claude/agents/design-review.md` will remain
  after this PR lands.
  **Mitigation:** out of scope per spec — recorded under § Open questions and
  in spec § Deferred so a follow-up issue can track it. The fix here is the
  one stale ref called out in the audit; the agent-prompt cleanup is a
  cross-cutting documentation sweep with different risk profile (touches
  Propagation Rule sync groups, affects every `/task`'s design bootstrap, may
  also want to revisit the trigger-conditions list in `AGENTS.md § Design
  system`).

## Test Design

No new tests. Per spec § Key decisions row 5: "A test asserting
`DARK_PALETTE.color(Highlight) == Color::DODGER_BLUE` adds no information
over the source itself, and any future drift between `Color::DODGER_BLUE`'s
numeric value and its `#1e90ffff` doc comment is caught by the existing
doctest convention." The doc-comment edit is text-only on a module-level
`//!` block; there is no behaviour to test.

Gate-level test execution (which exercises the existing test suite to confirm
no incidental regression) is captured under Subtask 1's gate cadence (`cargo
test --workspace`).

## Open questions

- **Should the also-stale `colors_and_type.css` mentions in `AGENTS.md` line 13,
  `.claude/agents/design.md` line 15, `.claude/agents/design-review.md` lines
  26/34, and `design-system/SKILL.md` lines 7/15 be cleaned up in a follow-up
  PR?** Out of scope here; spec explicitly limits the fix to
  `dark_palette.rs:6`. Recording the question so it can be tracked as a
  separate issue. Note: `design-system/SKILL.md`, `design-system/README.md`,
  `design-system/proposals/text-edit-read-only-overlay.md`, and
  `design-system/fonts/README.md` are all design-system-internal and would
  need a parallel `design-system/` PR (the design-system folder is the source
  of truth; `design-system/palette-state-groups.proposal.md` still cites
  `colors_and_type.css` because the proposal predates the file's removal —
  may want to update the proposal too, or close it as superseded once #402
  designs the replacement).
- **Confirm `design-system/README.md § Dark theme` over
  `design-system/preview/dark-palette-compare.html` as the doc-comment
  citation.** Spec § Key decisions row 2 already chose README; spec § Open
  questions row 2 left it as a reviewer flip. Going with README per Key
  decisions unless the reviewer flips it during design-review.
