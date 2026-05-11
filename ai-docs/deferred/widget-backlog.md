# Widget Backlog

Long-term reference of widgets to consider for `quartzite-widgets`. Inspired by
Qt 6's `QtWidgets` taxonomy but **not bound by it** — quartzite is its own
framework and may diverge in shape or paradigm.

This file is a backlog, not a roadmap. The umbrella issue for the first-pass
widget set is [#46](https://github.com/maratik123/quartzite/issues/46) and its
spec at [`ai-docs/plans/deferred/2026-05-01-widgets.spec.md`](../plans/deferred/2026-05-01-widgets.spec.md)
is authoritative for the first milestone. Items here outside that spec require
their own issue + spec when picked up.

## Status legend

- ✅ **first pass** — in scope of #46
- 🟡 **v2** — deferred to a follow-up issue, definitely planned
- 🤔 **undecided** — design call needed before scoping (paradigm question)
- ❌ **dropped** — explicitly will not implement
- 📭 **future** — interesting but no decision; revisit when need surfaces

## 1. Basic display

| Widget | Status | Notes |
|---|---|---|
| `Label` | ✅ first pass | text + alignment |
| `ProgressBar` | 🟡 v2 | tracked: #222 — depends on numeric range model |
| `TextBrowser` | 📭 future | rich-text + hyperlink navigation; needs rich-text engine |
| `LCDNumber` | ❌ dropped | retro-style 7-segment display; no compelling use case |

## 2. Buttons

| Widget | Status | Notes |
|---|---|---|
| `Button` (push) | ✅ first pass | text + checkable + signals |
| `CheckBox` | 🟡 v2 | tracked: #223 — trivial extension of `Button` once checkable groups work |
| `RadioButton` | 🟡 v2 | tracked: #224 — needs button group abstraction |
| `ToolButton` | 🟡 v2 | tracked: #225 — depends on `ToolBar` shell |
| `CommandLinkButton` | 📭 future | platform-specific styling; low priority |

## 3. Input

| Widget | Status | Notes |
|---|---|---|
| `LineEdit` | ✅ first pass | single-line text input |
| `TextEdit` | ✅ first pass | multi-line; rich text deferred to text engine v2 |
| `PlainTextEdit` | 🟡 v2 | tracked: #226 — optimised for large logs; specialisation of `TextEdit` |
| `SpinBox` | 🟡 v2 | tracked: #227 — integer numeric input |
| `DoubleSpinBox` | 🟡 v2 | tracked: #228 — float numeric input |
| `ComboBox` | 🟡 v2 | tracked: #229 — dropdown selection |
| `Slider` | 🟡 v2 | tracked: #230 — range value via drag |
| `Dial` | ❌ dropped | speedometer-style; rare in modern UIs |
| `DateEdit` / `TimeEdit` / `DateTimeEdit` | 📭 future | needs date/time model + calendar popup |
| `KeySequenceEdit` | 📭 future | shortcut capture widget; low priority |

## 4. Containers

| Widget | Status | Notes |
|---|---|---|
| `Container` | ✅ first pass | generic layout container with no chrome |
| `ScrollArea` | ✅ first pass | scrollable view |
| `GroupBox` | 🟡 v2 | tracked: #231 — titled frame around a layout |
| `TabWidget` | 🟡 v2 | tracked: #232 — tabbed pages |
| `StackedWidget` | 🟡 v2 | tracked: #233 — one-of-many visible; programmatic control |
| `ToolBox` | 📭 future | accordion-style; low priority |
| `Splitter` | 📭 future | drag-resizable child panes |
| `Frame` | 📭 future | base for bordered widgets; may not need a separate type |

## 5. Item views

| Widget | Status | Notes |
|---|---|---|
| `ListWidget` / `ListView` | 🤔 undecided | needs a Model/View decision (see below) |
| `TreeWidget` / `TreeView` | 🤔 undecided | same |
| `TableWidget` / `TableView` | 🤔 undecided | same |
| `ColumnView` | 📭 future | rare; only after `ListView` lands |

> **Paradigm question — Model/View vs alternative.** Qt's Model/View
> architecture (separate `QAbstractItemModel` + view widgets) is one option,
> but quartzite already has a property/reflection model and signals/slots that
> overlap with what Model/View provides. Before any item-view widget is
> implemented, a design pass must decide:
>
> - Adopt Qt-style `Model` traits with view widgets bound by `ObjectId`?
> - Lean on the existing property system + signals (`row_inserted`, etc.) and
>   give views a direct `&dyn AsObject` pointer?
> - Hybrid — a thin `ItemModel` trait that wraps property/signal access?
>
> No item-view widget should be implemented before this question has its own
> spec. Tracked: TBD (file an issue when first item-view need surfaces).

## 6. Main window & navigation

| Widget | Status | Notes |
|---|---|---|
| `MainWindow` | 🟡 v2 | tracked: #234 — top-level shell containing menubar/toolbar/statusbar/dock; gates 7-9 below |
| `MenuBar` | 🟡 v2 | tracked: #235 — needs action system |
| `Menu` | 🟡 v2 | tracked: #236 — dropdown; needs action system |
| `ToolBar` | 🟡 v2 | tracked: #237 — needs action system + `ToolButton` |
| `StatusBar` | 🟡 v2 | tracked: #238 — bottom status messages |
| `DockWidget` | 📭 future | floatable panels; low priority |

## 7. Dialogs

| Widget | Status | Notes |
|---|---|---|
| `MessageBox` | 🟡 v2 | tracked: #239 — needs modal event loop |
| `FileDialog` | 🟡 v2 | tracked: #240 — platform-native preferred; needs OS layer |
| `InputDialog` | 🟡 v2 | tracked: #241 — simple value prompt |
| `ColorDialog` | 📭 future | low priority |
| `FontDialog` | 📭 future | low priority |
| `ProgressDialog` | 📭 future | depends on `ProgressBar` |

## 8. Layout primitives

| Layout | Status | Notes |
|---|---|---|
| `BoxLayout` (H/V) | ✅ first pass | horizontal + vertical stacking |
| `GridLayout` | ✅ first pass | rows × columns + cell spanning |
| `FormLayout` | 🟡 v2 | tracked: #242 — label-input pairs; common form shape |

## Tracking

When an item moves from this backlog to "in progress," file a dedicated issue
referencing the row above and link the issue back here in a follow-up edit.

## Topic-area follow-ups (paint, graphics, events, gpu)

Items routed here by `/triage` from `_inbox.md` whose source specs are
topic-adjacent to the widget set: `paint-style`, `widgets`, `graphics-stack`,
`geometry-events`, `event-types-crate`, `gpu-snapshot-tests-ci`. Schema mirrors
the thematic deferred files (`| Item | Source | Status | Tracked |`) so rows
remain candidates for `/triage` promotion to GitHub issues.

### Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Touch events (deferred) | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | untracked |
| Drag & drop events (deferred) | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | untracked |
| Wheel events (deferred) | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | untracked |
| Platform-specific input (handled by backend, not this crate) | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | untracked |
| ComboBox, SpinBox, Slider, ProgressBar (deferred to v2) | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Platform-native rendering (uses `quartzite-paint` Painter abstraction) | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Accessibility | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Text / font rendering (deferred to a later step) | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| Pluggable backend abstraction (single fixed stack for v1) | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| `quartzite-style` and `quartzite-widgets` implementation (separate plans) | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| Mobile as a primary target (Android / iOS supported by winit + wgpu; treated as a bonus) | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| WASM-specific packaging / bundling | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| Concrete event types that are not `TimerEvent` (`KeyEvent`, `MouseEvent`, `ResizeEvent`, `CloseEvent`) — these remain in `quartzite-events`. | [event-types-crate spec](../plans/done/2026-05-06-event-types-crate.spec.md) |  | untracked |
| `enumflags2`-backed `KeyModifiers` flags — stays in `quartzite-events`. | [event-types-crate spec](../plans/done/2026-05-06-event-types-crate.spec.md) |  | untracked |
| Widget or graphics-stack events. | [event-types-crate spec](../plans/done/2026-05-06-event-types-crate.spec.md) |  | untracked |
| A full event-dispatch loop using the unified types. | [event-types-crate spec](../plans/done/2026-05-06-event-types-crate.spec.md) |  | untracked |
| A working software / GPU rasteriser. `quartzite-renderer` already has a vello-backed `VelloPainter` skeleton; concrete rendering of new methods (`draw_text`, `draw_path`, `draw_image`) is deferred to its own plan once the API surface here is stable. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | untracked |
| SVG / PDF export. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | untracked |
| Bidirectional or shaped text layout. Basic LTR positioning only. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | untracked |
| Built-in image decoders (PNG/JPEG/etc.) — `Image` is a raw RGBA pixel buffer; loading from a file or compressed bytes is deferred. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | untracked |
| Per-widget `Style::draw_*` primitive methods (`draw_button`, `draw_label`, `draw_text_edit`, `draw_scroll_area`). The trait surface is `draw_widget` only; specialised dispatch happens inside concrete `Style` implementations. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | untracked |
| Sub-pixel font rendering. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | untracked |
| Implementing actual rendering inside `VelloPainter` (its `Painter` methods are currently no-ops). That is its own work item; this plan produces snapshots of *whatever the renderer emits today*, which for `Label` / `Button` / etc. is currently the clear colour. Goldens are regenerated by follow-up PRs as render code lands. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| Implementing `WidgetExt::paint` overrides on `Label`, `Button`, etc. Same reason as above. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| Conformance / fuzz testing of the renderer (vello upstream's job). | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| Performance / benchmark CI (separate issue). | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| A `quartzite-widgets`-side benchmarking pass against snapshots. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| Snapshotting / pixel-asserting the `xvfb-run` end-to-end frame (frame content is exercised by the offscreen suite; the windowed test asserts on clean startup + clean shutdown, not pixels). | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |

### Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `TouchEvent` — needs multi-touch design | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | #273 |
| `DragDropEvent` — needs clipboard/MIME design | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | #274 |
| `WheelEvent` — defer until scroll semantics decided | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | #275 |
| `MarginsF` — not needed until paint layer requires it | [geometry-events spec](../plans/done/2026-05-01-geometry-events.spec.md) |  | #276 |
| ComboBox, SpinBox, Slider, ProgressBar — not enough context to spec | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Menu / MenuBar — needs window/action system | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Dialog system — needs modal event loop | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Tooltip / ToolBar — low priority | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Text layout and font loading — deferred explicitly; vello uses `skrifa`/`parley` — will land with `quartzite-paint` implementation | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | #277 |
| Backend swap-out trait — deferred past v1 | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | #278 |
| Per-thread winit event loops — depends on winit multi-window / per-thread design | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | #279 |
| Porting more runtime concepts (e.g. `Application` event queue) to the unified event type system — out of scope for this PR | [event-types-crate spec](../plans/done/2026-05-06-event-types-crate.spec.md) |  | #280 |
| `BrushKind::LinearGradient` / `RadialGradient` variants — needs backend support to render | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | #281 |
| `Image::load_from_file` / `load_from_bytes` decoders — needs an I/O abstraction | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | #282 |
| Pixel-format metadata on `Image` (BGRA / premultiplied alpha / etc.) — only RGBA8 needed for v1 | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | #283 |
| `Style` per-platform overrides (e.g. macOS-flavoured vs. Windows-flavoured native style) — needs platform-detection plumbing | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | #284 |
| Re-evaluate `nv-flip` once renderer emits real pixels — Crate is alive but unmaintained (last release 0.1.2, June 2023). Acceptable for golden-of-clear-colour tests; revisit if real-pixel diffs reveal limitations. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | #285 |
| Per-test perceptual tolerance tuning — Default ΔE / FLIP threshold is a single number across the suite for v1; per-test thresholds added later if false positives accumulate. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | #286 |
| Larger canvases (256×256+) — Repo-size-driven for v1. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | #287 |
| Pixel-snapshotting the `xvfb-run` end-to-end frame — v1 asserts only clean startup + shutdown of the windowed pipeline; once the renderer emits real pixels we can revisit adding a snapshot to that test (or migrate it into the matrix). | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| Snapshot review tooling (web UI, image-diff viewer) — Workflow polish; the artifact-upload mechanism already lets reviewers see actual+diff PNGs. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | #288 |

### Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `WidgetExt::paint` call convention: should `paint()` be called directly from test code or only via a renderer-side dispatch? The spec leaves this open. For v1, `paint()` is a default no-op in `WidgetExt`; concrete widgets override it. No dispatch mechanism is added in this plan. | [widgets design](../plans/done/2026-05-01-widgets.design.md) |  | untracked |
| `Container` vs bare `WidgetBase + layout`: should `Container` be a distinct type with its own meta-object, or is it just a `WidgetBase` with a `layout` set? The spec says "generic layout container with no visual chrome." A distinct `Container` type is a cleaner API surface (its own class name shows up in reflection, separate `Meta`). Recommendation: keep `Container` as a distinct type; it is trivial to implement and avoids ambiguity. | [widgets design](../plans/done/2026-05-01-widgets.design.md) |  | untracked |
| Should `WidgetExt::paint` be called directly or only via a Painter-dispatch mechanism in the EventLoop? | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| How are event_filters (installed via ObjectId) resolved to concrete `EventFilter` trait objects at dispatch time? | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Should `Container` be a separate type or just `WidgetBase` with a layout attached? | [widgets spec](../plans/done/2026-05-01-widgets.spec.md) |  | untracked |
| Should `quartzite-renderer` re-export `winit::application::ApplicationHandler` for callers who have no direct `winit` dep? (Convenience re-export vs. letting callers control their own `winit` version.) Defer to implementation; document the choice in `quartzite-renderer/src/lib.rs`. | [graphics-stack design](../plans/done/2026-05-03-graphics-stack.design.md) |  | untracked |
| The CI `no_std` gate (`cargo build -p quartzite-paint-api --no-default-features`) must be added to `.github/workflows/ci.yml`. Confirm with the user whether this should be a new step in the existing `test` job or a separate job, and whether macOS/Windows runners should also run it (currently the `no_std` path check only runs on Linux). | [graphics-stack design](../plans/done/2026-05-03-graphics-stack.design.md) |  | untracked |
| Should the winit `EventLoop` integration be a trait (`EventDriver`?) in `quartzite-runtime` that both the headless loop and winit loop implement, or simply a separate entry point in `quartzite-renderer`? | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| Should `quartzite-renderer` expose a `WindowedApplication` wrapper over `Application`, or extend `Application` via an extension trait? | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| Does `quartzite-paint-api` need its own crate or can the types live in a `quartzite-paint::api` module with `pub use` re-exports? (Separate crate avoids the cycle risk; module approach avoids workspace churn.) | [graphics-stack spec](../plans/done/2026-05-03-graphics-stack.spec.md) |  | untracked |
| **Should `quartzite-geometry` add `quartzite-core` as a hard dep?** The spec lists only `quartzite-macros` for the `MetaEnum` derive. The macro's `crate_root()` helper (`quartzite-macros/src/util.rs:76`) resolves to `::quartzite_core` when neither the `quartzite` facade nor `quartzite-core` is found in the dependency graph — meaning the generated code refers to a crate that isn't there and the build fails. This design treats it as a mechanical consequence (yes, both deps required, with `quartzite-core` using `default-features = false` to preserve `no_std`). Asking for confirmation that the spec didn't intend either (a) a different macro family for geometry's `Alignment` or (b) inlining the `EnumMeta`/`IntoValue`/`FromValue` traits into geometry. Default is "add both deps" per the analysis above. | [paint-style design](../plans/done/2026-05-09-paint-style.design.md) |  | untracked |
| **Should `style()` (panicking accessor) ship in v1?** The spec says "may exist alongside; design phase decides." The design rejects it for YAGNI: `try_style().expect("style not installed")` at call sites is one extra line and keeps the panic surface explicit. Re-asking the product owner before close-out. | [paint-style design](../plans/done/2026-05-09-paint-style.design.md) |  | untracked |
| **Should `Path`/`Image`/`Font` impl `PartialEq`?** Spec is silent. `Path` and `Image` carry `Vec`s — derive is straightforward. `Font` derive is also fine (`String` + `f32` + bools). Default to `derive(PartialEq)` on all three for `assert_eq!` ergonomics; ask if there's a reason not to. | [paint-style design](../plans/done/2026-05-09-paint-style.design.md) |  | untracked |
| **Image error handling — overflow vs. length-mismatch.** The spec says `pixels.len() == (width * height * 4) as usize` is the validation. On 32-bit platforms, `width as usize * height as usize * 4` can overflow. The design adds an `Overflow` variant to `ImageError` to handle this case explicitly via `checked_mul`. Asking for confirmation; an alternative is to silently accept that very large dimensions aren't supported (panic-on-overflow in debug, wrap in release). Default is the explicit `Overflow` variant. | [paint-style design](../plans/done/2026-05-09-paint-style.design.md) |  | untracked |
| **Send + Sync bound on `Style`.** Required by the registry (a `&'static dyn Style` is shared across threads). The bound goes on every implementor. Confirming this is acceptable as an API constraint — current `quartzite-runtime` queues callbacks via `Send + 'static` so the precedent exists; default is "yes, add the bound." | [paint-style design](../plans/done/2026-05-09-paint-style.design.md) |  | untracked |
| **Light/dark `PaletteGroup`.** The spec § *quartzite-style-types* mentions "Light/dark variants are exposed via `PaletteGroup` rather than enum doubling." This design implements only `Palette` + `Default::default()` (light theme). `PaletteGroup` is an open question — its shape isn't pinned by any AC. Defer to a follow-up plan? The design treats `PaletteGroup` as out-of-scope for v1 and notes it explicitly so review doesn't surprise anyone. | [paint-style design](../plans/done/2026-05-09-paint-style.design.md) |  | untracked |
| Concrete `quartzite-renderer` implementations of the new `Painter` methods are deferred to a follow-up plan; this spec only nails down the trait surface and types they will be passed. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | #289 |
| Default-style content (e.g. a "Quartzite Default" struct shipped in `quartzite-style`): with the trait being generic-only, this becomes "design a single concrete `Style` struct whose `draw_widget` covers Button/Label/TextEdit/ScrollArea". Left to the design phase. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | #290 |
| `Image` source-rect cropping (drawing a sub-region of an `Image` into the destination `rect`) — not in v1; revisit when a backend gains real `draw_image` support. | [paint-style spec](../plans/done/2026-05-09-paint-style.spec.md) |  | #291 |
| **Behaviour when `WGPU_BACKEND` is set but the matching golden dir does not exist.** Resolved by subtask 6: the helper falls through to the standard "golden missing" panic, whose reviewer-friendly message names `scripts/update-snapshots.sh` and the resolved backend directory. CI treats this as a hard failure (a backend the suite hasn't been bootstrapped for); local dev is told how to bootstrap via the same message. No special-cased behaviour beyond the panic text. | [gpu-snapshot-tests-ci design](../plans/done/2026-05-10-gpu-snapshot-tests-ci.design.md) |  | untracked |
| Once `VelloPainter` emits real pixels, perceptual-diff tolerance will need real-world calibration (current default lands as a single number in design). False-positive rate after first real-render PR will tell us whether a per-test threshold is needed. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| `nv-flip` 0.1.2 was last published in June 2023. It still works, but its compute-shader path may lag wgpu major-version bumps. If it blocks a future wgpu upgrade, switch to `image-compare` 0.5; the design notes this contingency. Tracking decision belongs in a follow-up issue once the renderer emits real pixels. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| The `gpu-tests-pass` aggregator's `skipped`-as-success policy mirrors the existing `*-pass` aggregators in `ci.yml`; review during implementation in case the path-filter gate changes shape (e.g. if a future PR introduces a snapshot-only paths filter). | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |
| Whether the `xvfb-run` smoke test should later snapshot its frame (currently exit-code-only) is parked in *Deferred* — design phase may surface details about the production windowed-frame format that affect that future decision. | [gpu-snapshot-tests-ci spec](../plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) |  | untracked |

## Cross-references

- First-pass spec: [`ai-docs/plans/deferred/2026-05-01-widgets.spec.md`](../plans/deferred/2026-05-01-widgets.spec.md)
- Tracking issue: [#46](https://github.com/maratik123/quartzite/issues/46)
- Future-crates list: [`future-crates.md`](future-crates.md)
