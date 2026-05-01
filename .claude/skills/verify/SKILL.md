---
name: verify
description: "Run cargo test. Pass an optional filter to run a subset of tests."
argument-hint: "[test-filter]"
disable-model-invocation: true
allowed-tools: Bash(cargo test *)
---

Run `cargo test $ARGUMENTS`. If no arguments, runs the full test suite. Report any failures.
