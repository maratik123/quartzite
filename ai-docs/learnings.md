# Learnings

### 2026-05-08 — documentation — doctests that reference items behind a `#[cfg(feature = "X")]` re-export must be feature-gated via `#![cfg_attr(feature = "X", doc = r#"…"#)]` injection, not just `no_run`

**What happened:** PR #156 / `/task #60` shipped a `# Quickstart` doctest in `quartzite/src/lib.rs` modelled on `tokio` — `no_run`-annotated, using `#[derive(Extend, DeriveObject)]` + `#[object_impl]` + `#[slot]`. Local `cargo test --doc -p quartzite` passed (default features include `derive`, prelude re-exports the derive macros, doctest compile-checks fine). CI failed under the `Feature matrix (--no-default-features --features std)` job: under `--no-default-features --features std`, the `derive` feature is OFF, the prelude doesn't re-export the derive macros, and the doctest fails to resolve `Extend` / `DeriveObject` / `#[object_impl]` / `#[slot]` / `Object::write_property` / `Object::invoke_method`. **The `no_run` annotation does not skip compilation** — rustdoc still feeds the doctest source through `rustc` to check it; `no_run` only skips the run-the-resulting-binary step.

**Rule:** When a doctest references types, traits, or macros that are gated behind a cargo feature (typically via a `#[cfg(feature = "X")] pub use …` in the prelude or facade), the doctest itself must be **conditionally injected** so it only exists for builds where that feature is enabled. The clean way is to feature-gate the entire `# Heading` + code block via `cfg_attr` injection at the crate level:

```rust
#![cfg_attr(
    feature = "derive",
    doc = r#"# Quickstart

```no_run
use crate_name::prelude::*;

#[derive(SomeMacro)]
struct Foo { … }

fn main() { … }
```"#
)]
```

When the feature is enabled, the `cfg_attr` expands to `#![doc = "…"]` and the doctest is injected into the crate doc; rustdoc extracts it and compile-checks it. When the feature is disabled, the `cfg_attr` expands to nothing — no doc string, no extracted doctest, no compile failure under `--no-default-features`.

This pattern preserves the bit-rot guard under the feature-on path (compile-check still happens) AND keeps the section visible in rustdoc on `docs.rs` / GitHub Pages (both build with default features, so the `cfg_attr` injects).

**Why:** `no_run` is misleadingly named — readers (and authors) reasonably assume "no_run" means "rustdoc doesn't touch this", but rustdoc still parses, type-checks, name-resolves, and trait-resolves the doctest body. Anything reachable in the doctest's source must be reachable in the *current* feature configuration, not just one well-known configuration. CI matrices that test multiple feature combinations will surface this gap; a single-config local `cargo test --doc` will not.

**How to apply:**

- When writing a new doctest in a crate that has feature-gated re-exports (e.g., `quartzite` facade with the `derive` feature gating the macros), check whether the doctest's symbols are unconditionally available. If not, wrap the section in `#![cfg_attr(feature = "X", doc = r#"…"#)]`.
- Alternative for a fragment-only doctest where bit-rot guard is not load-bearing: use ```` ```ignore ```` instead of `no_run` — rustdoc skips it entirely under all configs. Loses compile-check; only acceptable when a runnable example elsewhere in the project (`examples/<thing>.rs` built by every CI matrix entry) covers the API surface.
- Reviewer / `self-review` agent obligation: when reviewing a PR that touches a doctest in a feature-gated crate, the doctest's feature configuration must match the symbols it uses. Spot-check by mentally running `cargo test --doc -p <crate> --no-default-features --features <minimal>` — if the doctest relies on a feature not in the minimal set, it must be `cfg_attr`-gated.

The mistake also exposes that explicit `fn main()` (used in the same doctest to defeat rustdoc's auto-`fn main` wrapping for derive-macro path resolution) does NOT itself gate the body — the macros are still referenced at parse time.

**Escalated?** no

> Candidate for escalation to `ai-docs/doc-convention.md` (a "Doctests + features" sub-section) and to `.claude/agents/self-review.md` / `.claude/agents/review-findings.md` checklist (under feature-gated re-exports, doctest must be feature-gated). `/improve` should consider on recurrence.

### 2026-05-08 — process — `/task` Step 12 finalise commit must regenerate every auto-derived file whose source is among the artefacts the step touches

**What happened:** PR #156's finalise commit (`977739d`) updated `ai-docs/plans/INDEX.md` (added the `project-docs` row to the Active table) but did NOT regenerate `ROADMAP.md` — the auto-generated file produced from `INDEX.md` by `scripts/gen-roadmap.sh`. The CI sync-gate landing in the same PR caught the drift on the very first run after push: `git diff --exit-code ROADMAP.md` failed because the committed `ROADMAP.md` (from subtask 5) was stale relative to the now-updated `INDEX.md`. The implementer ran the generator, regenerated `ROADMAP.md`, and committed in a follow-up (`887a11f`) — fix worked, but it cost an extra round-trip through CI.

**Rule:** During `/task` Step 12 (the finalise commit that moves spec/design to `done/` and updates `INDEX.md` / `context.md`), **identify every auto-generated file in the workspace whose source includes one of the files being modified**, regenerate them in the same commit. For quartzite the current set is:

- `ai-docs/plans/INDEX.md` → drives `ROADMAP.md` via `scripts/gen-roadmap.sh` → regenerate `ROADMAP.md` whenever INDEX changes.

The workspace may grow more such relationships (e.g., a future tool that generates a public-API surface listing from rustdoc JSON, a feature-matrix matrix file generated from `Cargo.toml`, etc.). Track them in `ai-docs/context.md` if it gets non-trivial.

**Why:** Each auto-generated file lands its own CI gate (the sync-gate pattern). The PR that introduces both the source change and the gate cannot merge until the artefact is in sync. Skipping the regenerate step turns a one-commit PR into a two-commit PR with a CI-failure round-trip in the middle. The sync-gate's *job* is to catch this — but the goal is for the implementer to land sync correctly the first time so the gate runs green.

**How to apply:** in the `/task` Step 12 checklist (or its skill prose), add a *Regenerate dependent artefacts* sub-step that lists the known auto-generation triggers. Today: "if `ai-docs/plans/INDEX.md` or `ai-docs/plans/done/**` changed in this commit, run `./scripts/gen-roadmap.sh` and stage `ROADMAP.md`". When new generators land, append to the list.

**Escalated?** skill:task

### 2026-05-07 — documentation — `document_features::document_features!()` invocation must sit inline within the `//!` crate doc, immediately after a `## Feature flags` heading; main vs diagnostic features must be sectioned in Cargo.toml

**What happened:** Two related bugs in our `document_features` integration, both surfaced once `cargo doc` was published live to GitHub Pages (PR #148, #137):

1. **Macro placement was wrong in two crate roots.**
   - `src/lib.rs` invoked `#![doc = document_features::document_features!()]` on line 9, **before** the `//!` crate-doc block (lines 10-24). The rendered feature list appeared **first**, ahead of the overview.
   - `quartzite-core/src/lib.rs` invoked it at the end of the attribute block (line 15) **with no preceding `## Feature flags` heading**. The features rendered as an unlabelled appendix with no TOC anchor.

2. **`verbose-tracing` was mixed with main features.** In workspace `Cargo.toml` and `quartzite-core/Cargo.toml`, `verbose-tracing` was listed as a peer of `std` and `derive` — so the rendered feature list mixed a purely-diagnostic observability flag with the main feature-flag choices users actually evaluate (build-target, derive availability).

PR #149 fixed both. Verified visually on `target/doc/*/index.html`: Examples → Feature flags (h2) → Diagnostic features (h4 subsection) — sidebar TOC correctly nests.

**Rule:**

- **Macro placement.** When a crate uses `document_features`, place the `#![doc = document_features::document_features!()]` invocation **inline within the `//!` crate doc**, immediately after a `## Feature flags` (or `# Feature flags` — whichever heading level matches sibling sections like `# Examples` already in that crate doc). The remaining inner attributes (lints, `cfg_attr`, etc.) follow afterwards. Never place the macro **before** the `//!` block (forces features-list to render first) or **after the entire attribute block with no preceding heading** (renders as an unlabelled appendix). Canonical shape:

  ```rust
  //! Crate overview…
  //!
  //! # Examples
  //! …
  //!
  //! # Feature flags
  #![doc = document_features::document_features!()]
  #![other lint attributes…]
  ```

- **Cargo.toml feature sectioning.** Group features in `[features]` by audience using `#! ### <Section>` headings:
  - **Main features** (default-on, commonly toggled, affects build target / API surface): listed first under no extra heading, just `## per-feature docstrings`.
  - **Diagnostic features** (purely additive observability — tracing spans, debug instrumentation, profiling hooks): under `#! ### Diagnostic features` with a one-paragraph `#!` intro stating "Off by default. Enabling these is purely additive and only affects observability, never correctness or behaviour."
  - Other categories (e.g. `#! ### Experimental features`, `#! ### Optional dependencies`) follow the canonical `document_features` example as needed.

  Crates that **don't** invoke `document_features!()` (e.g. `quartzite-runtime` with only `verbose-tracing`) don't need section headings — the comments would be inert decoration.

**Why:**

- Reader priority: the human-curated overview is what readers want first. An auto-generated feature list before it inverts reading priority and forces every visitor to scroll past low-value content.
- Anchorability: a section without an `## Feature flags` heading has no TOC anchor; readers can't link to it, can't search for it in the sidebar, can't deep-link from external docs.
- Decision fatigue: mixing diagnostic features with main features makes every reader evaluate "do I need this?" on every line. Section headings communicate the "main features matter; diagnostic features are additive observability" distinction at a glance.

**How to apply:**

- When adding `document_features` to a new crate, write the source file in the canonical shape above before any other lint attributes are added — easier than retrofitting later.
- When adding a feature to an existing `[features]` table:
  1. Decide its audience: build-target/API-surface (main) vs observability-only (diagnostic) vs experimental/in-development.
  2. If it's diagnostic and the table doesn't already have a `#! ### Diagnostic features` section, add one (with the standard intro paragraph) before adding the feature.
  3. Place the `## per-feature docstring` immediately above the feature line.
- When reviewing a PR that touches `document_features`-using crates, check both: (a) macro is inline within `//!` after a heading; (b) any new feature lands under the right `#! ###` section per its audience.

**Escalated?** doc-convention, agent:self-review, agent:review-findings

### 2026-05-07 — process — query the registry / release API for current versions before pinning a dependency or GitHub Action in any spec, issue body, design doc, or instruction file

**What happened:** Twice in one session, issue bodies I authored cited stale dependency / action versions that the user had to correct:

1. **#135** (Bencher CI) initially said `criterion = "0.5"`. The actual current is `0.8.2`. The user paste-corrected it after observing crates.io.
2. **#137** (cargo doc publishing) said `actions/deploy-pages@v4` (and offered `peaceiris/actions-gh-pages@v3` as a fallback). The actual current is `@v5.0.0`; `@v4` is **node20** and would have re-introduced the exact Node-20 deprecation we'd just spent PR #146 fixing across the bench workflows. The user caught it before any `/task` session picked the issue up.

The pattern is the same in both cases: I named a specific version from training-data knowledge, and the named version was several majors behind real-world current.

**Rule:** When authoring an issue body, design doc, spec, or AGENTS.md / `ai-docs/**` addition that names a **specific** dependency or GitHub Action version, query the live source first:

- **Cargo crates:** `curl -sS "https://crates.io/api/v1/crates/<name>" | jq -r '.crate.max_stable_version'`
- **GitHub Actions:** `gh api /repos/<owner>/<repo>/releases --jq '.[0].tag_name'` — and also fetch `action.yml` to confirm the Node runtime is current (`gh api /repos/<owner>/<repo>/contents/action.yml --jq '.content' | base64 -d | grep -E 'using:|node'`)
- **Project crate convention** (AGENTS.md `## Dependency Versions`): use `0.x` for `0.x.y`, `x` for `x.y.z`. Apply that pinning *to the version actually observed in the registry*, not to a remembered version.

