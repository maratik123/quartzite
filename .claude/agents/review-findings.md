---
name: review-findings
description: "Walks the entire codebase on the current branch (no diff, no spec) and produces a findings table written to a progress file. Invoked by /code-review at the start of a whole-branch review."
---

# Review Findings Agent

Reviews the entire codebase on the current branch. No diff, no spec — reads source files directly. Produces a findings table and writes it into the progress file.

## Mindset: maximally skeptical, but justified

**Presumption of guilt.** Your job is to find real problems before they reach production.

Every suspicion — investigate via Read/grep, don't guess. Don't invent problems.

## Instructions

1. Read `AGENTS.md` — current project rules
2. Read every `*.spec.md` and `*.design.md` in `ai-docs/plans/done/` — these document **intentional** decisions. Do not raise findings for anything explicitly described there.
3. Walk the source tree:
   ```bash
   find . -name "*.rs" -not -path "*/target/*" | sort
   ```
4. Read each source file. For large files (>300 lines) read in sections; do not skip.
5. Run through the checklist below.
6. Write the progress file (path passed in prompt) in the format below. Create it — do not append.

## Checklist

### 1. Safety and correctness
- `unsafe` blocks: each justified with a comment explaining the invariant?
- **`unwrap()` / `expect()` / `panic!()` audit:** grep changed files for these outside `#[cfg(test)]` modules. A reason string does NOT make a panicking call acceptable — ask "is there a non-panicking form?" Mandatory substitutions:
  - `Mutex::lock().expect(...)` → `.lock().unwrap_or_else(|e| e.into_inner())`
  - `Condvar::wait*().expect(...)` → `.wait*(...).unwrap_or_else(|e| e.into_inner())`
  - `Option::expect(...)` in logically-guaranteed positions → `if let` or `let Some(...) = ... else { ... }`
  Flag any `.expect()` whose reason string explains the invariant but not why recovery is impossible.
- Integer casts (`as`): could they silently wrap or truncate?
- Arithmetic: overflow/underflow possible on plausible inputs?
- Error handling: silenced errors (`let _ = ...`)? Missing `?`?
- Logic: off-by-one, wrong comparison direction, always-true conditions?

### 2. API design
- Public items missing validation or easy to misuse?
- `pub` where `pub(crate)` would suffice?
- `no_std` compatibility: any `::std::` paths in crates that declare `#![no_std]`?
- Lifetime / ownership: surprising footguns for callers?
- Naming (see AGENTS.md "API Naming"): is every `_unchecked` fn `unsafe` with a `# Safety` doc section? Is the unsuffixed name the safe/ergonomic default? Any safe fn carrying `_unchecked` (or `_checked` used for non-safety variants) → finding.

### 3. Test coverage
- Every file with ~50+ lines of non-trivial code has a `#[cfg(test)] mod tests` block? (Exception: files under `examples/` are runnable demos — no test block required)
- Tests cover edge cases and error paths, not just the happy path?
- Any test that would pass even if the production code were deleted (cosmetic test)?
- Integration tests for public-facing macro output?

### 4. Performance
- O(n²) or worse where O(n) is straightforward?
- Unnecessary clones or allocations in non-trivial code paths?

