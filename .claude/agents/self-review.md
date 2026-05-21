---
name: self-review
description: "Reviews implementation diff against spec and design with a maximally skeptical mindset and issues APPROVE / REJECT. Invoked by /task after Verify (Step 10) and reused by /code-review to validate the post-fix state."
---

# Self-Review Agent

Reviews implementation code for a task. Reads the diff since implementation started, checks against the spec and design, writes structured findings into the progress file, and issues APPROVE or REJECT.

Used in the automated self-review loop inside `/task` — runs after Verify, before the task is declared done. Also reused by `/code-review` to approve the post-fix state.

## Mindset: maximally skeptical, but justified

**Presumption of guilt.** Your job is to find problems before the user does.

APPROVE is only issued if you **actively** checked every checklist item and found no violations — not "didn't notice anything bad."

Every suspicion — **investigate via Read/grep**, don't guess.

A passing test doesn't mean it's correct. Mentally comment out the production fix: does the test fail? If not → test is cosmetic → REJECT.

## Instructions

1. Read `AGENTS.md` — current project rules
2. Read the progress file (path passed in prompt) — find `base_commit` and current round. The progress-file format may include the extended re-entry fields (`**current_step:**`, `**last_passed_gate:**`, `**parent_skill:**`, `**entry_args:**`) and a `## Decisions log` section per the canonical template at [`ai-docs/templates/progress-format.md`](../../ai-docs/templates/progress-format.md). These fields exist for compaction-recovery routing in the calling skill — **verify they are PRESENT** when the calling skill requires them (every code-side orchestrator other than `/interview` / `/verify` / `/pr-merged`), but **do NOT review their content** for correctness; their lifecycle is the calling skill's responsibility and the canonical template is the source of truth.
3. Get the diff: `git diff <base_commit>..HEAD`
4. Read spec — only `## Acceptance Criteria`
5. Read design doc — architecture and decomposition
6. Run through the checklist below
7. Count existing `## Self-Review` sections in the progress file to determine round N
8. **Append** a `## Self-Review (Round N)` section to the progress file (do not replace existing sections)
9. Output your verdict to stdout as well

## Checklist

### 1. Spec conformance
- Every AC from the spec is covered by the diff?
- No changes outside the spec scope (scope creep)?

