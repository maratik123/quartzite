# Unsafe Index

Production `unsafe` sites — tracked here for future hardening.

Each entry notes the location, why the block (or `unsafe fn`) is unsafe, the
safety invariant the caller / surrounding code must uphold, why a safe-Rust
alternative was not used at the time, and the preferred fix when this is
eventually hardened.

---

## Active entries

### `quartzite-renderer` — `ActiveLoopGuard::drop`

| Field | Value |
|---|---|
| **Location** | `quartzite-renderer/src/wrapped_handler.rs:39` — `impl Drop for ActiveLoopGuard` |
| **Why unsafe** | Raw-pointer deref of a `*const Cell<*const ActiveEventLoop>` (`(*self.0).set(ptr::null())`). The guard holds a raw pointer rather than a reference so the borrow on `self.registry` does not persist across the winit callback boundary — `self.user_handler` and `self.registry` can be borrowed mutably while the guard is still live on the stack. |
| **Safety invariant** | The pointer was set by [`WrappedHandler::arm_active_loop`] from `&self.registry.active_loop`. `self.registry` is owned by the `WrappedHandler` which winit keeps alive for the entire `run_app` invocation; the guard is scoped to a single `ApplicationHandler` callback (always strictly shorter than the registry's lifetime). The pointed-to `Cell` therefore remains alive and valid for the full lifetime of the guard, on the same thread. |
| **Why not safe Rust** | The reference-based alternative — storing `&'a Cell<*const ActiveEventLoop>` in the guard — would require a lifetime parameter that ties the guard to `&self.registry`, which in turn prevents the caller from passing `&mut self.registry` to user code while the guard is on the stack. The whole point of the guard is to maintain a set-clear bracket *across* a borrow of `self.registry`, so a safe shared reference would defeat the design. |
| **Preferred fix** | If `winit::application::ApplicationHandler` ever exposes the `ActiveEventLoop` through a non-callback channel (e.g. a thread-local or a typed handle borrowable through the user-handler API), drop the raw-pointer slot entirely and rely on the safe handle. Until then, the set-clear-bracket pattern is the minimal-unsafety encoding of "transient slot only valid during a callback". |

### `quartzite-renderer` — `WindowRegistry::try_create_window`

| Field | Value |
|---|---|
| **Location** | `quartzite-renderer/src/window_registry.rs:160` — `WindowRegistry::try_create_window` |
| **Why unsafe** | Raw-pointer deref (`let event_loop = unsafe { &*ptr };`) of the `active_loop: Cell<*const ActiveEventLoop>` slot after a null check at line 152. The slot holds a transient pointer to a winit-owned `ActiveEventLoop` that is only valid while a winit `ApplicationHandler` callback is executing. |
| **Safety invariant** | See the field-doc `# Safety invariants` block (`window_registry.rs:66-84`) and the inline `// SAFETY:` comment (`window_registry.rs:155-159`) for the four numbered invariants (set-clear bracket maintained by `WrappedHandler` + `ActiveLoopGuard::drop`; `!Send + !Sync` enforced by the raw-pointer field + `PhantomData<*const ()>`; the resulting shared reference lives only for the body of `try_create_window`; null check precedes any deref). The index entry does NOT duplicate the four invariants — the doc block is the canonical source. |
| **Why not safe Rust** | winit's `ActiveEventLoop` is delivered to the host through `ApplicationHandler` callback parameters with a lifetime bound to the winit callback frame, not to `WindowRegistry`. There is no safe way to retain `&ActiveEventLoop` in the registry across a callback — the lifetime would be invalid the moment the callback returns. The raw-pointer slot + set-clear bracket is the minimal encoding of "available only during a callback, otherwise null". |
| **Preferred fix** | If winit's `ApplicationHandler` API ever permits passing `&ActiveEventLoop` *through* `&mut WindowRegistry` directly (e.g. by changing the callback signature to take both as bundled arguments to a user-supplied trait method that owns the lifetime relationship), refactor `try_create_window` to accept an explicit `&ActiveEventLoop` parameter and drop the slot. Alternative: encode the slot as a `thread_local!` once winit's callback signature permits — the underlying constraint (callback-scoped lifetime + single-threaded access) is unchanged. |

---

## Notes

- The **Safety invariant** field's `window_registry.rs:160` entry is phrased as a cross-reference to the in-source `# Safety invariants` doc block and inline `// SAFETY:` comment by design — duplicating the four numbered invariants here would create three drifting sources of truth. The cross-reference structure is the structural mitigation (see `ai-docs/plans/done/2026-05-16-unsafe-index.design.md` § Risks row "window_registry.rs:160 entry contradicts the existing in-source narrative bodies" for the rationale).
- **Why no `unsafe` block / `unsafe fn` taxonomy column?** With only two entries, free-form **Why unsafe** prose is sufficient. Revisit if the index grows past ~5 entries (see spec § Open questions).
- **Test-only `unsafe` is excluded.** Any `unsafe { … }` block inside `#[cfg(test)]` modules, `tests/`, `benches/`, or `examples/` is out of scope (e.g. the `unsafe { std::mem::transmute(n) }` at `wrapped_handler.rs:255` inside `#[cfg(test)] mod tests`). The catch-net grep recipe in `.claude/skills/task/reference.md` § Step 9 — unsafe-index sync (detail) surfaces them; reviewers walk hits and skip cfg-test sites.
- Entries should be removed once the preferred fix is implemented.
- Miri runs Tree Borrows on every master push over the FFI-free subset (see `.github/workflows/miri.yml`).