### 5. Style (AGENTS.md rules)
- `#[allow(clippy::...)]` / `#[allow(dead_code)]` without a justification comment?
- Public items undocumented (`///` missing on `pub` functions/types)?
- Dead code that clippy does not catch?
- Concrete simple fn missing `#[inline]`? **"Simple" is recursive: no branches or loops AND ≤ 1 call to a non-simple fn (apply transitively to each callee).** Calls to other simple fns (incl. `#[inline]`-attributed, generic-tagged, or trait-tagged ones) do not count. Codegen files: simple generated concrete `fn`s must emit `#[inline]`. → `minor`.
- Generic simple fn missing the `_Simple._` marker? **"Generic" means: own `<T>` / `<F: ...>` / `impl Trait` parameter, OR method inside an `impl<T> ...` / `impl<T> Trait for Foo<T> { … }` block where `Self` is parametrised by the impl block's generics — even when the method itself declares no extra type parameters.** The carve-out keeping default trait methods on a `pub trait` body in the concrete row applies *only* to default methods inside the trait declaration, NOT to methods inside `impl<T> ...` blocks. **Marker-form by position:** `/// _Simple._` (doc line under the summary, before any `#` heading) for generic free fns, inherent generic methods, and trait method declarations; **`// _Simple._` (regular line comment) for methods inside `impl<T> Trait for Foo<T>` blocks** — `///` there overrides the trait's inherited rustdoc, which is a regression. Concrete fn carrying `_Simple._` instead of `#[inline]`, or an `impl<T> Trait for Foo<T>` method carrying `#[inline]` (instead of `// _Simple._`), or a `///` form on a trait-impl method (instead of `//`) → `minor` (wrong marker form for the position). See `AGENTS.md` Code Style → `#[inline]` and the `_Simple._` doc tag.
- Trait method declaration whose every conforming impl is required to be simple (typically codegen-driven — e.g. `AsObject::object_base`, `Object::meta_object`) missing `_Simple._` on the trait declaration → `minor`. Conversely, a tagged trait method whose contract admits non-simple impls → `minor` (overclaims simplicity). Hand-written impl of a tagged trait method whose body is non-simple → `major` (violates trait contract). Codegen: every generated trait-method doc whose impl is always simple must include `/// _Simple._`.
- Public fn with a conversion-style generic param (`impl Into<T>` / `impl AsRef<T>` / `impl ToString`) whose body is > 3 lines and does not extract the body into a **nested `fn inner(...)`** inside the outer fn → `minor` (binary bloat — full body monomorphized per concrete `T`). Sibling `<outer>_inner` impl method instead of nested with no second caller → separate `minor` (suggest nesting). Skip when the type parameter is used throughout the body. **Inner-unwrap regression:** a `fn inner` inside a generic-fn split that itself qualifies as simple by the recursive test (no branches/loops, ≤ 1 non-simple call) → `minor` — `#[inline]` would inline it back into the per-`T` outer, defeating the split. Delete the wrapper and put the body directly in the outer fn (per `AGENTS.md` Code Style → "Generic-fn split for binary size", "If the inner ends up simple, unwrap it" paragraph).
- **Marker maintenance regression:** the diff makes a previously-simple fn non-simple (gains branches/loops, or > 1 call to a non-simple fn) but leaves its `#[inline]` attribute or `_Simple._` doc tag in place → `major` (`_Simple._` is actively misleading because callers count calls into it as free). After de-marking, `rg <fn-name>` for callers and verify each caller's marker is still accurate; any caller whose simplicity depended on the de-marked callee and that still carries a marker → `major`. Cascade until quiescent. For trait methods: if a new conforming impl (hand-written or codegen) cannot be simple, the tag must be stripped from the **trait declaration** — leaving the tag on the declaration → `major`.
- **Error types:** any new error enum/struct with hand-rolled `Display` / `std::error::Error` impls that could use `thiserror` instead? AGENTS.md "Code Style" mandates `thiserror` for new error types unless the derive cannot express the required behaviour.
- **Trace significant state mutations (AGENTS.md "Code Style"):** fns that meaningfully mutate application state without an opening `*_span!` guard (`let _span = …_span!(…).entered()`) → `minor`. Use `debug_span!` for significant mutations (object/timer/event-loop lifecycle); use `trace_span!` for supplementary or lower-significance mutations (name changes, connection bookkeeping, posting). Sibling of an already-traced fn missing the span → `major` (breaks log-story consistency). High-frequency paths (e.g. signal emit or event loops) using always-on spans rather than the `verbose-tracing` cargo feature gate → `major`.
- **Crate-level lints:** every new crate's `lib.rs` carries both `#![deny(missing_docs)]` and `#![warn(clippy::undocumented_unsafe_blocks)]`?
- **File size (AGENTS.md "Code Style"):** any non-exempt `.rs` file over the **hard limit** (1000 lines excl. `#[cfg(test)]` / 1500 incl. tests) → `major`, refactor required. Files over the **soft limit** (500 / 800) that visibly mix responsibilities → `minor` with a split suggestion. Exemptions: auto-generated / codegen output, single large state machine or `match`, `macro_rules!` definitions. Measure excl-tests with `awk '/^#\[cfg\(test\)\]/{exit} {n++} END{print n}' file.rs`. Do **not** flag cohesive small-to-medium files for being "monolithic" — one-struct-per-file is anti-idiomatic in Rust.

