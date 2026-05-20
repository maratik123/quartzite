# fonts/

**This directory is intentionally empty.**

The Quartzite framework asks the painter for `"sans-serif"`
(`Font::default()` in `quartzite-paint-api/src/font.rs`) and lets
the renderer resolve it at draw time. The `maratik123/quartzite`
repository ships **no font files** — `.ttf` / `.otf` distribution
is out of scope.

For HTML mockups in this design system, the CSS stack in
`colors_and_type.css` is rooted on **DejaVu Sans** /
**Liberation Sans** — the typical Linux text-shaper fallback that
most closely matches the rendered glyphs in the committed
snapshot PNGs at `quartzite-style/tests/snapshots/shared/` in the
upstream repo.

## To improve fidelity

If you want pixel-identical mocks, drop the actual render target
font into this directory and add a matching `@font-face` rule:

```css
@font-face {
  font-family: 'Quartzite UI';
  src: url('./fonts/your-font.woff2') format('woff2');
  font-weight: 100 900;
  font-style: normal;
}
```

Then either edit `--qz-font-family` in `colors_and_type.css` to
list `'Quartzite UI'` first, or open a follow-up question with
the user about which family the project is standardizing on.

## ⚠️ Substitution flag

The default `Font::default()` family is the abstract string
`"sans-serif"`. Any specific typeface choice is a downstream
decision the framework intentionally leaves open. If you make
one in a mock, flag it.
