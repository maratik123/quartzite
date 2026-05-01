# Learnings

### 2026-05-02 — code-style — let chains are allowed and formattable in edition 2024

**What happened:** During the macros task, rustfmt errored on a let chain with "let chains are only allowed in Rust 2024 or later". The workspace uses `edition = "2024"`, Rust 1.95, and rustfmt 1.9.0 — all of which support let chains. The error was caused by running rustfmt without `--edition 2024` explicitly, or against a stale binary. The response was to replace let chains wholesale with match expressions as a blanket rule.

**Rule:** Let chains (`if let A = x && let B = y { ... }`) are valid in this codebase. Do not avoid them. Run `rustfmt` via `cargo fmt` (which picks up the workspace edition automatically) rather than invoking `rustfmt <file>` directly.

**Escalated?** AGENTS.md

### 2026-05-02 — testing — codegen files require unit tests like any other file

**What happened:** `object_impl/codegen.rs` (183 lines, 4 functions) was written without a `#[cfg(test)]` module. The code reviewer and user both flagged this. The `extend/codegen.rs` gap was caught in review; `object_impl/codegen.rs` only after the user asked.

**Rule:** Every `codegen.rs` file must have a `#[cfg(test)] mod tests` block before the PR is ready. Tests call `parse()` to build IR from `quote!` input, then call `codegen()`, then `assert!(out.to_string().contains(...))`. Cover: the main entry point, each emit helper, and at least the key branches (void vs typed return, empty vs populated collections).

**Escalated?** AGENTS.md