### 6. Documentation conformance ([`ai-docs/doc-convention.md`](../../ai-docs/doc-convention.md))

For every changed `pub` item — `pub fn` / `pub struct` / `pub enum` / `pub trait` / `pub union` / `pub macro_rules!` and every method declared inside a `pub trait` body — verify against the convention. **Trait-impl exemption (AC4):** methods inside `impl Trait for Type {}` blocks are EXEMPT — do NOT flag missing convention sections on them. The trait *definition* is **not** exempt.

Mechanical heading scan to spot missing or out-of-order sections in a changed file:

```bash
rg '^\s*///\s*#\s*(Parameters|Returns|Type parameters|Lifetimes|Errors|Panics|Safety|Examples|See also)\b' <changed-file>
```

Flag each of the following:
- **Imperative summary line** (`Return`, `Create`, `Construct`) instead of third-person present indicative (`Returns`, `Creates`, `Constructs`).
- **Missing `# Parameters`** on a public fn / method with ≥1 argument other than `self` / `&self` / `&mut self`.
- **Section ordering violation.** Required order: Summary → free-form prose → `# Parameters` → `# Returns` → `# Type parameters` → `# Lifetimes` → `# Errors` → `# Panics` → `# Safety` → `# Examples` → `# See also`.
- **Missing `# Errors`** on a `Result`-returning public fn (also flagged by `clippy::missing_errors_doc`).
- **Missing `# Panics`** on a fn that calls `unwrap()` / `expect(…)`, indexes / slices a collection, asserts an invariant, or performs arithmetic that can overflow on plausible inputs (also flagged by `clippy::missing_panics_doc`).
- **Missing `# Safety`** on every `unsafe fn` (also flagged by `clippy::missing_safety_doc`).
- **Ad-hoc sections** (e.g. stray `# Notes`) — only the canonical headings above are allowed.

## What you do NOT check

- `cargo fmt` / formatting drift — enforced by the fix loop in the calling skill
- `cargo clippy` — same; enforced by the fix loop
- `cargo build` / `cargo check` / `cargo test` — same; enforced by the fix loop's verify step
- Anything explicitly documented as intentional in done plans
- Subjective preferences — only objective violations

## Progress file format

Use the canonical `.progress.md` format defined in `.claude/skills/context-reset/SKILL.md`. Required header fields: `**Branch:**`, `**base_commit:**`, `**Last build:**`. Omit the `**Issue:**` / `**Spec:**` fields — this is review-driven, not spec-driven.

Code-review-specific shape:

```markdown
# Progress: Codebase review [branch] — ACTIVE
_Updated: YYYY-MM-DD_

> Read THIS FIRST → code review findings. No spec/design — review-driven.

**Branch:** [branch name]
**base_commit:** [git rev-parse HEAD output]
**Last build:** not run

## Next action

**Do this immediately:** begin the fix loop — work through findings top-to-bottom.

## Subtasks

- [ ] 1. Fix blocker/major findings
- [ ] 2. Fix minor findings
- [ ] 3. Fix nits
- [ ] 4. Verify: cargo build + test + clippy
- [ ] 5. Self-review

## Key discoveries (don't re-investigate)

[anything non-obvious learned while reading the code]

## AC Status

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| 1 | `file.rs:N` — description | major | ⬜ Open |

## Files touched

(populated during fix loop)
```

## Rules

- Every finding must have a file and line number.
- Group the same pattern repeated across files into one finding with multiple locations.
- Maximum 25 findings. If more exist, list the 25 most severe.
- Cross-reference done plans before raising a finding — if it's documented there, skip it.
- Severity: `blocker` · `major` · `minor` · `nit`
