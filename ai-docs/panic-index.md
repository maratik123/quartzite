# Panic Index

Production `.expect()` / `unwrap()` / `panic!()` sites — tracked here for future hardening.

Each entry notes the location, the invariant being asserted, why a `Result`-returning alternative
was not used at the time, and the preferred fix when this is eventually hardened.

---

## Active entries

### `quartzite-renderer` — `WindowedApplication::run`

| Field | Value |
|---|---|
| **Location** | `quartzite-renderer/src/application.rs` — `WindowedApplication::run` |
| **Trigger** | Called from a non-main thread on platforms that require it (notably macOS). Enforced internally by `winit::event_loop::EventLoop::run_app`. |
| **Invariant** | winit's event loop must own the main thread on some platforms. |
| **Why not `Result`** | `run_app` panics at the platform level; the panic is not catchable or convertible to an error without a thread-spawn wrapper. Returning a `Result` for this case would require spawning a dedicated main thread, which is a larger architectural decision. |
| **Preferred fix** | Provide a `run_on_main_thread` helper (or document `#[cfg(target_os = "macos")]` guard) once multi-window / multi-platform support is added. Tracked implicitly under #53 (multi-window). |

---

## Notes

- **Why not `let _ = ...`?** AGENTS.md prohibits silenced `Result`s in production code without an explanatory comment. The `.expect()` was chosen over `let _ =` to make invariant violations loud rather than silent.
- **Why not a channel-based fix at the time?** The self-review loop caught the `let _ =` issue but not the availability of the channel alternative; the panic was the minimal fix that satisfied the linter.
- Entries should be removed once the preferred fix is implemented.
