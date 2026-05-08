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
| `ProgressBar` | 🟡 v2 | depends on numeric range model |
| `TextBrowser` | 📭 future | rich-text + hyperlink navigation; needs rich-text engine |
| `LCDNumber` | ❌ dropped | retro-style 7-segment display; no compelling use case |

## 2. Buttons

| Widget | Status | Notes |
|---|---|---|
| `Button` (push) | ✅ first pass | text + checkable + signals |
| `CheckBox` | 🟡 v2 | trivial extension of `Button` once checkable groups work |
| `RadioButton` | 🟡 v2 | needs button group abstraction |
| `ToolButton` | 🟡 v2 | depends on `ToolBar` shell |
| `CommandLinkButton` | 📭 future | platform-specific styling; low priority |

## 3. Input

| Widget | Status | Notes |
|---|---|---|
| `LineEdit` | ✅ first pass | single-line text input |
| `TextEdit` | ✅ first pass | multi-line; rich text deferred to text engine v2 |
| `PlainTextEdit` | 🟡 v2 | optimised for large logs; specialisation of `TextEdit` |
| `SpinBox` | 🟡 v2 | integer numeric input |
| `DoubleSpinBox` | 🟡 v2 | float numeric input |
| `ComboBox` | 🟡 v2 | dropdown selection |
| `Slider` | 🟡 v2 | range value via drag |
| `Dial` | ❌ dropped | speedometer-style; rare in modern UIs |
| `DateEdit` / `TimeEdit` / `DateTimeEdit` | 📭 future | needs date/time model + calendar popup |
| `KeySequenceEdit` | 📭 future | shortcut capture widget; low priority |

## 4. Containers

| Widget | Status | Notes |
|---|---|---|
| `Container` | ✅ first pass | generic layout container with no chrome |
| `ScrollArea` | ✅ first pass | scrollable view |
| `GroupBox` | 🟡 v2 | titled frame around a layout |
| `TabWidget` | 🟡 v2 | tabbed pages |
| `StackedWidget` | 🟡 v2 | one-of-many visible; programmatic control |
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
| `MainWindow` | 🟡 v2 | top-level shell containing menubar/toolbar/statusbar/dock; gates 7-9 below |
| `MenuBar` | 🟡 v2 | needs action system |
| `Menu` | 🟡 v2 | dropdown; needs action system |
| `ToolBar` | 🟡 v2 | needs action system + `ToolButton` |
| `StatusBar` | 🟡 v2 | bottom status messages |
| `DockWidget` | 📭 future | floatable panels; low priority |

## 7. Dialogs

| Widget | Status | Notes |
|---|---|---|
| `MessageBox` | 🟡 v2 | needs modal event loop |
| `FileDialog` | 🟡 v2 | platform-native preferred; needs OS layer |
| `InputDialog` | 🟡 v2 | simple value prompt |
| `ColorDialog` | 📭 future | low priority |
| `FontDialog` | 📭 future | low priority |
| `ProgressDialog` | 📭 future | depends on `ProgressBar` |

## 8. Layout primitives

| Layout | Status | Notes |
|---|---|---|
| `BoxLayout` (H/V) | ✅ first pass | horizontal + vertical stacking |
| `GridLayout` | ✅ first pass | rows × columns + cell spanning |
| `FormLayout` | 🟡 v2 | label-input pairs; common form shape |

## Tracking

When an item moves from this backlog to "in progress," file a dedicated issue
referencing the row above and link the issue back here in a follow-up edit.

## Cross-references

- First-pass spec: [`ai-docs/plans/deferred/2026-05-01-widgets.spec.md`](../plans/deferred/2026-05-01-widgets.spec.md)
- Tracking issue: [#46](https://github.com/maratik123/quartzite/issues/46)
- Future-crates list: [`future-crates.md`](future-crates.md)
