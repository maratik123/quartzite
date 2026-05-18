---
name: design
description: "Explicitly load the Quartzite design-system context for visual work — paint paths, Style impls, Palette / ColorRole changes, snapshot tests. Pointer-only; Reads the canonical design-system/ folder on demand. Distinct from design-system/SKILL.md (name: quartzite-design), which is not slash-discoverable."
disable-model-invocation: true
allowed-tools: Read
---

Load the Quartzite design-system context for the current visual task. This skill is pointer-only — no rules are inlined here; the source of truth lives under `design-system/` at the repo root.

Read in order:

1. `design-system/SKILL.md` — manifest + Quick reference (Palette source, font, asset paths).
2. `design-system/README.md` — the visual foundations section plus the full derivation rules.
3. `design-system/preview/` and `design-system/ui_kits/widgets/` — explore as needed for per-widget visual contracts and reference renderings.

Use this skill when working on `quartzite-widgets` paint paths or widget views, any `Style` impl in `quartzite-style` (including `DefaultStyle`), changes to `Palette` / `ColorRole` semantics or seeds, or snapshot tests under `quartzite-style/tests/snapshots/`.
