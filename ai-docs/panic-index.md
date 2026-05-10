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

### `quartzite-renderer` — `RenderHarness::render_widget`

| Field | Value |
|---|---|
| **Location** | `quartzite-renderer/src/render_harness.rs` — `RenderHarness::render_widget` |
| **Trigger** | One of: (a) `vello::Renderer::render_to_texture` returns an error mid-render; (b) wgpu buffer mapping fails (`map_async` callback delivers `Err` or `Device::poll` returns `Err`); (c) the readback callback channel is dropped before the callback fires; (d) `image::RgbaImage::from_raw` returns `None` (would only happen if `width * height * 4` overflowed or the alignment math was wrong). |
| **Invariant** | The harness is test-only infrastructure: a successfully-constructed `RenderHarness` (which validated the GPU adapter, device, and texture allocation up front) can complete one render+readback cycle. A failure mid-cycle indicates a GPU driver fault or workspace bug, not a recoverable runtime condition. |
| **Why not `Result`** | The harness API is consumed by `#[test]` fns and the snapshot helper; both surface a panic identically to a `Result::Err` returned to the test harness. Surfacing `Result<RgbaImage, RendererError>` would force every snapshot-test call site to `.expect("…")` for no observable difference. The constructor (`RenderHarness::new`) does return `Result` because adapter/device acquisition failures are environmental and call sites legitimately want to skip vs. fail. |
| **Preferred fix** | Hoist the readback path into a `try_render_widget` returning `Result<RgbaImage, RendererError>` once snapshot tests need to discriminate "transient driver flake" from "real diff" (e.g. when CI flakes warrant retries). Until then, the panic is the right shape — tests fail loudly with the underlying error message. |

---

## Notes

- **Why not `let _ = ...`?** AGENTS.md prohibits silenced `Result`s in production code without an explanatory comment. The `.expect()` was chosen over `let _ =` to make invariant violations loud rather than silent.
- **Why not a channel-based fix at the time?** The self-review loop caught the `let _ =` issue but not the availability of the channel alternative; the panic was the minimal fix that satisfied the linter.
- Entries should be removed once the preferred fix is implemented.
