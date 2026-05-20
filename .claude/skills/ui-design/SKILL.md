---
name: ui-design
description: "Load Quartzite design-system context for visual / paint work — Style impl + DefaultStyle changes, Palette / ColorRole edits, paint paths in quartzite-style / quartzite-widgets / quartzite-paint-api, snapshot tests. Pointer-only: reads design-system/ on demand. Distinct from design-system/SKILL.md (name: quartzite-design), which is not slash-discoverable."
disable-model-invocation: false
allowed-tools: Read
---

Load the Quartzite design-system context for the current visual task. This skill is pointer-only — no rules are inlined here; the source of truth lives under `design-system/` at the repo root.

Read in order:

1. `design-system/SKILL.md` — manifest + Quick reference (Palette source, font, asset paths).
2. `design-system/README.md` — the visual foundations section plus the full derivation rules.
3. `design-system/preview/` and `design-system/ui_kits/widgets/` — explore as needed for per-widget visual contracts and reference renderings.

Use this skill when:

- When working on `quartzite-style` (any `Style` impl, including `DefaultStyle`)
- When working on `quartzite-widgets` paint paths, widget views, or any user-facing rendering
- When changing `Palette` / `ColorRole` semantics or seeds
- When adding or modifying snapshot tests under `quartzite-style/tests/snapshots/`
- When working on quartzite-paint-api painter primitives, brush, pen, path, font, or color
