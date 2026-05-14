# Dependency Versions — live-lookup reference

This page extracts the live-lookup table from [`AGENTS.md` § Dependency Versions](../AGENTS.md#dependency-versions). The AXIOM headline and pinning bullets stay in AGENTS.md.

## Why query live first

Whenever you write a specific version of a Cargo crate or a GitHub Action — anywhere (`Cargo.toml`, workflow file, issue body, spec, design doc, learning, any `ai-docs/**` page) — query the live source first. Treating remembered versions as authoritative has put wrong majors into specs twice in this repo (`criterion 0.5` vs. live `0.8`; `actions/deploy-pages@v4` vs. live `@v5`). The same logic applies one level deeper: a third-party GitHub Action's **behaviour** (what env vars it exports, what files it produces, which defaults it sets) is also stale in training data and in marketplace blurbs — treating it as authoritative landed the wrong claim into spec + design once (PR #179 sccache: "action sets `RUSTC_WRAPPER` and `SCCACHE_GHA_ENABLED` by default" — false; `src/setup.ts` only exports `SCCACHE_PATH` + cache-service vars, README's "Rust code" subsection explicitly mandates the user set them).

## Lookup table

| If you need to write... | Run this first |
|---|---|
| A Cargo crate version | `curl -sS "https://crates.io/api/v1/crates/<name>" \| jq -r '.crate.max_stable_version'` |
| A GitHub Action version | `gh api /repos/<owner>/<repo>/releases --jq '.[0].tag_name'` (and verify the action's Node runtime is current) |
| A version into a long-lived doc (won't be revisited for months) | Annotate `(verified current YYYY-MM-DD)` next to the version |
| A **load-bearing claim about an Action's behaviour** (env vars it exports, defaults it sets, files it produces — anything the spec or design relies on) | `gh api /repos/<owner>/<repo>/contents/action.yml --jq '.content' \| base64 -d` AND `gh api /repos/<owner>/<repo>/contents/src/setup.ts --jq '.content' \| base64 -d \| grep -inE 'exportVariable\|process\.env\|GITHUB_ENV\|saveState'` (or `src/main.ts` for run-step actions). Cite the source-line evidence in the design — README narrative alone is **not** evidence. |

Then apply the pinning rule (in AGENTS.md) to the **observed** version, never the remembered one. If `setup.ts` / `main.ts` does not export the env vars your design assumed, set them explicitly in the workflow (per-job `env:` or `echo >> $GITHUB_ENV` after the action step) — don't rely on "the action probably sets it".
