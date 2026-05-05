# Doc convention

**Source:** issue #80
**Date:** 2026-05-05
**Tracked in:** #80

## References

Source material for the convention (carried over from issue #80):

- [Rust by Example — Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)
- [RFC 1574 — More API documentation conventions](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text)
- [The Rust Book — Making useful documentation comments](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments)
- [Pascal Hertleif — Machine-readable inline Markdown code documentation](https://deterministic.space/machine-readable-inline-markdown-code-cocumentation.html)

## Scope

1. Adopt a workspace-wide documentation convention based on RFC 1574 + Rust
   book ch14-02 + deterministic.space machine-readable inline-markdown convention.
2. Author canonical convention document at `ai-docs/doc-convention.md`
   (must cite the four references above).
3. Add a short "Documentation Conventions" pointer in `AGENTS.md` (Code Style
   section) linking to the canonical doc.
4. Audit and update every public item across all 6 workspace crates to conform.
5. Update review skill + agents (`code-review` skill, `review-findings`,
   `self-review`) to enforce the convention.
6. Enable all relevant compiler/clippy lints that mechanically enforce the convention.
7. Update `quartzite-macros` so emitted user-facing code carries conforming docs.

## Out of scope

- Private items (only `pub` items must conform).
- Doc rendering / docs.rs styling.
- Doctest coverage targets beyond the existing `# Examples` rule.

## Deferred

(none — single PR, full audit per user request.)

## Key decisions

| Question | Decision |
|---|---|
| Convention basis | Full RFC 1574 + deterministic.space machine-readable inline-markdown convention. |
| Where convention lives | `ai-docs/doc-convention.md` (canonical) + short pointer in `AGENTS.md`. |
| Audit scope | All 6 workspace crates, single PR. |
| Proc-macro emitted code | Convention applies to both macro's own public API and emitted user-facing items. |
| `# Parameters` trigger | Required on every public fn with ≥1 arg. |
| Trait-impl methods | Skipped — methods inherit docs from the trait definition. |
| Section order | Summary → Parameters → Returns → Type parameters → Lifetimes → Errors → Panics → Safety → Examples → See also. |
| Always-present | `# Examples` (existing rule), `# Parameters` (when ≥1 arg). |
| Conditional | `# Returns` (non-trivial), `# Errors` (returns `Result`), `# Panics`, `# Safety` (`unsafe`), `# Type parameters` (non-obvious bounds), `# Lifetimes` (non-obvious). |
| Optional | `# See also`. |
| Summary line | Single sentence, third person singular present indicative ("Returns the…"). |
| Linking | Prefer intra-doc links (`` [`Type`] ``); use full generic names (`Option<T>`). |
| Language | American English. |
| Enforcement (review) | `code-review` skill + `review-findings` + `self-review` updated together (Propagation Rule). |
| Enforcement (lints) | Enable all relevant compiler/clippy lints. |

## Technical constraints

- All crates already declare `#![deny(missing_docs)]`. New lints must coexist.
- CI runs `cargo clippy -- -D warnings`, so `warn`-level lints become hard errors.
- `clippy::doc_markdown` is noisy — needs a `clippy.toml` `doc-valid-idents`
  allowlist for project-specific names (e.g. `OpenGL`, `WebGL`, `JSON`).
  Final list emerges during the audit.
- Proc-macro `quartzite-macros` emits code in user crates under
  `#![deny(missing_docs)]` — emitted docs must satisfy that gate.
- Audit is large (6 crates). May require `/context-reset` handoff during impl.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/doc-convention.md` documents: summary-line rule (3rd person present), full section list and required ordering, `# Parameters` requirement for ≥1 arg, conditional-section rules, intra-doc link preference, American English rule, trait-impl-skip exemption, plus a conforming and a non-conforming example; cites the four reference URLs from issue #80. |
| AC2 | `AGENTS.md` Code Style section contains a "Documentation Conventions" paragraph linking to `ai-docs/doc-convention.md` with a one-line summary of mandatory rules. |
| AC3 | Every public fn / method / struct / enum / trait / macro in all 6 crates conforms to the convention (summary tense + `# Parameters` when ≥1 arg + conditional sections as applicable + correct ordering). |
| AC4 | Trait-impl methods are exempt from `# Parameters` and the structural rules; they keep their existing one-line doc (or none if the trait already documents them and clippy permits). |
| AC5 | Each crate's `lib.rs` enables: `#![deny(rustdoc::broken_intra_doc_links)]`, `#![warn(clippy::missing_errors_doc)]`, `#![warn(clippy::missing_panics_doc)]`, `#![warn(clippy::missing_safety_doc)]`, `#![warn(clippy::doc_markdown)]`. |
| AC6 | `clippy.toml` at workspace root contains a `doc-valid-idents` allowlist for project-specific identifiers (final list determined during audit). |
| AC7 | `cargo clippy --workspace --all-targets -- -D warnings` is clean. |
| AC8 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` is clean. |
| AC9 | `cargo test` is green (no doctest regressions). |
| AC10 | `cargo build -p quartzite --no-default-features` succeeds (no_std / derive-free path). |
| AC11 | `quartzite-macros` codegen emits doc comments on generated inherent items that conform to the convention; emitted trait-impl methods exempted per AC4. |
| AC12 | `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md` are updated together (Propagation Rule) to require verifying conformance to `ai-docs/doc-convention.md`. |
| AC13 | `MouseEvent::new` in `quartzite-events` carries `# Parameters` for `event_button` and `buttons_state` (resolves the issue's motivating example). |

## Open questions

(none — proceed to design.)