If a body needs a long-lived stable reference (e.g., a doc that won't be revisited for months), include a comment `(verified current YYYY-MM-DD)` so future readers can spot drift before a `/task` session pins the stale value into a Cargo.toml or workflow file.

**Why:** Stated versions become **load-bearing** the moment a `/task` session picks the issue up. The Sonnet 4.6 implementer follows the spec literally — `criterion = "0.5"` lands in `Cargo.toml` even though `"0.8"` is current; `actions/deploy-pages@v4` lands in a workflow file even though `@v4` is the Node-20 version we'd just deprecated workspace-wide. The cost of the mistake is asymmetric: 30 seconds of registry query at authoring time vs. a corrective PR + reviewer time vs. (worst case) a reverted regression.

**How to apply:** Before writing any version-pinning string in any document the user might act on, run the registry query. The training cutoff is months behind live releases — treat training-data version knowledge as untrustworthy by default, especially for fast-moving projects (criterion bumped 0.5 → 0.8 between training and now; `actions/deploy-pages` bumped v4 → v5; `actions/upload-artifact` bumped v4 → v7).

**Escalated?** AGENTS.md

### 2026-05-07 — process — run actionlint on every new or modified GitHub Actions workflow file

**What happened:** Three new workflow files were created for Bencher CI integration and committed without running actionlint. The user had to ask explicitly. actionlint caught `actions/github-script@v6` being too old for the current GitHub Actions runner; fixing it required an extra commit.

**Rule:** Run `actionlint <file>` on every GitHub Actions workflow file that is created or modified before committing. A clean actionlint pass is a required gate alongside `cargo build` / `cargo clippy`.

**Why:** actionlint catches runner-version mismatches, deprecated action versions, expression syntax errors, and shell quoting issues that are invisible to `cargo` checks.

**How to apply:** After writing or editing any `.github/workflows/*.yml` file, run `actionlint <file>` before staging. If multiple files changed, pass all of them in one invocation. Fix all errors before committing.

**Escalated?** AGENTS.md, skill:task

### 2026-05-07 — process — do not escalate learnings without being asked; escalation is triggered only by /improve

**What happened:** After writing a learning entry about actionlint, I immediately escalated it to AGENTS.md and task/SKILL.md without being asked. The user had to point out the mistake.

**Rule:** Writing a learning entry (`ai-docs/learnings.md`) and escalating it to project instruction files are two separate actions. The learning entry is written automatically on any non-obvious correction. Escalation (updating AGENTS.md, skills, agents, hooks, code-style.md, doc-convention.md) is done only when the user runs `/improve`. Do not escalate on your own initiative.

**How to apply:** After writing a learning entry, mark `Escalated? no` and stop. Do not touch AGENTS.md, skill files, or agent files as a follow-up to writing the entry. Wait for `/improve`.

**Escalated?** AGENTS.md

### 2026-05-07 — code-style — criterion bench files (`harness = false`) are exempt from `#[cfg(test)]` requirement

**What happened:** Self-review of the criterion benchmarks task flagged that `quartzite-core/benches/signal_property.rs` and `quartzite-runtime/benches/object_tree.rs` have no `#[cfg(test)] mod tests` block. AGENTS.md exempts only `examples/`, not `benches/`.

**Rule:** Criterion bench files with `[[bench]] harness = false` compile as standalone criterion-driver binaries where the Rust test framework is entirely replaced. `#[cfg(test)]` code is never compiled into these binaries (there is no `rustc --test` invocation). Adding a test block is dead code and not meaningful. These files belong to the same category as `examples/` for the purposes of the `#[cfg(test)]` rule.

**Why:** `harness = false` disables the default test runner; only the `criterion_main!`-generated entry point runs. There is no mechanism to invoke `#[test]` items even if they were written.

**How to apply:** When writing a criterion bench file (`harness = false`), do not add a `#[cfg(test)] mod tests` block. The AGENTS.md test-coverage rule applies to library and application source (`src/`), not to criterion bench binaries.

**Escalated?** AGENTS.md, agent:self-review, agent:review-findings

### 2026-05-07 — code-style — concrete trait-impl methods take `#[inline]`, not `// _Simple._` (PR #129 follow-up)

**What happened:** PR #129 stripped `#[inline]` uniformly from all generated trait-impl methods in `extend/codegen.rs`, `object_impl/codegen.rs`, and `meta_enum/codegen.rs` — including emissions inside `impl AsObject for ConcreteFoo` (concrete user struct). The `/interview` discussion treated all trait-impl methods as one bucket, picked option (c) "emit no marker; rely on rustdoc inheritance from the trait declaration", and concluded that the change was rule hygiene with no codegen effect. The actual rule in `ai-docs/code-style.md` L131-138 already split the trait-impl case into two rows by impl-block genericity — concrete impl falls in the **concrete** row (`#[inline]`), only generic impl falls in the **generic** row (`_Simple._`). The marker-form decision tree at L165-170 omitted the concrete-impl row entirely, which is what primed the misreading.

**Rule:** A method inside `impl Trait for ConcreteFoo` (concrete impl on a concrete struct — impl block introduces **no** generics, `Self` is concrete) takes `#[inline]`, not `// _Simple._`. The comment form has no codegen effect, and concrete-impl trait methods need `#[inline]` to be inlinable across crate boundaries without LTO — without it, only the symbol's signature crosses the rmeta boundary, so downstream callers get a real `call` instruction even for `fn object_base(&self) -> &ObjectBase { &self.base }`. Codegen that emits trait-impl methods must branch on whether the user struct introduces type/const params: emit `#[inline]` for concrete-struct emission, emit nothing for generic-struct emission. `// _Simple._` cannot be emitted via `quote!` (Rust strips comments before token-stream parsing) and is unnecessary on either branch.

**Why:** Three independent reasons reinforce the rule. (1) The canonical position-to-marker rule treats "no own type params AND `Self` concrete AND impl block introduces no generics" as the concrete row regardless of whether the method sits in an inherent impl or a trait impl. (2) `#[inline]` is the only attribute that exports MIR into the consuming crate's rmeta when LTO is off; comments don't. (3) Conflating concrete-impl and generic-impl trait methods loses real codegen output — release builds without LTO regress on accessor inlining.

**How to apply:** When writing or reviewing codegen that emits a trait-impl method whose body is simple, inspect whether the user struct introduces type/const params (`generics.params.is_empty()` predicate on the relevant `syn::Generics`). Emit `#[inline]` on the concrete-struct branch; emit nothing on the generic-struct branch. When writing or reviewing hand-written code, apply the same split: `impl Trait for ConcreteFoo` → `#[inline]`; `impl<T> Trait for Foo<T>` → `// _Simple._`. The `self-review` and `review-findings` agent checklists now spell out the concrete trait-impl case explicitly, so a future PR repeating PR #129's pattern should be caught at review.

**Escalated?** code-style, doc-convention, AGENTS.md, agent:self-review, agent:review-findings

### 2026-05-07 — process — `memory` is user-local, not a project-level escalation target

**What happened:** The `Escalated?` enum in AGENTS.md `## Corrections Log` listed `memory` alongside `AGENTS.md`, `skill:[name]`, `hook`, etc. as a valid value. `/improve` and the escalation-audit agent both said "memory counts as unescalated for project-level audit purposes" — i.e., the system was treating it as `no` while still presenting it as a legitimate-looking enum option. Five existing learnings entries said `Escalated? AGENTS.md, memory` where the `, memory` was load-bearing-looking but actually carried no project-level signal.

**Rule:** `Escalated?` records **project-level** persistence only — instruction files visible to every contributor (AGENTS.md, skills, agents, hooks, project `settings.json`, `ai-docs/doc-convention.md`, `ai-docs/code-style.md`). User-local persistence (`~/.claude/.../MEMORY.md`, `settings.local.json`) is structurally identical: private to one developer, not visible to future readers. **Neither is a valid value of `Escalated?`.** If a correction was saved only to user-local persistence, mark `Escalated? no`. Saving to memory is independent of escalation; it's worth doing for personal context but doesn't substitute for landing the rule in a project file.

**Why:** Listing `memory` as an enum option created the false impression it counted as escalation. Audit agents had to special-case it (`treat as no`), and reviewers of correction-log diffs couldn't tell at a glance whether `, memory` was load-bearing. Dropping the value and unifying the explanatory note around "user-local persistence does NOT count" eliminates the mental footnote and aligns the enum with what `/improve` already does.

**How to apply:** When recording a correction, ignore whatever was saved to global memory or `settings.local.json` for the purposes of `Escalated?`. Mark `no` if no project-level rule was added; mark with the appropriate target (or comma-separated list) if it was. The `learnings-escalation-audit` agent now skips entries with `Escalated? no` (was: `no` or `memory`); the `self-improve` agent's classification table no longer mentions `memory`.

**Escalated?** AGENTS.md, agent:self-improve, agent:learnings-escalation-audit

### 2026-05-07 — code-style — if the `fn inner` of a generic-fn split is simple, unwrap it (don't `#[inline]` it)

**What happened:** Applied the "Generic-fn split for binary size" pattern to four fns. Two of the four inner fns (`ObjectFactory::register`'s inner — single `HashMap::insert`; `ObjectBase::named`'s inner — single `ObjectBase::new()` call + struct literal) are concrete and simple by the recursive definition. The first reaction was to add `#[inline]` to those two simple inners, matching the concrete-row marker rule. User then pointed out that `#[inline]` on a simple inner causes the compiler to inline it back into the (per-`T`-monomorphized) outer fn, which **defeats the split entirely** — the body ends up duplicated per concrete `T` anyway, just with extra source indirection. The right answer is to unwrap a simple inner, not to mark it.

**Rule:** Run the recursive simplicity test on every `fn inner` body of a generic-fn split. If the inner qualifies as simple (no branches/loops, ≤ 1 non-simple call), **delete the wrapper and put the body directly in the outer fn**. Then re-evaluate the outer's `_Simple._` marker per the marker-maintenance rule — unwrapping moves the inner's non-simple calls (≤ 1) into the outer, which can flip the outer from simple to non-simple, requiring the tag to be stripped. Keep the split only when `inner` is non-simple, which is the case the rule was designed for. An `#[inline]` simple `fn inner` inside a generic-fn split is a code smell: either the split was unnecessary (unwrap), or the inner is mis-marked.

**Why:** The split's value is exactly to keep the body out of the per-`T` monomorphization. `#[inline]` on a simple inner re-inlines it back into the outer, so the body lands in every `T` instantiation — same binary cost as never-having-split, but with extra source noise. A simple inner is by definition ≤ ~2 lines, so unwrapping never re-triggers the "> 3 lines → split" threshold.

**How to apply:** After writing a `fn inner(...)`, run the recursive simplicity check on its body. If simple → unwrap (delete the inner, hoist the body into the outer). If non-simple → keep the split, no marker on the inner. Do NOT add `#[inline]` to a simple inner inside a split — the split is the wrong tool for that body.

**Escalated?** AGENTS.md, agent:review-findings, agent:self-review

### 2026-05-07 — code-style — methods inside `impl<T> Trait for Foo<T>` are generic-shaped and need `_Simple._`, not `#[inline]`; and use `// _Simple._` (not `///`) on trait-impl methods

**What happened:** PR #120 (issue #115 implementation) added `#[inline]` to `Signal<Args>::default`, `ObjectRef<T>::clone`/`eq`, `WeakRef<T>::clone`/`eq`. All five methods sit inside `impl<T> Trait for Foo<T>` blocks and have no own type parameters, but `Self` is parametrised by the impl block's generics. The original carve-out wording was ambiguous enough to be read as covering this case. The first fix replaced `#[inline]` with `/// _Simple._`, but rendered rustdoc revealed `///` *overrides* the trait-inherited docstring — `Clone::clone`'s actual doc ("Returns a copy of the value...") was wiped out and replaced by just `_Simple._` on the rustdoc page.

**Rule (combined):**
1. From the Rust compiler's point of view, methods inside `impl<T> ...` / `impl<T> Trait for Foo<T>` blocks are generic-shaped — the body is monomorphized per concrete `T` even when the method declares no extra type parameters. They go to the **generic** row → use `_Simple._`. The carve-out for default trait methods applies *only* to default methods inside a `pub trait` body (where the body lives in the trait declaration, a single source location).
2. **Marker form by position:** `/// _Simple._` (doc line) for generic free fns, inherent generic methods, and trait method declarations — these need their own docs. `// _Simple._` (regular line comment) for methods inside `impl<T> Trait for Foo<T>` blocks — these inherit the trait's docs and a `///` line would silently replace them on the rustdoc page. Audit grep `rg '_Simple\._'` matches both forms.

**Why:** `#[inline]` on a generic `impl<T>` method is redundant (monomorphization already makes the body cross-crate-available); the marker should be `_Simple._`. But `///` on a trait-impl method is a documentation regression (overrides inherited rustdoc). `//` keeps the marker as a human-only signal in source while leaving the trait-inherited rustdoc intact. Verified against rendered HTML in `target/doc/`.

**How to apply:** When deciding the marker for a method, look at the surrounding `impl` block's generic parameter list, not just the method's own. If the impl block introduces generics that parametrise `Self`, use `_Simple._`. If you're inside a trait-impl block (`impl<T> Trait for Foo<T> { fn ... }`), use `//` not `///`. Reviewer/self-reviewer rejects `#[inline]` on generic-shape methods AND rejects `///` on trait-impl methods.

**Escalated?** AGENTS.md, doc-convention, agent:review-findings, agent:self-review

### 2026-05-07 — code-style — strip `#[inline]` / `_Simple._` when a fn stops being simple, and cascade to callers

**What happened:** During design discussion of the recursive `#[inline]` rule, the user surfaced a maintenance gap: when a previously-simple fn grows branches/loops or a second non-simple call, its marker (`#[inline]` or `_Simple._`) becomes a lie, and any caller that was simple only because this callee was treated as free may itself no longer qualify. The original rule covered when to *add* the marker; it said nothing about when to *strip* it.

**Rule:** When an edit makes a previously-simple fn non-simple, strip its `#[inline]` attribute or `_Simple._` doc tag in the same edit. Then `rg <fn-name>` for callers and re-test each by the recursive simple definition; de-mark any caller that no longer qualifies. Cascade until quiescent. For trait methods: if a new conforming impl (hand-written or codegen) cannot be simple, strip `_Simple._` from the trait *declaration*, not the impl — the contract is what changed.

**Why:** A stale `_Simple._` is actively misleading because callers count calls into the tagged fn as "free" in the recursive budget rule, so a stale tag silently mis-classifies callers as simple too. The cascade is cheap in practice — the simple class is small (leaves and trivial wrappers) and rarely goes more than 2 levels deep.

**How to apply:** When editing any fn body, re-run the recursive simple test on it before committing. If the marker no longer matches, strip it and walk callers with one `rg`. Reviewer/self-reviewer flags stale markers as REJECT/`major` because the misleading effect propagates.

**Escalated?** AGENTS.md, doc-convention, agent:review-findings, agent:self-review

### 2026-05-06 — code-style — `#[inline]` rule is recursive; three markers by fn shape

**What happened:** `ObjectExt::id` (and other simple wrappers, including generic methods on `Signal<Args>` / `ObjectRef<T>` / `WeakRef<T>` and codegen-driven trait methods such as `AsObject::object_base`) was left without `#[inline]` because the rule (a) counted source-level calls flatly with a "≤ 1 fn call" budget that did not see through other `#[inline]` callees, and (b) blanket-excluded generic / blanket-impl fns as "monomorphized, no benefit". Even after relaxing those, generic simple fns and trait method declarations had no visible marker, so the recursive budget rule was not auditable from a fn or trait surface.

**Rule:** "Simple" is recursive: no branches/loops AND ≤ 1 call to a non-simple fn (apply transitively to each callee). Mark every simple fn / trait method by shape:
- concrete simple fn → `#[inline]` attribute (the existing marker);
- generic simple fn (or simple-bodied default method on a generic blanket impl) → `_Simple._` doc tag, italic line below the summary, before any `#` heading;
- trait method whose every conforming impl is required to be simple (typically codegen-driven — `AsObject::*`, `Object::{meta_object, connect_signal}`) → `_Simple._` doc tag on the trait method declaration. Hand-written impls of a tagged trait method MUST be simple; non-simple impl violates the trait contract.

Markers are mutually exclusive by shape. Codegen mirrors the rule: emit `#[inline]` for concrete generated fns; emit `/// _Simple._` for generic generated fns and for generated trait-method docs whose impls are always simple.

**Why:** The "≤ 1 fn call" budget was too tight because it counted calls to other `#[inline]` fns, even though those calls collapse during inlining. Without recursion, simple wrappers around simple wrappers were missed. The blanket "exclude generic / blanket-impl" exclusion missed an entire class of simple fns whose bodies LLVM can absolutely inline. The doc tag exists because `#[inline]` on a generic fn is redundant (monomorphization already makes the body cross-crate-available), so there was no source-visible marker for "this generic call is free in the budget rule" until now.

**How to apply:** When writing or reviewing a new fn, run the recursive simple test on it; if it passes, add the marker that matches its shape. When writing a trait method, ask "must every conforming impl be simple?" — if yes (because every impl is generated by a derive in this workspace, or because the contract demands it), tag the declaration. Do not tag a trait method whose contract admits non-simple impls (e.g. `Object::read_property`, which has a `match`).

**Escalated?** AGENTS.md, doc-convention, agent:review-findings, agent:self-review

### 2026-05-06 — code-style — extract body of `impl Into<T>` shells into a nested `fn inner`

**What happened:** `ObjectTree::rename(impl Into<String>)` (`quartzite-runtime/src/object_tree.rs:318`) ships its full ~30-line body in one copy per concrete `T` callers pass — `&str`, `String`, `&String`, `Cow<'_, str>`, etc. The same pattern repeats in `ObjectFactory::register`, `Timer::named`, `ObjectBase::named`. Lifting the body out is the standard-library idiom (see `Path::new`, `PathBuf::from`, `OsStr::new`), but a sibling `*_inner` impl method pollutes the type namespace.

**Rule:** When a public fn takes `impl Into<T>` / `impl AsRef<T>` / `impl ToString` and the body is > 3 lines, place the body in a nested `fn inner(...)` defined inside the outer fn (NOT as a sibling method); the outer fn carries the `_Simple._` doc tag and only does the conversion + forwards to `inner(...)`. Promote to a sibling private fn only when a second caller actually needs it. Skip when the type parameter is used throughout the body (genuine generic logic — e.g. `F: FnOnce(...)` invoked inside the body).

**Why:** Only the trivial conversion shell is monomorphized; the body ships once. Nesting keeps the helper out of the impl namespace, makes it impossible to misuse from elsewhere, and matches the `std` idiom. A nested `fn` cannot capture `self`; pass it explicitly as `this: &mut Self`.

**How to apply:** When adding or reviewing a public fn with a conversion-style generic param, count the body lines; if > 3, extract into a nested `fn inner(...)`. Reviewer: flag missing extraction as `nit` (binary-size cost may be accepted with rationale).

**Escalated?** AGENTS.md, agent:review-findings, agent:self-review

### 2026-05-06 — process — do not object to "breaking API change" on an unpublished crate

**What happened:** Code review finding #2 (dynamic `SingleShot` slot leak) was objected to with "breaking API change" as the reason. The user rejected this immediately — the project has not been published to crates.io and AGENTS.md explicitly allows freely changing the public API before the first `cargo publish`.

**Rule:** "Breaking API change" is not a valid objection reason on this codebase until after the first `cargo publish`. API can be freely renamed, removed, or restructured. When a finding is technically fixable but would require an API change, implement the change rather than deferring.

**How to apply:** Before objecting to any finding citing API stability, check AGENTS.md § "API Stability". If the crate is unpublished, remove that objection and implement the fix.

**Escalated?** AGENTS.md

### 2026-05-03 — process — /interview has no "too small for a spec" off-ramp; flag and ask instead of silently switching to implementation

**What happened:** `/interview errors should iml Error` was started. Mid-interview the user redirected: "ok, add the two impl blocks under the same cfg." Instead of completing the interview (producing a spec) or explicitly flagging the pivot, execution switched silently to direct implementation. The `/interview` skill produced code changes rather than a spec.

**Rule:** When a task arrives via `/interview` (or `/task`) but turns out to be trivially small during the interview, do not silently abandon the spec and implement. Flag explicitly: "This is small enough to implement directly — want me to skip the spec and just make the change?" Wait for the user to confirm the mode switch before writing any code.

**How to apply:** If the interview answers reveal the entire change is < ~20 lines with no design decisions, pause and offer to skip the spec rather than silently pivoting. The user stays in control of when the workflow gets bypassed.

**Escalated?** skill:interview

### 2026-05-03 — process — commit and push without waiting for explicit approval when inside a /task workflow

**What happened:** All rename changes were complete, verified, and ready to push, but no commit or push was made until the user explicitly asked. The stated reason was the global system instruction "only create commits when requested." The user clarified that this instruction does not override the /task workflow, which already authorizes commits at Steps 8 and 12.

**Rule:** Within a `/task` workflow, committing after each subtask (Step 8) and committing + pushing at Step 12 are part of the authorized workflow — no additional user prompt is needed. The global "ask before committing" instruction applies to ad-hoc work outside a task, not to steps explicitly prescribed by the task skill.

**How to apply:** If currently executing a `/task` (progress file exists or task was just completed), follow the step instructions for committing and pushing. Only pause to ask if something is ambiguous or risky beyond what the task step covers.

**Escalated?** skill:task

### 2026-05-03 — process — include ai-docs/learnings.md in the PR commit when it changes during the task

**What happened:** `ai-docs/learnings.md` was modified while working on a PR but not staged or committed — leaving learnings outside the PR diff and not reviewed alongside the code change that prompted them.

**Rule:** If `ai-docs/learnings.md` is modified during work on a PR (Step 8–11 or a follow-up push), stage and commit it together with the related code changes. Learnings are part of the task deliverable and should be visible in the PR for review.

**How to apply:** Before any `git commit` during a PR task, check `git diff --name-only ai-docs/learnings.md`. If it shows changes, include the file in the commit being staged.

**Escalated?** AGENTS.md

### 2026-05-03 — process — update PR title and body after commits that change public API or scope

**What happened:** After renaming `emit_unchecked`/`emit_checked` → `emit`/`emit_unless_blocked`, the PR title and body still referenced the old names. The user had to prompt explicitly to update them.

**Rule:** After any `git push` that changes public API names, scope, or acceptance criteria, immediately update the PR title and body with `gh pr edit` before reporting done. The PR description is part of the deliverable — it must stay in sync with the actual implementation.

**How to apply:** At the end of Step 12 (or any follow-up push), check whether the PR title/body mentions any symbol, AC, or scope item that the new commits changed. If yes, run `gh pr edit --title "..." --body "..."` before posting the PR URL to the user.

**Escalated?** skill:task

### 2026-05-03 — code-style — `_unchecked` is reserved for `unsafe` fns; default name is the safe variant

**What happened:** Renamed `Signal::emit` → `Signal::emit_unchecked` and added `Signal::emit_checked` (which consults the `blocked` flag). Neither function is `unsafe`. This conflicts with `std` ecosystem convention: `_unchecked` is reserved for `unsafe` fns whose invariants the caller must uphold to avoid UB (e.g. `slice::get_unchecked`, `str::from_utf8_unchecked`). The natural unsuffixed name should be the safe, ergonomic default — using `_unchecked` for "skips an unrelated runtime check" misleads readers and reviewers and removes the ergonomic reward for the safe path.

**Rule:** API naming follows `std`:
- Default `do_something()` is safe/checked. `do_something_unchecked()` is the `unsafe` companion (must include a `# Safety` doc section listing invariants).
- If the safe default panics, add `try_do_something()` (preferred) or `do_something_checked()` returning `Result`/`Option`.
- For non-safety "with/without runtime behavior X" variants, pick descriptive names — do not co-opt `_unchecked`/`_checked`.

**Escalated?** AGENTS.md, agent:review-findings, agent:self-review

### 2026-05-03 — process — breaking public API changes are allowed before first crates.io release

**What happened:** Suggested keeping `Signal::emit` for backward compatibility. User clarified: the project has no downstream clients yet; API can be freely broken until the first release to crates.io.

**Rule:** Do not add backward-compat shims, deprecation layers, or keep old names "for compatibility" while the crate has not been published to crates.io. Rename, remove, or restructure public API freely. Note the release milestone in decisions when it matters.

**Escalated?** AGENTS.md

### 2026-05-02 — code-style — let chains are allowed and formattable in edition 2024

**What happened:** During the macros task, rustfmt errored on a let chain with "let chains are only allowed in Rust 2024 or later". The workspace uses `edition = "2024"`, Rust 1.95, and rustfmt 1.9.0 — all of which support let chains. The error was caused by running rustfmt without `--edition 2024` explicitly, or against a stale binary. The response was to replace let chains wholesale with match expressions as a blanket rule.

**Rule:** Let chains (`if let A = x && let B = y { ... }`) are valid in this codebase. Do not avoid them. Run `rustfmt` via `cargo fmt` (which picks up the workspace edition automatically) rather than invoking `rustfmt <file>` directly.

**Escalated?** AGENTS.md

### 2026-05-02 — process — do not touch IDE files unless explicitly asked

**What happened:** `.idea/quartzite.iml` had an uncommitted modification. Without being asked, it was added to `.gitignore` and removed from tracking.

**Rule:** Never add, remove, modify, or `.gitignore` IDE files (`.idea/`, `*.iml`, `.vscode/`, etc.) unless the user explicitly asks. Treat them as the user's domain.

**Escalated?** AGENTS.md

### 2026-05-02 — process — "submit to PR" means push to remote, not merge

**What happened:** User said "submit to pr". Interpreted as merging the PR via `gh pr merge`. User meant pushing the local commits to the remote branch so they appear in the open PR.

**Rule:** "Submit to PR" (and similar: "push to PR", "add to PR") means `git push` the branch to remote. It does not mean merging. Only merge when the user explicitly says "merge" or "merge the PR".

**Escalated?** AGENTS.md

### 2026-05-02 — process — "wtf" signals that the previous action was wrong

**What happened:** User said "add ide files". Interpreted as adding IDE files to `.gitignore`. User meant commit and track them. User responded "wtf?" to signal the action was wrong.

**Rule:** "wtf" (and similar expressions of surprise/frustration) means the last action was the opposite of what the user wanted. Stop immediately, ask what went wrong, and do not proceed until the intent is clarified.

**Escalated?** AGENTS.md

### 2026-05-02 — process — never use git reset --hard; use soft reset, stash, cherry-pick, or backup branch

**What happened:** `git reset --hard origin/master` was used to move commits off local master to a feature branch. This discarded uncommitted changes to `ai-docs/learnings.md` that had not been staged.

**Rule:** Never use `git reset --hard`. Use one of these instead:
- `git reset --soft HEAD~N` — moves commits back to staged, preserves working tree
- `git stash` — saves uncommitted changes before switching branches
- `git cherry-pick` — moves specific commits to another branch
- Backup branch — `git checkout -b backup/...` before any destructive operation

**Escalated?** AGENTS.md

### 2026-05-02 — process — always create a feature branch before committing; never commit directly to master

**What happened:** When the user said "submit PR", commits were already on local master. Instead of creating a feature branch first, `git push` was run directly against master — pushing the commits to origin/master. `master` is branch-protected (no force push), so the commits could not be removed after the fact and a proper PR became impossible.

**Rule:** When work is intended for a PR, create a feature branch (`git checkout -b feat/...`) *before* making any commits. Never commit to local master with the intention of later turning it into a PR.

Recovery — if commits were accidentally made on local master and not yet pushed (full procedure in `AGENTS.md`):
1. `git stash` — save any uncommitted changes
2. `git checkout -b feat/...` — branch off from current HEAD (carries the commits)
3. `git checkout master && git reset --soft origin/master && git restore --staged .` — soft-rewind local master to remote state without discarding work
4. Push the feature branch and open the PR from it; pop the stash on the feature branch if needed
5. **Never push master** — not even as an intermediate step. **Never use `git reset --hard`** — see the dedicated rule above.

Before any `git push`: run `git branch --show-current` and confirm it is **not** `master`. If it is master — stop, do not push, apply the recovery procedure above.

If "submit PR" is requested and commits are already pushed to origin/master: stop and tell the user — there is no recovery without a force push, which branch protection may block.

**Escalated?** AGENTS.md

### 2026-05-02 — process — create feature branch before committing at the start of Step 8

**What happened:** The auto-connection task completed all implementation steps on the working tree without ever committing. Only after the user asked "why didn't you create a PR?" was the branch created. The changes had to be recovered by checking out a feature branch from the unstaged state.

**Rule:** At the start of Step 8 (Implementation), immediately create a feature branch (`git checkout -b feat/...`) before writing any code. Record the branch name in the progress file. Do not wait until after self-review to create the branch.

**Escalated?** skill:task, hook (PreToolUse on `git commit` blocks master)

### 2026-05-02 — testing — any sufficiently large file requires unit tests

**What happened:** Three codegen files (`object/codegen.rs`, `object_impl/codegen.rs`, `meta_enum/codegen.rs`) were written without `#[cfg(test)]` modules. Gaps were caught in review and by the user. The original rule was codegen-specific, but the user generalised it: any file with substantial logic needs tests.

**Rule:** Any file with ~50+ lines of non-trivial code must have a `#[cfg(test)] mod tests` block. This applies equally to codegen, parse, util, and any other module — not just files named `codegen.rs`.

**Escalated?** AGENTS.md

### 2026-05-02 — process — propagate skill/agent fixes to all related files in the same operation

**What happened:** A fix to `self-review.md` was applied in isolation; `codebase-review.md` was only updated after the user pointed it out. Similarly, `/task` and `/task-issue` fixes were done together, but the code-review family was handled piecemeal.

**Rule:** When fixing a skill or agent, immediately propagate the change to all files in the same sync group before reporting done. Two sync groups:
- **Task group:** `skills/task/SKILL.md` ↔ `skills/task-issue/SKILL.md`
- **Review group:** `skills/code-review/SKILL.md` (workflow orchestrator) ↔ `agents/review-findings.md` (findings producer) ↔ `agents/self-review.md` (fix validator)

Note: `code-review` is a **skill** (user-facing workflow); `review-findings` and `self-review` are **agents** spawned by it. Do not refer to any of these as "code-review agent" — that conflates the skill with an agent. (A `diff-review` agent existed historically but was removed as orphan; do not reintroduce it without wiring it into a skill.)

**Escalated?** AGENTS.md

### 2026-05-02 — process — self-review must not re-run cargo fmt or cargo clippy

**What happened:** The self-review agent checked `cargo fmt -- --check` and raised REJECT findings for formatting drift, even though both `cargo fmt` and `cargo clippy -- -D warnings` are already mandated after every subtask during Implementation (Step 8 of /task and /task-issue). This caused a spurious round-trip.

**Rule:** Self-review must not run or re-check `cargo fmt`, `cargo clippy`, `cargo build`/`check`, or `cargo test`. These are all guaranteed by the Implementation and Verify steps before self-review runs. Self-review scope: spec conformance, design conformance, test coverage, safety/correctness, style (Rust file conventions, allow attributes) — not build tooling.

**Escalated?** agent:self-review, agent:review-findings

### 2026-05-02 — process — propagate rule exemptions to agent/skill files in same task

**What happened:** When adding `quartzite-examples` exemptions to `AGENTS.md` (no `#![deny(missing_docs)]`, no `#[cfg(test)]`), the corresponding checks in `.claude/agents/self-review.md` and `.claude/agents/review-findings.md` were not updated. Future reviews would have incorrectly flagged the examples crate.

**Rule:** When a rule exemption is added to `AGENTS.md`, immediately propagate it to every agent/skill/settings file that enforces that rule. Check with `grep` across `.claude/agents/` and `.claude/skills/` before closing the task.

**Escalated?** AGENTS.md

### 2026-05-02 — process — check current branch before committing, not only before pushing

**What happened:** For the `docs/learnings-and-skill-fix` branch, commits were made directly to local master without checking `git branch --show-current` first. The error was caught at push time (branch protection rejected the push), not at commit time. The rule in AGENTS.md mentions checking before `git push`, but the correct mental model is: verify branch before `git commit`.

**Rule:** Run `git branch --show-current` and confirm it is **not** `master` before any `git commit` that is intended for a PR. A pre-push check is a last resort, not the primary safeguard. The commit should never happen on master — the push check only exists as a final gate.

**Escalated?** hook, skill:task

### 2026-05-02 — process — run cargo fmt --all after every code change, including post-self-review fixes

**What happened:** `cargo fmt --all -- --check` was run once during Step 9 (Verify). A self-review finding then triggered a code fix (Step 11). The fix was committed and pushed without re-running `cargo fmt --all`. CI failed on the formatting drift introduced by that fix.

**Rule:** Run `cargo fmt --all` (and re-check with `cargo fmt --all -- --check`) after *every* code change — including fixes made after self-review. The verify step (Step 9) is not a one-time gate; it must be re-run after any subsequent edit before committing. Never commit without a clean `cargo fmt --all -- --check` immediately before the commit.

**Escalated?** hook

### 2026-05-03 — code-style — use stdlib methods instead of explicit branching for comparisons

**What happened:** In `Rect::united` / `RectF::united`, explicit `if` branches were written to compute min/max of edge coordinates. The review comment pointed out `.min()`/`.max()` should be used instead.

**Rule:** Use `.min()`, `.max()`, `.clamp()`, and other stdlib methods instead of `if`/`match` branching for simple value comparisons. This applies equally to all numeric types (`i32`, `f32`, etc.). Explicit branches are harder to read and error-prone.

**Escalated?** AGENTS.md

### 2026-05-03 — architecture — do not reference other frameworks as justification for design choices

**What happened:** Design decisions for geometry types were described as "consistent with Qt's Qt::NoButton = 0" in analysis. The user explicitly said not to copy or reference other frameworks; we are writing a Rust framework from scratch using language strengths.

**Rule:** Never cite other frameworks (Qt, GTK, WinForms, etc.) as justification for design choices. Design based on Rust idioms, crate ecosystem norms, and explicit reasoning about the problem. "Other library does X" is not a valid argument.

**Escalated?** AGENTS.md

### 2026-05-03 — process — delete progress file immediately on self-review APPROVE, before Step 12

**What happened:** Self-review Round 2 returned APPROVE. Proceeded directly to Step 12 (finalize INDEX.md, commit, PR) without deleting the progress file first. The file was left as an untracked artifact after the PR was created.

**Rule:** On self-review APPROVE, delete `.progress.md` as the *first action* before starting Step 12. The task skill is explicit: "On APPROVE: delete `.progress.md` → proceed to Step 12." The file is transient handoff state; it must not outlive the task.

**Escalated?** skill:task
**Superseded by:** PR #300 — rule reversed; the `/task` progress file is now gitignored, persists in the working tree across `/pr-commented` rounds, and is deleted by `/pr-merged` after the PR merges.

### 2026-05-03 — process — do not ask about backward compatibility; AGENTS.md already prohibits compat shims

**What happened:** During an interview, asked "should `#[object_meta]` be kept as a deprecated no-op alias (soft removal) or fully deleted?" — despite AGENTS.md explicitly stating "The project has not yet been published to crates.io and has no downstream clients. Public API may be freely renamed, removed, or restructured without backward-compat shims or deprecation layers." The same lesson was already in learnings.md (2026-05-03 "breaking public API changes are allowed") and escalated to AGENTS.md.

**Rule:** Never ask the user about backward compatibility, deprecation layers, or keeping old names for compatibility. Read AGENTS.md before formulating interview questions. If a rule is already documented there, apply it silently — do not re-ask.

**How to apply:** Before asking any interview question, check whether AGENTS.md already answers it. Questions about API stability, compat shims, deprecation, and release timing are all answered there.

**Escalated?** skill:interview

### 2026-05-02 — process — verify relative markdown links before committing

**What happened:** Files generated in `ai-docs/deferred/` used `../../plans/done/` as the relative path to `ai-docs/plans/done/`. The correct path is `../plans/done/` — one level up from `ai-docs/deferred/` reaches `ai-docs/`, then `plans/done/` is a sibling of `deferred/`. GitHub rendered the broken paths as `/plans/done/` (repo root), which does not exist.

**Rule:** After generating files with relative links, verify at least one link manually: trace the path on disk (`realpath` or mental directory traversal) before committing. From `ai-docs/deferred/file.md`, one `..` reaches `ai-docs/`; two `../..` reaches the repo root.

**Escalated?** AGENTS.md

### 2026-05-05 — code-style — `clippy::doc_markdown` allowlist scope + heuristic limits

**What happened:** Initial doc-convention seed populated `clippy.toml`'s `doc-valid-idents` with ~60 entries: every project type (`MouseEvent`, `ObjectBase`, `BitFlags`, …), third-party types (`IndexMap`, `RwLock`), and `no_std`. The reasoning was "avoid noise from `clippy::doc_markdown` false positives during the audit." On review the user pointed out this was the wrong default — Rust identifiers in prose should always be backticked; the allowlist is for *non-code* tokens only. Shrinking to just `GPU` surfaced only **3 violations** workspace-wide. **A follow-up empirical test then showed that even `GPU` was dead code:** the `doc_markdown` heuristic flags `CamelCase` identifiers (`HelloWorld` → warning) but does NOT flag pure all-caps acronyms regardless of length (`GPU`, `REALLYLONGACRONYM` → no warning). The entire `clippy.toml` was deleted as a result.

**Rule:** `doc-valid-idents` entries must be genuine non-code tokens **AND** must be `CamelCase` / mixed-case names (the only shape the heuristic catches). **Never add Rust type names, function names, module names, third-party crate types, or build-config tokens like `no_std`** to the allowlist — backtick them inline. **Never add pure all-caps acronyms** like `GPU`, `JSON`, `URL` — the heuristic already ignores them. Realistic candidates are `CamelCase` brand names or proper nouns the project genuinely uses in prose without intending them as code (e.g. a product name).

**How to apply:** Default response to a `doc_markdown` warning is "add backticks", not "allowlist the name". Before adding any entry, verify the token actually fires the lint by removing it temporarily and re-running clippy with `--all-targets`. The workspace currently ships **no `clippy.toml`** — only resurrect it if a real non-code `CamelCase` token surfaces.

**Escalated?** doc-convention (`ai-docs/doc-convention.md` *Linking and code references* + *Lints* sections)

### 2026-05-05 — architecture — prefer AtomicBool + safe OnceLock over AtomicPtr + unsafe for process-global accessors

**What happened:** The `global_tree` process-global accessor used `static TREE_PTR: AtomicPtr<Mutex<ObjectTree>>` with an `unsafe` dereference inside `try_with_tree`. The user pointed out that the `AtomicPtr` + `unsafe` design is unnecessary: since `APP: OnceLock<Arc<ApplicationInner>>` already holds the tree for the process lifetime, a simple `AtomicBool TREE_LIVE` flag (set by `Application::new`, cleared by `Drop`) is sufficient. `try_with_tree` then checks the bool, calls `APP.get()?`, and locks safely — no raw pointers, no unsafe blocks.

**Rule:** When implementing a process-global accessor that tracks whether a singleton is "live", prefer `AtomicBool` (or a similar safe primitive) over `AtomicPtr` to raw memory. Reach for `unsafe` only when there is no safe alternative. An `OnceLock` or `Weak<T>` already in scope can often provide the pointer without raw pointer manipulation.

**How to apply:** Before writing an `AtomicPtr`-based global, ask "can I express the same semantic with a `bool` flag plus an already-existing `OnceLock`?" If yes, use that. Reserve `AtomicPtr` for cases where the pointee's lifetime genuinely cannot be tracked through existing safe constructs.

**Escalated?** AGENTS.md

### 2026-05-05 — process — context-reset and self-review agents must check `#[inline]` on every simple new fn

**What happened:** The context-reset subagent implementing Tasks 3–11 of the timer-object task wrote `impl AsObject` and `impl Object` methods (`object_base`, `object_base_mut`, `as_any`, `as_any_mut`, `meta_object`, `invoke_method`, `connect_signal`) without `#[inline]`. The self-review agent's Round 1 pass also missed the gap, only flagging it in Round 2. The user had to point it out explicitly before the PR commit.

**Rule:** Both the context-reset implementing agent and the self-review agent must check `#[inline]` on every new simple, non-generic function — not only on named groups like "getters" or "Default impls". Every `AsObject`/`Object` impl method, every trivial wrapper, every constructor that is just a struct literal, must be reviewed for this annotation before the task is reported done.

**Why:** The AGENTS.md `#[inline]` rule is unambiguous and applies broadly. Context-reset agents must apply it at write time. Self-review agents must validate it independently on every new public fn.

**How to apply:** At implementation time, add `#[inline]` immediately before writing the body of any simple fn. At self-review time, scan every `fn` in the diff for missing `#[inline]` before reporting APPROVE — not just fn groups the reviewer already knows about.

**Escalated?** agent:self-review

### 2026-05-05 — architecture — use existing derive macros for `AsObject`/`Object`; do not write manual impls

**What happened:** The context-reset subagent implementing the Timer type wrote manual `impl AsObject for Timer` and `impl Object for Timer` instead of using `#[derive(Extend, Object)]` from `quartzite-macros`. The design document explicitly specified the derive approach. Adding `quartzite-macros` as a dependency of `quartzite-runtime` has no circular dependency (quartzite-macros only depends on `syn`/`quote`/`proc-macro2`), so there was no obstacle. The manual impl caused a 2-round review cycle to surface the `#[inline]` gap the macro would have generated automatically.

**Rule:** When a type in `quartzite-runtime` (or any crate with access to `quartzite-macros`) needs to implement `AsObject` and `Object`, use `#[derive(Extend, Object)]` + `#[object_impl]`. Do not hand-write the impls. The derive macros generate correct `#[inline]` annotations, property dispatch, and MetaObject registration automatically.

**Why:** The derive macros exist precisely to eliminate boilerplate. Using them ensures consistency, correctness, and automatic compliance with all codegen conventions. Rolling a manual impl is reinventing the wheel and drifts from the generated pattern.

**How to apply:** When adding `#[derive(Extend, Object)]` to a crate that doesn't yet depend on `quartzite-macros`, add the dep first (`quartzite-macros = { path = "../quartzite-macros" }`). Proc-macro crates are never circular.

**Escalated?** no

### 2026-05-05 — code-style — use `.ok()?` not `.unwrap()` on `Mutex::lock` in library code

**What happened:** `try_with_tree` used `mutex.lock().unwrap()`, which panics on a poisoned mutex. Since AGENTS.md mandates non-panicking APIs for libraries, and mutex poisoning (another thread panicking while holding the lock) is not a broken global invariant from the caller's perspective, returning `None` is the correct behaviour. `.lock().ok()?` achieves this with no additional code.

**Rule:** In library code, use `mutex.lock().ok()?` (or `mutex.lock().unwrap_or_else(|e| e.into_inner())` when you want to recover the inner value) rather than `.unwrap()`. Panicking on mutex poisoning is not appropriate for a library — callers should decide how to handle a "tree unavailable" result.

**How to apply:** Any time you write `something.lock().unwrap()` in a function that returns `Option` or `Result`, replace with `.lock().ok()?`. Reserve `.unwrap()` for cases where poisoning truly indicates an unrecoverable program invariant failure.

**Escalated?** AGENTS.md

### 2026-05-05 — process — never ask whether a library API should panic for an avoidable error

**What happened:** During interview for #55 (parent/children accessors), asked "should `parent()` / `children()` panic or return a default when called outside an Application scope?" — AGENTS.md already answers this: "Prefer non-panicking APIs for libraries; panicking is acceptable only when a fundamental invariant is broken." Being outside an Application scope is a recoverable condition, not a broken global invariant, so `None`/empty is the correct answer by rule.

**Rule:** Never ask the user whether a library function should panic for an avoidable error condition. Read AGENTS.md first — the non-panicking default is already mandated. Only ask about panic behavior if the scenario involves a genuinely broken global invariant (e.g., internal data structure corruption, double-free).

**How to apply:** Before formulating interview questions, check whether AGENTS.md already resolves the question. "Should X panic or return None/Err?" is almost always answered by the non-panicking library API rule — apply it silently.

**Escalated?** skill:interview

### 2026-05-05 — process — filter to unresolved PR review threads before reading comments

**What happened:** When the user said "I've commented gh pr", all inline comments were fetched and read — including ones already resolved by the reviewer in a prior session. Time was wasted re-reading and re-resolving already-closed threads.

**Rule:** Always fetch PR review threads filtered to `isResolved: false` before reading comments. Use the GraphQL query:
```
reviewThreads(first: 20) { nodes { id isResolved comments(first:1) { nodes { databaseId body } } } }
```
then filter `isResolved == false` before reading any comment bodies.

**How to apply:** Start every "I've commented gh pr" workflow with the GraphQL unresolved-threads query, not the REST `/pulls/{N}/comments` endpoint (which returns all comments regardless of resolution state).

**Escalated?** no

### 2026-05-05 — process — always run the PR body check after every push, even if no edit seems needed

**What happened:** After pushing AGENTS.md + learnings.md changes to the open PR branch, the PR body check (`gh pr view <N>`) was skipped on the grounds that instruction-only commits can't affect code claims. The rule is unconditional: re-read first, then decide. Reasoning your way out of the check is the failure mode the rule prevents.

**Rule:** After every push to a branch with an open PR, always run `gh pr view <N>` and read the body. Only then decide whether an edit is needed. Never skip the read — only skip the edit if the body is still accurate.

**How to apply:** Post-push checklist: `gh pr view <N> --json title,body`. If the body matches reality → done. If not → `gh pr edit`. The cost is one command; the benefit is catching invisible drift.

**Escalated?** AGENTS.md, skill:task, hook (PostToolUse Bash pr-sync reminder in `.claude/settings.json`)

### 2026-05-05 — process — resolve fixed review comments; leave objected ones for the reviewer

**What happened:** After applying fixes and posting an objection reply to a PR review comment, neither type of comment was resolved on GitHub. The user clarified the correct rule: resolve only comments that were fixed; leave comments where an objection was posted so the reviewer can decide whether to accept the objection.

**Rule:** After pushing fixes to a PR:
- Comments addressed by a code fix → resolve on GitHub (`gh api … -X PUT … {"resolved": true}` or via the UI).
- Comments where a reply was posted explaining why no change was made (objection) → leave unresolved; it is the reviewer's call to accept or push back.

**How to apply:** After any `git push` that addresses review feedback, iterate over the closed-out comments and resolve only the ones that have a corresponding code change. Objection replies are the reviewer's domain to close.

**Escalated?** AGENTS.md

### 2026-05-05 — code-style — use thiserror for error types; apply undocumented_unsafe_blocks to every crate

**What happened:** `TreeAccessError` was initially hand-rolled with manual `Display` / `std::error::Error` impls. PR review requested `thiserror = "2"` be added. Separately, `#![warn(clippy::undocumented_unsafe_blocks)]` was added only to `quartzite-runtime` when first introduced; review comment pointed out it should be in every crate's `lib.rs`.

**Rule:** Use `thiserror` for any new error enum/struct in this workspace — avoids boilerplate `Display` / `Error` impls. Add `#![warn(clippy::undocumented_unsafe_blocks)]` to every crate `lib.rs` (next to the other clippy attributes), not just the crate where unsafe was first introduced.

**How to apply:** When adding a new crate-level lint attribute, immediately propagate it to all other crate `lib.rs` files in the same PR. When defining an error type, reach for `thiserror` first.

**Escalated?** AGENTS.md

### 2026-05-05 — process — keep PR description in sync after every push to an open PR

**What happened:** While iterating on PR #83 (doc-convention) after it was already open, we landed two follow-up commits — first tightening the "backtick every Rust identifier" rule and shrinking `clippy.toml` to one entry, then deleting `clippy.toml` entirely after an empirical test showed the heuristic ignores all-caps tokens. The original PR body still claimed "New workspace-root `clippy.toml` with a ~60-entry `doc-valid-idents` allowlist…" and the AC6 test-plan line still said "seeded; no growth needed". Neither was true after the follow-ups. The PR description was not synced until the user explicitly asked. The `/task` skill *does* spell out this rule in Step 11 ("If the fixes changed any public API name, scope, or AC referenced in the PR title/body (and the PR is already open), run `gh pr edit --title ... --body ...` to bring the PR description in sync before pushing"), but the rule applies to *any* push that invalidates a claim in the body — not only Step 11 review-fix commits.

**Rule:** **After every push to a branch with an open PR, re-read the PR body (`gh pr view <N>`) and ask "is anything here now wrong?".** If yes, sync via `gh pr edit <N> --title "..." --body "..."` *before* the next action. Sync triggers include: scope items added or dropped; renames or removals of files / types / fns named in the body; AC checkbox status flips; numbers cited in the body that drifted (test count, lint count, allowlist size, file count, etc.). Routine commits — typos, formatting, refactors strictly within already-described scope — do not need a body edit. The **upstream tracking issue's** title and body are the user's original problem statement and **must not be rewritten**; communicate scope changes via an issue comment instead, and rely on the PR's `Closes #N` line to surface the merging PR via GitHub's auto-link.

**How to apply:** Add a "re-read PR body" step to your post-push checklist whenever the open-PR-on-branch state holds. Cheapest shape: `gh pr view <N> --json title,body | rg -i '<thing-you-just-changed>'` — if any hit, the body needs an edit. The cost of an extra `gh pr edit` is minutes; the cost of a reviewer reading a body that contradicts the diff is wasted reviewer trust.

**Escalated?** AGENTS.md, skill:task, hook (PostToolUse Bash pr-sync reminder in `.claude/settings.json`)

### 2026-05-05 — process — `gh pr view` not needed immediately after `gh pr create`

**What happened:** After `gh pr create` for PR #85, ran `gh pr view 85` to "apply the new unconditional read rule". The body returned was exactly what was just authored a second earlier — the read was wasted work. The "PR body sync after every push" rule treats `gh pr create` as if it were a subsequent push that might have invalidated the body, but in fact the body and the push are authored together: there is nothing to discover.

**Rule:** The "PR body sync after every push" rule does not apply to the push that immediately precedes `gh pr create`. The body at that moment is what you just authored; nothing to discover. The rule starts firing on the *next* push to the branch.

**How to apply:** After `gh pr create` returns the URL, stop — do not re-read the body until at least one further `git push` happens. From the *second* push onward, the unconditional read rule applies.

**Escalated?** AGENTS.md

### 2026-05-05 — process — backward-compat question asked again despite skill:interview escalation

**What happened:** During `/interview` for issue #36, asked "should the current `Timer` construction/usage API stay roughly the same (just gaining object-tree integration), or is a full redesign expected?" — i.e., a backward-compat framing. The rule was already escalated to `skill:interview` on 2026-05-03 ("do not ask about backward compatibility; AGENTS.md already prohibits compat shims"). User rightly pushed back again.

**Rule:** Never frame an interview question around backward compatibility, keeping old APIs, or preserving existing behavior "for users". AGENTS.md is clear: no crates.io release, no downstream clients, free to rename/remove/restructure. Apply this silently before any question round.

**How to apply:** Before every interview round, re-read AGENTS.md § API Stability. If a candidate question touches compat, deprecation, or "keeping old X", discard it and apply the rule silently.

**Escalated?** skill:interview

### 2026-05-05 — tooling — use `0.x` version format for 0.x.y deps, not bare `0`

**What happened:** Added `tracing = "0"` and `itertools = "0"` to Cargo.toml. AGENTS.md rule is "use `0.x` for `0.x.y` versions — never pin the patch." The correct forms are `tracing = "0.1"` (tracing is 0.1.x) and `itertools = "0.14"` (itertools is 0.14.x). Bare `"0"` is overly broad; it would accept any 0.x.y release including incompatible minor versions.

**Rule:** Check the resolved minor version (from `cargo update` output or crates.io) and use `0.x` — e.g. `"0.1"`, `"0.14"`. `"0"` is wrong because Cargo treats 0.x as breaking per minor version; `"0.x"` pins to that minor line.

**How to apply:** When adding a new 0.x.y dep, look at the version that `cargo update` resolves (e.g., `tracing v0.1.44`) and write `"0.1"`, not `"0"`. Same applies to `{ version = "0.x", ... }` inline tables.

**Escalated?** no

### 2026-05-05 — process — switch to feature branch BEFORE editing files in `/improve` (and similar skills)

**What happened:** During `/improve`, all instruction-file edits were applied while on `master`. Only at commit time was the branch-switch made reactively (`git checkout -b chore/...`). AGENTS.md says "create a feature branch before any commits" — that was technically respected (no commits on master), but the spirit (don't accumulate work on master) was broken. `/task` already gates this at Step 8; `/improve` had no equivalent gate.

**Rule:** Skills that produce commits intended for a PR (currently: `/improve`; future: any new skill in the same shape) must check `git branch --show-current` and create a feature branch *before any file edit*, not just before commit.

**How to apply:** In `/improve` Step 5 (Apply), the first action — before any Edit/Write — is the branch check. If on `master`, `git checkout -b chore/YYYY-MM-DD-improve-<short-name>` carries the working tree over. Edit only after the branch is in place.

**Escalated?** AGENTS.md, agent:self-improve

### 2026-05-06 — code-style — never use .expect() on mutex locks or condvar waits in production code

**What happened:** All `.lock().expect("... poisoned")` and `condvar.wait().expect("...")` calls in `ThreadDriver`, `AppDriver`, and `PoolDriver` were written as panicking. Reviewer flagged all three sites asking why panicking behavior was there and whether it was avoidable.

**Rule:** Mutex poisoning in library code is recoverable. Use `.lock().unwrap_or_else(|e| e.into_inner())` (or `.ok()?` in Result-returning fns) instead of `.expect("... poisoned")`. The `AGENTS.md` library safety idioms section already states this explicitly. Also remove any `# Panics` doc sections from methods that no longer panic after the fix.

**How to apply:** On any new driver, scheduler, or shared-state type: whenever a `Mutex::lock()` or `Condvar::wait()` call appears in production code (i.e., not in tests), use `unwrap_or_else(|e| e.into_inner())` by default. Reserve `.expect("reason")` only for cases where poisoning genuinely means an unrecoverable invariant violation (document why in the reason string).

**Escalated?** hook (PostToolUse Edit/Write panic-gate in `.claude/settings.json`)

### 2026-05-06 — code-style — `.expect()` on mutex/condvar/Option is still a panic; checklist must catch it explicitly

**What happened:** `timer.rs` was implemented with `.expect("... mutex poisoned")` on every `Mutex::lock()`, `Condvar::wait()`, and `BinaryHeap::peek()`/`pop()` call in `ThreadDriver`, `AppDriver`, and `PoolDriver`. Two rounds of self-review (Rounds 2 and 3) both approved without catching these. The user had to flag all five sites via PR comments.

**Why the gate broke — implementor side:** AGENTS.md § *Library safety idioms* says "use `mutex.lock().ok()?` or `.unwrap_or_else(|e| e.into_inner())`". The implementor read the *general* test rule ("no `unwrap()` in production code; `expect("reason")` preferred") and concluded that spelling out a reason string was sufficient. The mutex-specific rule was applied silently only to the `Mutex<Signal>` from `quartzite-core` (which was correctly handled via `ok()?`) but not to the driver-owned mutexes.

**Why the gate broke — self-review agent side:** The reviewer checklist item was worded as "No `unwrap()` in production code (only in `.expect("reason")` form with justification)." Because `.expect(...)` is literally a different method name, the grep mental model (`grep unwrap`) missed it entirely. The agent never checked `.expect()` calls for panicking soundness — only for the presence of a justification string.

**Rule:** Both implementor and self-review agent must treat `.unwrap()`, `.expect()`, and `panic!()` as the same category — *unconditional panics* — and apply the same scrutiny to all three. For every occurrence in production code (outside `#[cfg(test)]`), ask: "Is there a non-panicking form?"

Concrete substitutions:
- `mutex.lock().expect(...)` → `mutex.lock().unwrap_or_else(|e| e.into_inner())`
- `condvar.wait(g).expect(...)` → `condvar.wait(g).unwrap_or_else(|e| e.into_inner())`
- `condvar.wait_timeout(g, d).expect(...)` → `condvar.wait_timeout(g, d).unwrap_or_else(|e| e.into_inner())`
- `option.expect("logically guaranteed")` → `let Some(x) = option else { continue/return; };`
- `result.expect("infallible")` → `result.unwrap_or_else(|e| /* handle or return */)`

**How to apply — self-review agent:** After checking for `unwrap`, also `grep -n '\.expect(' src/` (excluding test modules) and verify each call. Acceptable forms: `.expect()` in tests, `.expect()` where poisoning means a genuine broken global invariant with the reason string explaining *why* it is unrecoverable. Anything else is a finding.

**Escalated?** agent:self-review, hook (PostToolUse Edit/Write panic-gate in `.claude/settings.json`)

### 2026-05-06 — process — "add to learnings" means learnings only; do not propagate to agent/skill files

**What happened:** User said "add to learnings: unwrap/expect/panic should be avoided... with explanations how this quality gate is broken." I wrote the learnings entry, then also updated `.claude/agents/self-review.md` and `.claude/agents/review-findings.md` citing the Propagation Rule. The user flagged this as unauthorized — they asked for one action (learnings), not three.

**Rule:** "Add to learnings" is a request to write to `ai-docs/learnings.md` only. The `Escalated?` field in the learnings entry records that an issue *is a candidate* for escalation to an agent/skill/AGENTS.md — it is not authorization to make that escalation immediately. The Propagation Rule in AGENTS.md only triggers when you are already editing an instruction file (`.claude/agents/**`, `.claude/skills/**`, `AGENTS.md`, `.claude/settings.json`). Creating a learnings entry is not editing an instruction file, so no propagation is triggered.

**How to apply:** When the user says "add to learnings" (or "note this", "remember this"), write the entry and stop. If the learnings entry merits escalation to project instructions, mark `Escalated? no` and let the user trigger `/improve` when ready. Never escalate to agents/skills on the same turn as a learnings-only request.

**Escalated?** AGENTS.md

### 2026-05-06 — process — `/next` skill cannot see "Blocked by:" in issue bodies; use a GitHub label instead

**What happened:** `/next small` recommended issue #48 (BlockingQueued) as a runner-up even though its body says "Blocked by: Per-thread event loops (#51, still open)". The skill fetches issues via `gh issue list --json number,title,labels,updatedAt` — it gets labels but never reads issue bodies. The "Skip blocked items" selection rule in the skill refers to the plan index (🔴 flags) and does not cross-reference GitHub issue bodies.

**Rule:** When a GitHub issue has a "Blocked by:" dependency on another open issue, add a `blocked` label to it on GitHub. The `/next` skill can then filter it out by label without fetching issue bodies. Do not rely on the "Blocked by:" section in the body being visible to the skill.

**How to apply:** After opening or triaging a new issue that depends on another open issue, immediately run `gh issue edit <N> --add-label blocked`. When the blocker is resolved, remove the label. This keeps the `/next` recommendation list accurate without requiring body reads.

**Escalated?** skill:next

### 2026-05-06 — process — resolve fixed PR review comments via GraphQL after pushing the fix

**What happened:** After fixing the panicking mutex ops (commits 543bb4f, 2ddece7), I replied to each comment but did not resolve the conversations. I attempted `resolveReviewThread` via GraphQL but used a guessed thread ID (`PRRT_kwDOSR5chs5UHUwU`) that did not exist, got NOT_FOUND, printed "resolve via graphql not available", and moved on. AGENTS.md says: "After pushing fixes, resolve only the comments that were addressed by a code change" — this was not done.

**Rule:** When a PR review comment is fixed by a code change, the conversation must be resolved. The flow is:
1. Reply to the comment explaining the fix.
2. Query for the actual thread node IDs:
   ```bash
   gh api graphql -f query='{ repository(owner:"OWNER", name:"REPO") { pullRequest(number:N) { reviewThreads(first:20) { nodes { id isResolved path } } } } }'
   ```
3. For each unresolved thread whose comment was addressed, call:
   ```bash
   gh api graphql -f query='mutation { resolveReviewThread(input:{threadId:"<id>"}) { thread { isResolved } } }'
   ```
4. Verify `isResolved: true` in the response.

When a GraphQL mutation fails with NOT_FOUND, do not silently move on — investigate: the thread ID was wrong, so query for the correct one first.

**How to apply:** In the post-push step after every PR fix round: reply first, then query for thread IDs, then resolve. If resolution fails, diagnose before giving up.

**Escalated?** AGENTS.md (GraphQL recipe added to "PR review comment resolution" bullet)

### 2026-05-06 — code-style — significant state-mutating fns should open with a debug! trace

**What happened:** `ObjectTree::rename` had a `debug!` call but `clear_name` was added without one. The reviewer caught the inconsistency.

**Rule:** Any function that makes a meaningful, traceable change to application state should open with a `debug!` call. This is at the writer's discretion — the threshold is "would someone debugging a live system want to see this in a log?" Minor field setters and high-frequency paths (e.g. every signal emit) do not qualify; high-frequency tracing should be gated behind a dedicated feature flag (e.g. `debug` or `trace-emit`) rather than always-on.

**Why:** Consistent traceability — when debugging lifecycle or state issues, log output should cover all significant mutations. Omitting a trace on one sibling method while its peers are traced breaks the log story.

**How to apply:** When adding a function that mutates non-trivial application state (tree mutations, lifecycle transitions, index updates, config changes): add a `debug!` at the top. When adding a sibling to an existing traced function, check whether it warrants the same treatment. Skip for: trivial getters/setters, emit paths, and any path called at high frequency without a feature-flag guard. Canonical feature name for high-frequency tracing: `verbose-tracing`.

**Escalated?** AGENTS.md, agent:self-review, agent:review-findings

### 2026-05-07 — process — actionlint skipped again on docs.yml despite existing learnings entry

**What happened:** `.github/workflows/docs.yml` was created, committed, and pushed without running `actionlint`. The user had to ask explicitly — same pattern as the previous entry (2026-05-07, line ~26). The existing learning is `Escalated? no` and has not been enforced.

**Rule:** Run `actionlint <file>` on every GitHub Actions workflow file created or modified, before staging. This is a required gate, same as `cargo build` and `cargo clippy`.

**Why:** Two occurrences in the same session with the rule already written confirms the rule needs to live in AGENTS.md and the task skill, not just in learnings.

**Escalated?** AGENTS.md, skill:task

### 2026-05-07 — process — do not skip design/design-review for "simple" tasks

**What happened:** `/task 116` was a pure annotation task (replace `#[inline]` with `/// _Simple._` at 12 sites). I skipped Steps 6–7 (design agent + design review) on the grounds that no design decision existed. The user called out the omission.

**Rule:** The `/task` workflow has no "too simple" exemption. Steps 6–7 are mandatory before Step 8 unless the user explicitly authorises skipping them. For annotation-only or trivially-scoped tasks, the design doc will be short — that is fine. The process cost of a one-paragraph design doc is lower than the cost of violating the workflow gate.

**How to apply:** Before writing any implementation code under `/task`, confirm that `ai-docs/plans/YYYY-MM-DD-name.design.md` exists and carries a GO verdict. If it does not exist, run the design agent (Step 6) regardless of how simple the task appears. Only skip with explicit user approval.

**Escalated?** no

### 2026-05-07 — process — bench binaries are exempt from the `#[cfg(test)]` requirement

**What happened:** Self-review flagged that `benches/macro_object.rs` (~90 lines) lacked a `#[cfg(test)] mod tests` block. AGENTS.md exempts only `examples/` explicitly. In practice, criterion bench binaries cannot host a test module — `criterion_main!` generates `main`, making the binary incompatible with the test runner.

**Rule:** Files under `benches/` are criterion bench binaries, not library code. Like `examples/`, they are runnable binaries and do not require a `#[cfg(test)] mod tests` block regardless of line count. Both existing workspace bench files (`quartzite-core/benches/signal_property.rs`, `quartzite-runtime/benches/object_tree.rs`) follow this pattern.

**Escalated?** AGENTS.md, agent:self-review, agent:review-findings

### 2026-05-08 — code-style — `quartzite_runtime::ApplicationError` has no `error` sub-module; it is re-exported at crate root

**What happened:** The `/task` prompt for Task 7 referenced `quartzite_runtime::error::ApplicationError` as the import path for the integration test. That module path does not exist — `ApplicationError` is re-exported directly via `pub use application::{Application, ApplicationError, TreeAccessError}` in `quartzite_runtime/src/lib.rs`, so the correct import is `use quartzite_runtime::ApplicationError`.

**Rule:** Before writing an import path for a type from another crate in this workspace, confirm the actual re-export path by reading the crate's `src/lib.rs`. Do not assume an `error` sub-module exists unless it is declared with `pub mod error` in that file.

**Escalated?** no

### 2026-05-08 — architecture — `WindowedApplication::new()` must return `Result<Self, RendererError>`, not `Result<Self, ApplicationError>`

**What happened:** The design doc said `new() -> Result<Self, ApplicationError>`, but `EventLoop::new()` can also fail with `winit::error::EventLoopError`, which is not convertible to `ApplicationError`. Returning `ApplicationError` would force either panicking on EventLoop error or silently discarding it. The correct approach is to return `Result<Self, RendererError>` and add an `Application(#[from] ApplicationError)` variant to `RendererError`, so both failure modes are propagated cleanly.

**Rule:** When a constructor wraps two independent fallible operations that produce different error types, the constructor's return error should be an encompassing error type (e.g., `RendererError`) with `#[from]` variants for each, not the error type of just one of them.

**Escalated?** no

### 2026-05-08 — tooling — `--workspace` clippy is required for leaf crates not in the default dep tree

**What happened:** `cargo clippy -- -D warnings` (without `--workspace`) passed for `quartzite-renderer` because it is a leaf crate not depended upon by the workspace root or any default-dep member. Running `cargo clippy --workspace -- -D warnings` caught a `clippy::must_use_candidate` lint on `VelloPainter::new()` that the per-default-tree run missed.

**Rule:** Always run `cargo clippy --workspace -- -D warnings` (not just `cargo clippy -- -D warnings`) to catch lints in leaf crates. The default-dep-tree clippy run is a subset, not a full check.

**Escalated?** AGENTS.md, skill:task, skill:bugfix, skill:code-review

### 2026-05-08 — process — update ai-docs/panic-index.md when introducing production panic sites

**What happened:** Graphics-stack (#73) introduced `WindowedApplication::run` which documents a platform-level panic (macOS main-thread requirement from `winit::run_app`) in its `# Panics` section. The panic-index was not checked or updated during implementation or at Step 9 (Verify). The omission was caught only after the PR merged.

**Rule:** At Step 9 (Verify), scan all new/modified production sources for two signals:
1. `grep "# Panics" src/**/*.rs` — documented panic behaviour (primary signal; always present when a panic exists).
2. `grep -n "\.expect\b\|\.unwrap\b\|panic!" src/**/*.rs` outside `#[cfg(test)]` — direct panic call sites.
For each hit, add an entry to `ai-docs/panic-index.md` (location, trigger, invariant, why not `Result`, preferred fix). Stage and commit `panic-index.md` with the implementation commit.

**Why not in design or as an AC:** The design phase cannot enumerate panic sites that don't exist yet. `# Panics` sections are the canonical indicator — they are written at implementation time, so the check belongs at Step 9 after the code exists.

**Escalated?** skill:task, agent:self-review, agent:review-findings

### 2026-05-08 — process — regenerate ROADMAP.md after every INDEX.md change

**What happened:** Updated `ai-docs/plans/INDEX.md` (marking graphics-stack ✅, unblocking widgets/paint-style) without re-running `scripts/gen-roadmap.sh`. The ROADMAP sync CI gate re-ran the generator, found a diff, and failed on PR #159.

**Rule:** Whenever `ai-docs/plans/INDEX.md` is modified, run `bash scripts/gen-roadmap.sh` and stage `ROADMAP.md` in the same commit. The CI gate enforces this — ROADMAP.md must always be in sync with INDEX.md at commit time.

**Escalated?** skill:task

### 2026-05-08 — code-style — generic-fn split (`fn inner`) requires a conversion-style generic param; don't apply it to parameter-less fns

**What happened:** `WindowedApplication::new()` in `quartzite-renderer/src/application.rs` wrapped its entire body in a nested `fn inner()` and called it, despite having no parameters at all. The generic-fn split pattern (AGENTS.md "Generic-fn split for binary size") exists solely to avoid monomorphization bloat from `impl Into<T>` / `impl AsRef<T>` / `impl ToString` generics. With no generic param, the indirection is dead weight — the body lands in one copy regardless, and the extra wrapper just adds noise.

**Rule:** Only apply the nested `fn inner()` split when the outer function has a conversion-style generic parameter (`impl Into<T>`, `impl AsRef<T>`, `impl ToString`) **and** the body is >3 lines. For non-generic fns (no params, concrete types only), write the body directly in the outer fn. Additionally, re-evaluate `#[inline]` after inlining: if the body now has >1 non-simple call, the fn no longer qualifies and `#[inline]` must be removed.

**Escalated?** no

### 2026-05-08 — code-style — `_Simple._` and `#[inline]` are mutually exclusive; drop `_Simple._` when `#[inline]` is present

**What happened:** Default trait method bodies in `WidgetExt` ended up with both `/// _Simple._` in the doc comment and `#[inline]` on the fn. Reviewer corrected: if a fn already has `#[inline]`, `_Simple._` is redundant and should be dropped. `_Simple._` is only for positions where `#[inline]` can't go (abstract trait method declarations with no body, generic impls).

**Rule:** `_Simple._` (in any form: `/// _Simple._`, `// _Simple._`) and `#[inline]` are mutually exclusive on the same fn. Use `#[inline]` on any fn that has a body and is in a concrete position (concrete fn, default trait method, `impl Trait for ConcreteFoo` method). Use `_Simple._` only where `#[inline]` doesn't apply: abstract method declarations (no body) and generic impl methods (`impl<T> Trait for Foo<T>`). Never annotate the same fn with both.

**Escalated?** code-style, agent:self-review, agent:review-findings

### 2026-05-08 — code-style — default trait method bodies need `#[inline]` in addition to `/// _Simple._` (superseded by next entry)

**What happened:** `WidgetExt` default method bodies (e.g. `fn geometry(&self) -> Rect { ... }`) carried `/// _Simple._` in their doc but no `#[inline]`. Reviewer flagged that the doc marker carries no optimizer hint. `#[inline]` was added alongside `/// _Simple._`.

**Rule (superseded):** Default method bodies in a `trait` definition that qualify as simple need both `/// _Simple._` and `#[inline]`. — **This was wrong.** See next entry: the two are mutually exclusive; drop `_Simple._` when `#[inline]` is present.

**Escalated?** code-style, agent:self-review, agent:review-findings
**Superseded by:** 2026-05-08 ("`_Simple._` and `#[inline]` are mutually exclusive") — corrected: the two markers are mutually exclusive; drop `_Simple._` when `#[inline]` is present. (The inline `(superseded by next entry)` note in this entry's title refers to the same correction; the formal field is the machine-readable record.)

### 2026-05-08 — code-style — use named constants instead of magic numbers

**What happened:** `Palette::default()` used raw `Color::new(0.94, 0.94, 0.94, 1.0)` etc. literals inline. Reviewer flagged them as magic numbers.

**Rule:** Extract numeric literals that carry semantic meaning into named `const` values. The name documents the intent; the literal is an implementation detail. This applies to color values, sizes, timeouts, limits — any number that isn't self-evidently 0 or 1.

**Escalated?** no

### 2026-05-08 — process — learnings are append-only; never edit or remove existing entries

**What happened:** A learning entry was deleted because a newer entry contradicted it. User corrected: learnings are a permanent record — every entry stays, including superseded or wrong ones.

**Rule:** Never edit or delete an existing entry in `ai-docs/learnings.md`. Only append new entries. If a prior entry was wrong, write a new entry that corrects it (cross-referencing the old one if helpful). The history of corrections is itself valuable.

**Escalated?** AGENTS.md

### 2026-05-08 — process — on corrections, write to learnings only; do not update instruction files

**What happened:** When the user pointed out `_Simple._` and `#[inline]` should not coexist, the response was to update `code-style.md` and `AGENTS.md` directly. User corrected: corrections go to `ai-docs/learnings.md` only. Instruction files are updated exclusively by `/improve`.

**Rule:** When the user corrects a behaviour or clarifies a rule, append to `ai-docs/learnings.md` and stop. Do not touch `AGENTS.md`, `ai-docs/code-style.md`, `ai-docs/doc-convention.md`, `.claude/agents/**`, or `.claude/skills/**` in the same turn. Those files are updated only when the user runs `/improve`.

**Escalated?** AGENTS.md

### 2026-05-07 — process — do not escalate learnings inline during `/task`; leave `Escalated? no` for `/improve`

**What happened:** During `/task 143`, I wrote a learnings entry and immediately escalated it by editing `AGENTS.md`, `.claude/agents/self-review.md`, and `.claude/agents/review-findings.md` in the same commit. The user corrected: escalation is `/improve`'s job.

**Rule:** When writing to `ai-docs/learnings.md` during `/task` (or any non-`/improve` skill), always set `Escalated? no`. Do not touch `AGENTS.md`, `.claude/agents/**`, or `.claude/skills/**` on the basis of a new learnings entry. The Propagation Rule only triggers when you are *already* editing an instruction file for a separate reason — it does not authorize pre-emptive escalation. Run `/improve` when ≥3 unescalated entries accumulate.

**Escalated?** AGENTS.md

### 2026-05-08 — process — verify a GitHub Action's actual behaviour against its source; "the action defaults match" is not a free claim

**What happened:** PR #179 (`/task #178`) added `mozilla-actions/sccache-action@v0.0.10` to five merge-gate compile jobs in `.github/workflows/ci.yml`. The spec and design both stated: *"No `env:` block, no `with:` block on the sccache-action step is required — the action's defaults (`SCCACHE_GHA_ENABLED=true`, `RUSTC_WRAPPER=sccache`) match the spec's GHA-backed decision and the leave-default cache-size decision."* That claim was **false**. The action's `src/setup.ts` exports only `SCCACHE_PATH`, `ACTIONS_CACHE_SERVICE_V2`, `ACTIONS_RESULTS_URL`, `ACTIONS_RUNTIME_TOKEN` — it does **not** set `RUSTC_WRAPPER` or `SCCACHE_GHA_ENABLED`. The README's "Rust code" subsection says these "should be set" — explicitly the user's responsibility. After PR #179 merged, sccache binary was installed in every compile job but cargo never invoked it, so AC4 (sccache stats visible in logs) silently failed and the cache stayed empty. PRs #180 and #181 (docs-only) wouldn't have benefited from sccache anyway, but on any source-touching PR we'd have paid setup overhead with zero compile savings.

The user caught this by asking why subsequent PRs weren't faster. Reading the action's source code took ~2 minutes — should have been part of registry-query.

**Rule:** **The registry-query rule (AGENTS.md § Dependency Versions) extends beyond version pinning. When integrating a third-party GitHub Action whose behaviour is load-bearing (i.e., whose claimed defaults are part of your design), verify the actual behaviour against the action's source — typically `action.yml` plus `src/main.ts` / `src/setup.ts` — not from the action's marketplace blurb, training-data memory, or even the README's "happy path" example.** The README often lists conditional / advanced setup separately (in this case "Rust code" had its own subsection mandating env vars); skipping past those subsections is the failure mode.

Concrete check at task time:

```bash
# 1. Pull the action's setup script
gh api /repos/<owner>/<action>/contents/src/setup.ts --jq '.content' | base64 -d \
    | grep -inE 'exportVariable|process\.env|GITHUB_ENV|saveState'

# 2. Pull action.yml
gh api /repos/<owner>/<action>/contents/action.yml --jq '.content' | base64 -d
```

If the action's setup script doesn't export the env vars your design assumed it does, set them explicitly in your workflow (per-job `env:` block or `echo >> $GITHUB_ENV` after the action step). Don't rely on "the action probably sets it" without source verification.

**Why this fits the existing registry-query rule:** the rule's stated motivation is *"Training-data version knowledge is months stale by default and treating it as authoritative has, in this repo, twice put the wrong major into a spec the user then had to correct."* The same logic applies one level deeper — the action's *behaviour* is also stale in training data (or the README skim), and treating it as authoritative without source verification put the wrong claim into the spec and design that the user had to correct.

**How to apply — for `/task` flows that integrate a third-party Action:**

- During the design phase, when the design assumes the action's defaults match the spec, include a registry-query-equivalent verification step that reads the action's `setup.ts` / `main.ts` for `exportVariable` / `GITHUB_ENV` writes.
- Cite the source-line evidence in the design's "Implementation steps" section, not just the README's claim.
- During the self-review phase, the reviewer checks: "did the design verify the action's behaviour against source, or only against README narrative?"

**Escalated?** AGENTS.md

### 2026-05-10 — documentation — enable all feature-gated optional features when running cargo doc, and in package.metadata.docs.rs, to get full doc coverage

**What happened:** The CI docs job ran `cargo doc --no-deps --workspace` without `--features serde`. Intra-doc links in `quartzite-core/src/lib.rs` that pointed to items in the `serde`-gated `snapshot` module (`[`snapshot`]`, `[`snapshot::ObjectSnapshot`]`, `[`snapshot::TreeSnapshot`]`) failed to resolve, breaking the docs build.

**Rule:** When a crate uses `#[cfg(feature = "...")]` to gate public modules or items, all three of the following must include those features or intra-doc links silently break under `-D warnings`:
1. The CI `cargo doc` command in `.github/workflows/ci.yml` (`--features serde`).
2. The GH Pages publish command in `.github/workflows/docs.yml` (`--features serde`) — same command, same flag, same reason.
3. `features = [...]` in `[package.metadata.docs.rs]` of every crate whose docs reference those items (controls docs.rs builds).

All three must be kept in sync whenever a new optional feature adds public API with intra-doc links.

**Escalated?** AGENTS.md, doc-convention, skill:task, agent:self-review, agent:review-findings

### 2026-05-09 — process — /task with a bare issue number should activate a matching deferred spec instead of starting a fresh interview

**What happened:** `/task 47` was invoked. Issue #47 has a matching deferred spec at `ai-docs/plans/deferred/2026-05-01-paint-style.spec.md` (`**Tracked in:** #47`). Instead of moving that spec to `ai-docs/plans/` and confirming ACs with the user, the interview machinery was started and a spurious state file `ai-docs/plans/2026-05-09-paint-style.spec.md.state.md` was created.

**Rule:** The `/task` deferred-activation keyword check ("activate", "start", "proceed") fires only on those literal words in the arg. When the arg is a bare issue number, load the issue body first, then check whether any deferred spec already has `**Tracked in:** #N` matching that issue. If found, treat it as "already have a spec" — move it to `ai-docs/plans/`, update `INDEX.md`, confirm ACs with the user, and skip to Step 6. Do not run the interview and do not create a state file.

**Escalated?** no

### 2026-05-10 — documentation — prefer inline form `[`Foo`](path)` over reference form `[`Foo`][path]` for intra-doc links

**What happened:** Post-merge code review of PR #200 (issue #199 docs cleanup) surfaced 24 intra-doc links across 8 files using the CommonMark **reference-style** form `` [`Foo`][crate::path::Foo] `` instead of the **inline** form `` [`Foo`](crate::path::Foo) ``. Both forms render to identical HTML, but the workspace was inconsistent — 100+ inline-form sites against 24 reference-form sites concentrated in the snapshot module (one author's stylistic choice).

**Rule:** Use the inline form `` [`Foo`](path::Foo) `` for intra-doc links throughout the codebase. This matches the dominant convention in the Rust ecosystem — `std`, the rustdoc book examples, tokio, serde, and most popular crates use the inline form. Reference form is the minority style; reserve it only when the link target legitimately contains characters that would break the inline form (rare for Rust paths). When auditing or writing new doc comments, pick inline; when touching a file with reference-form links, convert them in the same edit.

**Why:** consistency makes the codebase navigable; aligning with the ecosystem default reduces cognitive load for newcomers; the only tooling-equivalent variation is form (rendered output is identical), so the choice is purely stylistic and there's no reason to retain two forms.

**How to apply:** when writing or reviewing doc comments, the intra-doc link form is `` [`Type`](crate::path::Type) `` — text in `[]`, target in `()`. Reject the reference form `` [`Type`][crate::path::Type] `` in review unless the target genuinely cannot be expressed inline.

**Escalated?** no

### 2026-05-11 — testing — a local FAILED result was not investigated before pushing; CI caught the same failure

**What happened:** During `/task #53` (multi-window support), the first local `cargo test` run returned `FAILED. 27 passed; 1 failed`. Instead of identifying the specific failing test and fixing it, a second run was performed (`--nocapture`) which returned `ok. 28 passed; 0 failed`. The "green" second run was accepted as proof the failure was transient and the code was pushed. CI then failed on the same test (`ac7_builder_exists_and_build_works` on Ubuntu; `build_result_is_ok_or_already_exists` on macOS and Windows): both called `winit::EventLoop::new()` on `cargo test` worker threads, which winit forbids on all platforms unless `with_any_thread(true)` is passed (Linux-only API). The local second run appeared green because the user's machine had a display session AND the Wayland/X11 check may have been satisfied, but CI worker threads did not have that environment.

**Rule:** When `cargo test` returns FAILED, always identify the specific failing test (`grep "FAILED"` on the output) and reproduce it in isolation (`cargo test test_name_substring -- --nocapture`) before deciding it was transient. A subsequent green run does not prove the failure was transient — different test-thread assignments or environment variables (DISPLAY, WAYLAND_DISPLAY) can flip the result. Only accept "transient" when the test is known flaky (e.g. timing-sensitive) and multiple reruns are consistently green.

**Why:** Tests that construct `winit::EventLoop` (directly or via `WindowedApplicationBuilder::build()`) panic on worker threads on macOS and Windows, and on Linux without `with_any_thread(true)`. A local display session may suppress the Linux panic incidentally; CI worker threads do not have this luxury. The correct fix is `#[cfg(target_os = "linux")]` + `.with_any_thread(true)`, or to avoid calling `build()` in any test that runs on a worker thread.

**How to apply:** Any test — unit or integration — that calls `WindowedApplicationBuilder::build()` or any API that internally creates a `winit::EventLoop` must either: (a) be gated `#[cfg(target_os = "linux")]` and pass `.with_any_thread(true)`, OR (b) avoid calling `build()` in the test body entirely (test only builder field state, not `build()` completion). Never push after accepting a second-run green that followed a first-run FAILED without identifying why the first run failed.

**Escalated?** no

### 2026-05-13 — process — design-review notes not resolved in the design document before implementation started

**What happened:** Design review Round 2 issued a GO verdict but included two notes: (1) use `b.kind()` method (not `b.kind` field — it's private); (2) use `quartzite_widgets::Alignment` (not `quartzite_geometry::Alignment`) to avoid adding a non-dev dep. Both notes were applied correctly during implementation, but they were never written back into the design document before coding started. The user asked why the two notes were not fixed before implementation began.

**Rule:** Any note, even minor/note-severity, emitted by a design-review GO verdict must be incorporated into the design document before writing the first line of implementation code. "Applied during coding" is not the same as "resolved in the design" — the design document is the authoritative implementation contract. If the design doc says one thing and the code does another (even correctly), the design is stale and future reviewers see a contradiction.

**How to apply:** After receiving a design-review GO verdict, scan the verdict text for notes or minor issues. For each one: (a) update the relevant section of the design document (API table, helper list, risk table, etc.); (b) verify the design doc now matches the intended code. Only then proceed to implementation. This takes < 5 minutes and prevents spec/design divergence from the first commit.

**Escalated?** agent:design-review, agent:self-review, skill:task

### 2026-05-13 — process — `/triage` lost classification state between subagent invocations

**What happened:** A `/triage` run executed in three subagent turns. Turn 1 (agent `a11b5ffeee8bc82c6`) walked all 51 `Tracked = —` candidates, classified them into 35 declines + 15 promotes, and surfaced the decline table for user spot-check. The user approved with one tweak (move row L179 from decline to promote). Turn 2 spawned a fresh `triage-runner` instance, prompted to "apply your prior classifications" — but the new instance had no memory of turn 1's partition (subagents do not share context, and no progress file persisted the state). It correctly halted with a "classification list not recoverable from current context" report instead of guessing which 34 of the 35 declines to write. Turn 3 worked around the gap by re-pasting the full classification table into the next subagent prompt verbatim — recoverable only because the parent (this assistant) still had the table in its own context. A compaction or context-reset would have forced a full reclassification pass (~10 min of work + risk of inconsistent decisions).

**Rule:** `/triage` should create a progress file (e.g. `ai-docs/triage/triage-YYYY-MM-DD.progress.md`, local-only / gitignored, schema mirroring [[ai-docs-templates-progress-format]]) at the start of any multi-turn run. The `triage-runner` subagent writes its Phase 4 dedupe map summary, Phase 4.5 bridge classifications, and Phase 6/7 candidate partitions into the file as it produces them. Subsequent subagent invocations within the same `/triage` workflow read the file to restore context. When `/triage` completes (all writes landed, run summary emitted), the file is removed — same lifecycle as the `/task` and `/pr-commented` progress files (see [[ai-docs-plans-progress-files]]).

**Why:** subagent context isolation means classification state is unrecoverable across invocations unless persisted. Today's session got lucky because the parent retained the table verbatim; compaction would have erased it. The cost of a re-classification pass is ~10 min plus the risk that the second pass partitions differently from what the user approved — a silent divergence between user-approved plan and executed actions.

**How to apply:** when amending `.claude/skills/triage/SKILL.md` and `.claude/agents/triage-runner.md`, add (a) progress-file creation as Phase 1.5 (after the branch check, before Phase 2 dedupe), (b) progress-file resume read at Phase 1 if the file already exists for the current branch, (c) progress-file deletion after the final run summary emits, and (d) Propagation-Rule sync into the Triage group (skill ↔ agent ↔ `/next` skill). Mirror the cleanup-on-merge mechanic from `/pr-merged`'s `scripts/cleanup-progress.sh`.

**Escalated?** no