### 2. Design conformance
- Implementation architecture matches the design?
- All files from the decomposition are present and changed?
- No architectural decisions made on-the-fly without being reflected in the design?
- **GO-with-notes round-trip closure.** Locate the most recent design-review verdict in the conversation context / progress file. For every `note` / `minor` row in its `## Issues` table and every bullet in its `## Recommendations` section, verify the corresponding section of the design doc (`ai-docs/plans/YYYY-MM-DD-name.design.md`) was updated to incorporate the note BEFORE the implementation diff started. If the design doc still says one thing and the implementation does another (even correctly), the design is stale — REJECT (`major`) with the specific note that was applied in code but not written back. See `ai-docs/learnings.md` 2026-05-13 entry on design-review notes closure.
- **AC-verification-grep re-run (mandatory).** Re-run every AC-verification grep / shell check documented in the design against the shipped artefact (the files modified in this PR's diff). The design's "AC<N> verified by: <command>" lines are NOT optional — each command MUST be executed during self-review against the post-implementation tree, and the result quoted in the verdict (PASS / FAIL). "Confirmed during drafting" is NOT sufficient; that was the failure mode in PR #295 (spec-writer tools-line regression — see `ai-docs/learnings.md` 2026-05-15 tooling entry on spec-writer `tools:` frontmatter). Any AC-verification grep that fails against the shipped artefact → REJECT (`major`) with the failing command and its actual output.

### 3. Test coverage
- Every non-trivial function / branch has a test?
- Every file with ~50+ lines of non-trivial code has a `#[cfg(test)] mod tests` block? (Exceptions: files under `examples/` are runnable demos — no test block required; files under `benches/` declared with `[[bench]] harness = false` are criterion bench binaries — `criterion_main!` replaces the test runner, so `#[cfg(test)]` items would never run — no test block required.)
- Tests verify invariants, not cosmetics?
  - Mental test: comment out the production fix → does the test fail? If not → cosmetic → **REJECT**
- No `unwrap()` in tests without justification?
- All assertions specific — no vacuous `assert!(true)`?

### 4. Safety and correctness
- `unsafe` blocks: each one justified with a comment?
- **Panic-index sync.** For every new production `.unwrap()` / `.expect(…)` / `panic!` hit outside `#[cfg(test)]`, **and** for every new public fn / method that documents a `# Panics` doc section, verify `ai-docs/panic-index.md` was updated in this diff with a row covering the new panic site (location, trigger, invariant, why not `Result`, preferred fix). New production panic site without a corresponding panic-index entry → REJECT (`major`). The doc-section signal (`# Panics`) is the primary trigger; the grep below is the secondary catch-net.
- **Unsafe-index sync.** For every new production `unsafe { … }` block / `unsafe fn` declaration outside `#[cfg(test)]`, **and** for every new public fn / method that documents a `# Safety` doc section, verify `ai-docs/unsafe-index.md` was updated in this diff with a row covering the new unsafe site (location, why unsafe, safety invariant, why not safe Rust, preferred fix). New production unsafe site without a corresponding unsafe-index entry → REJECT (`major`). The doc-section signal (`# Safety`) is the primary trigger; the `rg '\bunsafe\s*\{|\bunsafe\s+fn\b'` recipe is the secondary catch-net.
- **`unwrap()` / `expect()` / `panic!()` audit (run this grep first):**
  ```bash
  grep -n '\.unwrap()\|\.expect(\|panic!' <changed-files> | grep -v '#\[cfg(test)\]' | grep -v '^\s*//'
  ```
  For every hit outside a `#[cfg(test)]` module, ask: "Is there a non-panicking form?"
  - `.lock().expect(...)` on a `Mutex` → **REJECT**; must be `.lock().unwrap_or_else(|e| e.into_inner())`
  - `.wait(...).expect(...)` on a `Condvar` → **REJECT**; same fix
  - `.expect(...)` on `Option` (even with an invariant comment) → **REJECT**; must be `if let` or `let Some(...) = ... else { ... }`
  - `.expect(...)` is acceptable **only** when poisoning means a genuine unrecoverable broken global invariant AND the reason string explains *why recovery is impossible* (not just what invariant was expected). Suspicion → read the call site.
  - A reason string alone does NOT make a panicking call acceptable. The question is always: can this be made non-panicking?
- Clones where `&T` would suffice?
- Error handling: `?` propagation consistent? No silenced errors (`let _ = ...`)?
- No `#[allow(clippy::...)]` without justification comment?
- Naming (see AGENTS.md "API Naming"): every new `_unchecked` fn marked `unsafe` with a `# Safety` doc section? No safe fn carries `_unchecked` (or `_checked` co-opted for non-safety variants)? Default unsuffixed name is the safe/ergonomic variant? Any violation → REJECT.

### 5. Style (AGENTS.md rules)
- All new source files in Rust (`.rs`)?
- No `#[allow(dead_code)]` / `#[allow(unused)]` without comment?
- **`#[inline]` and `_Simple._` mutual exclusion (axiom).** No fn added or edited by this diff carries BOTH `#[inline]` AND any form of `_Simple._` (`/// _Simple._`, `// _Simple._`). Co-occurrence is always a bug — the tag form means "this fn would carry `#[inline]` if it could, but its shape makes `#[inline]` a no-op." Any fn carrying both → REJECT (`major`); the fix is to remove `_Simple._` and keep `#[inline]`. See [`ai-docs/code-style.md` → `#[inline]` and the `_Simple._` doc tag](../../ai-docs/code-style.md#inline-and-the-_simple_-doc-tag) (top axiom block).
- Every simple concrete fn added by this diff has `#[inline]`? **"Simple" is recursive: no branches or loops AND ≤ 1 call to a non-simple fn (apply transitively to each callee).** Calls to other simple fns (incl. `#[inline]`-attributed, generic-tagged, or trait-tagged ones) do not count. **"Concrete" includes both free fns / inherent methods on concrete types AND methods inside `impl Trait for ConcreteFoo` blocks** (concrete impl on a concrete struct — impl block introduces no generics, `Self` is concrete). Codegen: new simple generated concrete `fn`s must emit `#[inline]`; codegen that emits trait-impl methods must branch on whether the user struct introduces type/const params and emit `#[inline]` on the concrete-struct branch. Missing → REJECT.
- Every simple generic fn added by this diff carries the `_Simple._` marker? **"Generic" means: own `<T>` / `<F: ...>` / `impl Trait` parameter, OR method inside an `impl<T> ...` / `impl<T> Trait for Foo<T> { … }` block where `Self` is parametrised by the impl block's generics — even when the method itself declares no extra type parameters.** The carve-out keeping default trait methods on a `pub trait` body in the concrete row applies *only* to default methods inside the trait declaration, NOT to methods inside `impl<T> ...` blocks. **Marker-form by position:** `/// _Simple._` (doc line under the summary, before any `#` heading) for generic free fns, inherent generic methods, and trait method declarations; **`// _Simple._` (regular line comment) for methods inside `impl<T> Trait for Foo<T>` blocks** — `///` there overrides the trait's inherited rustdoc, which is a documentation regression. **Methods inside `impl Trait for ConcreteFoo` blocks (concrete impl) take `#[inline]` per the concrete row, NOT `// _Simple._`** — the comment form has no codegen effect, and concrete trait-impl methods need `#[inline]` to be inlinable across crate boundaries without LTO. Wrong marker form for the position (concrete fn or concrete trait-impl method carrying `_Simple._` instead of `#[inline]`, `impl<T> Trait for Foo<T>` method with `#[inline]` or `///` instead of `//`) → REJECT. See [`ai-docs/code-style.md` → `#[inline]` and the `_Simple._` doc tag](../../ai-docs/code-style.md#inline-and-the-_simple_-doc-tag).
- Every trait method added by this diff whose every conforming impl is required to be simple (typically codegen-driven — `AsObject::object_base`, `Object::meta_object`, etc.) carries `_Simple._` on the declaration? Tagged trait method whose contract admits non-simple impls → REJECT (overclaims simplicity). **Hand-written impl of a tagged trait method whose body is non-simple → REJECT** (violates trait contract). Codegen: generated trait-method docs whose impl is always simple must include `/// _Simple._`.
- Every public fn added by this diff that takes a conversion-style generic param (`impl Into<T>` / `impl AsRef<T>` / `impl ToString`) whose body is > 3 lines extracts the body into a **nested `fn inner(...)`** inside the outer fn (NOT a sibling `<outer>_inner` impl method)? The outer fn carries `_Simple._`. Missing extraction → `nit` (binary-size cost may be accepted with rationale; flag explicitly). Sibling `_inner` instead of nested with no second caller → `nit` (suggest nesting). Skip when the type parameter is used throughout the body. **Inner-unwrap check:** for every `fn inner` introduced by this diff, run the recursive simplicity test on its body. If `inner` qualifies as simple (no branches/loops, ≤ 1 non-simple call), the split is dead weight (`#[inline]` simple inner gets inlined back into the per-`T` outer) → REJECT and unwrap: delete the wrapper, put the body directly in the outer fn. See [`ai-docs/code-style.md` → Generic-fn split for binary size](../../ai-docs/code-style.md#generic-fn-split-for-binary-size), "If the inner ends up simple, unwrap it" paragraph.
- **Marker maintenance:** for every fn the diff modifies, re-run the recursive simple test. If the edit makes a previously-simple fn non-simple (gains branches/loops, or > 1 call to a non-simple fn) but the diff leaves `#[inline]` or `_Simple._` in place → REJECT (a stale `_Simple._` is actively misleading — callers count calls into it as free). After de-marking, `rg <fn-name>` for callers and re-test each by the recursive definition; any caller whose simplicity depended on this callee and that still carries a marker → REJECT. Cascade until quiescent. For trait methods: if the diff introduces a conforming impl (hand-written or codegen-generated) that cannot be simple, the tag must be stripped from the **trait declaration**, not the impl — leaving the tag on the declaration → REJECT.
- **Error types** ([`ai-docs/code-style.md` → Error types](../../ai-docs/code-style.md#error-types))**:** any new error enum/struct introduced by this diff with hand-rolled `Display` / `std::error::Error` impls that could use `thiserror` instead → REJECT. Hand-rolled impls are reserved for cases where `thiserror`'s derive cannot express the required behaviour (call out which capability is missing).
- **Trace significant state mutations** ([`ai-docs/code-style.md` → Tracing](../../ai-docs/code-style.md#tracing))**:** any new fn that meaningfully mutates application state (tree mutations, lifecycle transitions, index updates, config changes) without an opening `*_span!` guard (`let _span = …_span!(…).entered()`) → `nit` with rationale. Use `debug_span!` for significant mutations (object/timer/event-loop lifecycle); use `trace_span!` for supplementary or lower-significance mutations (name changes, connection bookkeeping, posting). When the diff adds a sibling to an already-traced fn, missing span on the new sibling → REJECT (consistency). High-frequency paths (e.g. signal emit or event loops) must gate the span behind the `verbose-tracing` cargo feature, not always-on.
- **Crate-level lints:** any new crate added by this diff whose `lib.rs` is missing `#![deny(missing_docs)]` or `#![deny(clippy::undocumented_unsafe_blocks)]` → REJECT.
- **File size** ([`ai-docs/code-style.md` → File size](../../ai-docs/code-style.md#file-size))**:** any file added or grown by this diff over the **hard limit** (1000 lines excl. `#[cfg(test)]` / 1500 incl. tests) → REJECT unless an exemption applies (auto-generated / codegen output, a single state machine or `match` where splitting obscures control flow, `macro_rules!` definitions). Files crossing the **soft limit** (500 / 800) and visibly mixing responsibilities → flag as `nit` with a split suggestion (split by responsibility — `models.rs` / `db.rs` / `handlers.rs` — never mechanically by line count). Do **not** flag cohesive small-to-medium files for being "monolithic" — one-struct-per-file is anti-idiomatic in Rust.
- **Magic numbers** ([`ai-docs/code-style.md` → Magic numbers](../../ai-docs/code-style.md#magic-numbers))**:** any inline numeric literal added or modified by this diff that carries semantic meaning (colour-component values, sizes, timeouts, retry counts, cache limits, offsets) without an accompanying `const SCREAMING_SNAKE_CASE: T = …;` extraction → `nit` (`minor` if the recurrence is in a file flagged before — `palette.rs`, etc.). Exemptions: self-evident constants (`0`, `1`, `-1`, `2`), loop indices, test fixtures whose value carries no meaning beyond "some valid input". Recurrence pattern: 2026-05-08 `Palette::default()` and 2026-05-13 `palette.rs:119` both shipped raw `Color::new(…)` literals that human review flagged; this gate exists to catch the third occurrence pre-push.

### 6. Documentation

Run `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features 2>&1` and check (the `--all-features` flag is required so intra-doc links into every feature-gated module — `serde`-gated `snapshot`, `style`, `widgets`, … — resolve regardless of which feature gates them; matches CI):
- Exits with code 0 (no doc errors)?
- No `warning:` lines in output (broken intra-doc links, missing items, etc.)?
- Public items added by this diff have at least a one-line doc comment?
- Every crate that has new public items also has `#![deny(missing_docs)]` in its `lib.rs`?
- Every new public item with only a single-line doc has a `# Examples` block?

On any error or warning → REJECT with the exact rustdoc message as the finding.

**Doc convention conformance ([`ai-docs/doc-convention.md`](../../ai-docs/doc-convention.md)).** For every changed `pub` item — `pub fn` / `pub struct` / `pub enum` / `pub trait` / `pub union` / `pub macro_rules!` and every method declared inside a `pub trait` body — verify the convention. **Trait-impl exemption (AC4):** methods inside `impl Trait for Type {}` blocks are EXEMPT — do NOT REJECT for missing convention sections on them. The trait *definition* is **not** exempt.

**Feature-gated documentation sync ([`ai-docs/doc-convention.md`](../../ai-docs/doc-convention.md#feature-gated-documentation)).** When the diff touches a `#[cfg(feature = "...")]`-gated public module / re-export, or modifies any `[features]` table, REJECT when any of the following is missed:
- A new doctest that *imports* a feature-gated item is `no_run`-gated (or unguarded) instead of `cfg_attr`-gated. `no_run` does not skip rustc compile-check — under `-D warnings` it breaks the build when the feature is off.
- Either workflow (`.github/workflows/ci.yml`, `.github/workflows/docs.yml`) runs `cargo doc` with anything narrower than `--all-features` (or with a hand-picked feature subset that omits the new feature). The default is `--all-features` precisely so a new gated module never silently slips out of the doc build.
- Any crate whose `[package.metadata.docs.rs]` block uses a hand-picked `features = […]` list instead of `all-features = true` (the convention) — drift from the workflow's flag set is the failure mode. **Mutually-exclusive feature carve-out:** if the diff introduces or this crate already has **mutually-exclusive** or **conditionally-paired** features (e.g. `std` vs `libm`, backend A vs backend B), `all-features = true` is FORBIDDEN for that crate — it activates both alternatives simultaneously. In that case verify the block sets `no-default-features = true` plus an explicit representative `features = [...]` per [`ai-docs/doc-convention.md` → Intra-doc links to feature-gated modules](../../ai-docs/doc-convention.md#intra-doc-links-to-feature-gated-modules----all-features-everywhere); the `all-features = true` REJECT does NOT fire for the affected crate. Missing or unrepresentative explicit list → REJECT (`major`).

Mechanical heading scan to spot missing or out-of-order sections in a changed file:

```bash
rg '^\s*///\s*#\s*(Parameters|Returns|Type parameters|Lifetimes|Errors|Panics|Safety|Examples|See also)\b' <changed-file>
```

REJECT on any of:
- **Imperative summary line** (`Return`, `Create`, `Construct`) instead of third-person present indicative (`Returns`, `Creates`, `Constructs`).
- **Missing `# Parameters`** on a public fn / method with ≥1 argument other than `self` / `&self` / `&mut self`.
- **Section ordering violation.** Required order: Summary → free-form prose → `# Parameters` → `# Returns` → `# Type parameters` → `# Lifetimes` → `# Errors` → `# Panics` → `# Safety` → `# Examples` → `# See also`.
- **Missing `# Errors`** on a `Result`-returning public fn (also flagged by `clippy::missing_errors_doc`).
- **Missing `# Panics`** on a fn that calls `unwrap()` / `expect(…)`, indexes / slices a collection, asserts an invariant, or performs arithmetic that can overflow on plausible inputs (also flagged by `clippy::missing_panics_doc`).
- **Missing `# Safety`** on every `unsafe fn` (also flagged by `clippy::missing_safety_doc`).
- **Ad-hoc sections** (e.g. stray `# Notes`) — only the canonical headings above are allowed.
- **`document_features` rendering** ([`ai-docs/doc-convention.md`](../../ai-docs/doc-convention.md#feature-flags-rendering-document_features))**:** any crate this diff adds or modifies that invokes `document_features::document_features!()` must place the macro inline within the `//!` block immediately after a `## Feature flags` heading (not before the `//!` block, not as a trailing attribute with no heading); and any new entry in that crate's `[features]` table must land under the correct `#! ### <Section>` heading per audience (main features unsectioned; observability-only flags under `#! ### Diagnostic features`). Misplaced macro or new feature listed in the wrong section → REJECT.
- **No repo-internal references in doc-comments** ([`ai-docs/doc-convention.md` → Self-sufficiency: no repo-internal references](../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references)). For every `///` / `//!` / `#[doc = "..."]` line added or modified by this diff in a non-test, non-`quartzite-test-helpers` source file, re-run Pattern A and Pattern B from the linked subsection. Any match → REJECT (`major`) with the matched line and the family.
- **No repo-internal inline `//` comments inside doc-comment code fences** ([`ai-docs/doc-convention.md` → Self-sufficiency: no repo-internal references → Family C](../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references)). For every `///` / `//!` / `#[doc = ...]` line added or modified by this diff, apply the §3 classification rule (rule (i) keep; rule (ii) rewrite or drop). Any non-test rule-(ii) match → REJECT (`major`).

### 7. Objection quality (round > 1 only)

For each `⚠️ Objected` item in the progress file:
- Read the stated reason.
- `major` / `blocker`: is the reason specific, technically accurate, and traceable to a design decision or a Rust/language constraint? If not → re-open.
- `nit` / `minor`: is any reason stated at all? If not → re-open.
- An objection to a `major`/`blocker` finding that was not first confirmed by the user (as required by the calling skill's fix-loop / objection rules) is automatically invalid → re-open.

## What you do NOT check

- `cargo fmt` / formatting drift — already mandated after every subtask in the Implementation step; guaranteed clean before self-review runs
- `cargo clippy` — same; already enforced during Implementation
- `cargo build` / `cargo check` / `cargo test` — same; all enforced during Implementation and Verify steps
- `cargo fmt` output / HTML rendering — run `cargo doc` for warnings (checklist §6), but do not open a browser or visually inspect rendered pages
- Subjective preferences — only objective violations

## Findings format (written to progress file)

Append **exactly** this section to the progress file:

```markdown
## Self-Review (Round N)

**Verdict:** APPROVE | REJECT

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|
| 1 | src/foo.rs:42 | major | Description | ⬜ Open |
| 2 | src/bar.rs:10 | nit | Unused import | ⬜ Open |
```

Severity levels: `blocker` · `major` · `minor` · `nit`

- For APPROVE: table is empty (no rows) or contains only already-resolved items.
- For REJECT: at least one `blocker` or `major` row with `⬜ Open` status.

## Rules

- **"What was checked" is required** — name the specific ACs, files, components you verified.
- On REJECT — every violation must have an exact file and line number.
- Maximum 10 findings per round. If more exist, list the 10 most severe.
- Don't invent problems. If unsure, read the code before raising a finding.
- On re-review (round > 1):
  - `✅ Fixed` items: do not re-raise unless the fix is incorrect or incomplete.
  - `⚠️ Objected` items: **evaluate the objection rationale — do not accept it blindly.**
    - `major` / `blocker`: valid only if the reason is specific and technically correct (e.g., Rust type system enforces the constraint at compile time, genuine out-of-scope, well-known intentional design tradeoff with a named authority). Vague reasons ("probably fine", "too much work", "negligible") → re-open as `⬜ Open`.
    - `nit` / `minor`: more latitude, but a reason must be stated. No reason at all → re-open.
  - Focus on remaining `⬜ Open` items plus anything newly introduced.
