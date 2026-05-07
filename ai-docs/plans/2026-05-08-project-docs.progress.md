# Progress: Project docs (#60)

**Branch:** `feat/2026-05-08-project-docs`
**base_commit:** `48324555a9aaa083ca8ee2b705c31ef60a861977`
**Issue:** #60
**Spec:** [`2026-05-08-project-docs.spec.md`](2026-05-08-project-docs.spec.md)
**Design:** [`2026-05-08-project-docs.design.md`](2026-05-08-project-docs.design.md)
**Design-review:** GO (Round 2 of max 3) — one informational Note: quickstart auto-emit comment isn't runtime-verified under `no_run`; implementer to spot-check.
**Last build:** clean (subtask 2; doc-gate, doctest, clippy, fmt, no-default-features all green)

## Files touched

| File | Subtask | Status |
|---|---|---|
| `README.md` | 1 | ✅ done |
| `src/lib.rs` | 2 | ✅ done |
| `CONTRIBUTING.md` | 3 | ⬜ pending |
| `scripts/gen-roadmap.sh` | 4 | ⬜ pending |
| `ROADMAP.md` | 5 | ⬜ pending |
| `.github/workflows/ci.yml` | 6 | ⬜ pending |

## Subtasks

| # | Task | Status |
|---|---|---|
| 1 | README description block + CI/license badges (badge order: CI, docs, codecov, license) | ✅ done |
| 2 | `src/lib.rs` rewrite: overview + `no_run` quickstart + 5 per-concept sections + ecosystem map + design notes; preserves `# Feature flags` + `document_features!()` | ✅ done |
| 3 | `CONTRIBUTING.md` at repo root, standard depth (10-section AGENTS.md excerpt) | ⬜ pending |
| 4 | `scripts/gen-roadmap.sh` (POSIX bash + awk/sed; comment banner of banned constructs) | ⬜ pending |
| 5 | Generated `ROADMAP.md` at repo root | ⬜ pending |
| 6 | CI sync-gate: `roadmap-sync` job + `roadmap-sync-pass` aggregator (ubuntu-latest only) | ⬜ pending |

> M = 6 subtasks (≥ 5). Per `/task` Step 8 rule, after subtask 3 hand off via `/context-reset`.

## Next action

Start **Subtask 3**: create `CONTRIBUTING.md` at repo root with the 10-section AGENTS.md excerpt per the design's *Open questions resolved → 4*. After this subtask, hand off via `/context-reset` per `/task` Step 8 rule (N=3 of M=6 ≥ 5).

### Notes from subtask 2

- Three intra-doc links needed explicit path qualifiers (`[Foo](core::Foo)`) because the implicit `[`Foo`]` form doesn't resolve from the facade scope. Fixed: `Object::read_property`/`write_property` (initially mis-attributed to `ObjectExt`), `Value`, `Object`. Dead-link gate caught all of them.
- Quickstart doctest required an explicit `fn main()` because rustdoc's auto-`fn main` wrapping puts the `Counter` struct inside `main()`, breaking the derive macro's generated `super::Counter` references. Comment in the doctest documents why.
- Auto-emit comment ("Writing the property emits count_changed automatically") preserved per design Round-2 note. Runtime semantics not verified under `no_run`; left as informational claim per AC9 contract (compile-only).
