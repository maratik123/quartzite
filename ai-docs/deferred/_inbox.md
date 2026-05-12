# Inbox

Untriaged rows extracted from completed plans' *Out of scope* / *Deferred* /
*Open questions* sections. Every row here is awaiting classification by
`/triage` (Issue B, [#204](https://github.com/maratik123/quartzite/issues/204))
— do not hand-edit.

This file is the universal landing zone for both forward-going propagation
(`/task` Step 12 appends one row per spec section after merging a plan) and
the one-shot backfill that seeded it. `/triage` drains rows by sorting each
into a thematic file (`signals-slots.md`, `ci-docs-workflow.md`, etc.),
promoting to a GitHub issue, or dropping with the literal `untracked`
decline-marker token written into the `Tracked` cell.

**Write discipline.** Hand-edits to this file are FORBIDDEN per the
`AGENTS.md` AXIOM (*Workflow* section, anchor `_inbox.md`) — only `/task`
Step 12 and `/triage` may write here.

**Schema.** 4-column markdown table. `Section` records which spec heading
the row was pulled from (`out-of-scope` / `deferred` / `open-question`).
`Tracked` mirrors cell 4 of the 8 thematic files — initially `—`,
rewritten to `#N` on promotion or `untracked` on decline by `/triage`.

| Item | Source | Section | Tracked |
|------|--------|---------|---------|
| Modal / parent-child window relationships (no `set_parent` / `is_modal` in this milestone). | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | out-of-scope | — |
| Multiple winit `EventLoop`s — single winit loop multiplexes all windows, matching how winit itself models multi-window apps. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | out-of-scope | — |
| Per-window `Application` instances. The process-singleton `Application` from `quartzite-runtime` remains a singleton; multi-window means many windows, not many `Application`s. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | out-of-scope | — |
| Non-winit / off-screen window backends. `RenderHarness` (offscreen) is unaffected and is not "a window". | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | out-of-scope | — |
| Cross-window focus orchestration policy beyond "the window that received the winit event owns dispatch". Cross-window tab traversal, click-to-focus across windows, and global focus tracking are deferred. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | out-of-scope | — |
| Window-level menu bars, dialogs, dock widgets — listed under `quartzite-widgets` v2 backlog (issue #46 carry-over). | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | out-of-scope | — |
| Window state persistence (size / position restoration across runs) — needs a settings layer | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | deferred | — |
| Multi-monitor placement APIs (`set_monitor`, fullscreen-on-display-N) — winit exposes the primitives but no widget-side consumer yet | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | deferred | — |
| Per-window cursor / IME state plumbing — not requested in issue #53 | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | deferred | — |
| Pluggable backend for headless / alternative windowing (smithay direct, sdl) — single-backend (winit) per #73 | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | deferred | — |
| Per-window scale-factor / DPI policy — Winit exposes `scale_factor`; the widget layout system does not yet consume it. Filed against widgets backlog. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | open-question | — |
| Window-level keyboard focus model across multiple windows on click-to-focus platforms — Outside the dispatch-routing scope of this milestone; needs a focus-state design that touches `quartzite-widgets`. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | open-question | — |
| Whether closed-window `WindowId` values may be re-issued — Winit guarantees uniqueness within a process; design phase confirms and documents. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | open-question | — |
| Whether `try_create_window` is sync or async — The current `WindowedApplication::run` is fully sync; design phase confirms (default: sync; winit `Window` creation is sync inside `ApplicationHandler::resumed`). | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | open-question | — |
| Exact handle shape exposed to user callbacks for calling `try_create_window` mid-loop — The design chose `WindowRegistry` threaded through `WindowedAppHandler` callbacks via `&mut`. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | open-question | — |
| Whether the existing `WindowedApplication::new()` constructor is retained as a shorthand for `builder().build()` or removed — Sugar question; design picks. Both options satisfy AC7. | ai-docs/plans/done/2026-05-11-multi-window-support.spec.md | open-question | — |
| Whether `WidgetRoot` should be folded into a closure adaptor instead of a named trait. Design picks: named trait for ergonomics. | ai-docs/plans/done/2026-05-11-multi-window-support.design.md | open-question | — |
| Whether `on_last_window_closed` is even useful in this milestone. Design picks: include it for AC4b test clarity. | ai-docs/plans/done/2026-05-11-multi-window-support.design.md | open-question | — |
| Whether `try_create_window` should accept window-level configuration (title, initial size, decorated/undecorated). Spec is silent; future spec adds a `WindowAttributes` arg. | ai-docs/plans/done/2026-05-11-multi-window-support.design.md | open-question | — |
| `WindowRegistry: !Send + !Sync` enforcement vs. ergonomics — blocks future cross-thread `try_create_window`; escape hatch is `EventLoopProxy<AppEvent>`. | ai-docs/plans/done/2026-05-11-multi-window-support.design.md | open-question | — |
| SVG / PDF export. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | — |
| Right-to-left / bidirectional text and complex shaping beyond what `parley`'s default `harfbuzz`-equivalent pipeline produces (BiDi-marked input is not specifically tested in this plan — basic LTR shaping is the AC target, matching the `paint-style` spec's "Basic LTR positioning only" stance). | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | — |
| A font-cache eviction policy. `parley::FontContext` keeps fonts loaded for the process lifetime; an eviction strategy is deferred until widget workloads expose pressure. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | — |
| Built-in image decoders — `Image` is still consumed as the raw RGBA8 buffer set up by `paint-style`. File / byte-stream decoding stays tracked under #282. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #282 |
| Sub-pixel font rendering — vello's default greyscale antialiasing is used; sub-pixel AA is a separate concern when it becomes available in vello. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | — |
| `BrushKind::LinearGradient` / `RadialGradient` rendering — variant already exists on `BrushKind` as `#[non_exhaustive]`; backend support tracked under #281. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #281 |
| `Image` source-rect cropping — tracked under #291. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #291 |
| New `Painter` trait methods. The trait surface is frozen by `paint-style`; this plan implements bodies only. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | — |
| New widget `WidgetExt::paint` overrides (e.g. drawing the actual `Label` text glyph run, `Button` chrome). Per `paint-style` and `gpu-snapshot-tests-ci`, widget-side `paint` implementations are a follow-up; this plan exercises `VelloPainter` directly via tests. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | — |
| `BrushKind::LinearGradient` / `RadialGradient` rendering — needs gradient-stop API + peniko `Gradient` wiring | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #281 |
| `Image` source-rect cropping — trait surface lacks a source rect; would require a `Painter` method addition | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #291 |
| Per-test perceptual-diff tolerance tuning — calibration once real pixels exist (mentioned in `gpu-snapshot-tests-ci` open questions) | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #286 |
| RTL / BiDi text and complex script shaping — tracked separately when a non-LTR widget surface lands | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | — |
| Font-cache eviction strategy for `parley::FontContext` — needs workload data | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | — |
| Exact stack-state probe mechanism for the transform / clip unit tests (call-counting probe vs. observing `vello::Scene` extent). | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | open-question | — |
| Exact API shape of the coordinate-space opt-out on `VelloPainter` (constructor variant `with_physical_pixels(...)` vs. `set_scale_factor(1.0)` setter) — both satisfy AC11/AC12; the design agent picks the more ergonomic shape. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | open-question | — |
| Per-OS golden divergence absorption strategy in `gpu-snapshot-tests-ci` (whether new `draw_text*` snapshots get `continue-on-error: true` until each runner bootstraps its golden, vs. seeding linux-only at merge and bootstrapping mac/win in follow-up PRs). | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | open-question | — |
| *(None remaining.)* Spec open questions (probe mechanism, opt-out API shape, snapshot bootstrapping policy) resolved in design: probe → `debug_stack_state` accessor; opt-out → `VelloPainter::with_scale`; bootstrapping → per-backend `draw_text*` with `continue-on-error: true`. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.design.md | open-question | — |
