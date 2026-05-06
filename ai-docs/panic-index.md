# Panic Index

Production `.expect()` / `unwrap()` / `panic!()` sites — tracked here for future hardening.

Each entry notes the location, the invariant being asserted, why a `Result`-returning alternative
was not used at the time, and the preferred fix when this is eventually hardened.

---

## Active entries

| # | File | Line | Call | Invariant asserted | Preferred fix |
|---|------|------|------|--------------------|---------------|
| P1 | `quartzite-runtime/src/event_loop.rs` | `spawn` closure | `.expect("newly spawned thread cannot have a pre-existing loop installed")` | `ThreadId` values are guaranteed never to be reused within a process lifetime (documented `std` contract), so a freshly spawned thread's `ThreadId` is guaranteed fresh and cannot already be in `LoopRegistry` | Replace `spawn` with a channel-based `try_spawn` that propagates `LoopAlreadyInstalled` synchronously: spawn thread → thread sends install result over `mpsc` → caller receives `Result<(Arc<Self>, JoinHandle<()>), LoopAlreadyInstalled>` |

---

## Notes

- **Why not `let _ = ...`?** AGENTS.md prohibits silenced `Result`s in production code without an explanatory comment. The `.expect()` was chosen over `let _ =` to make invariant violations loud rather than silent.
- **Why not a channel-based fix at the time?** The self-review loop caught the `let _ =` issue but not the availability of the channel alternative; the panic was the minimal fix that satisfied the linter.
- Entries should be removed once the preferred fix is implemented.
