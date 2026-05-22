//! GPU snapshot tests for [`DefaultStyle`].
//!
//! Each test drives [`DefaultStyle::draw_widget`] directly inside a
//! [`RenderHarness`] closure and compares the rendered image against a
//! committed golden PNG via the local [`support::snapshot_assert`].
//!
//! Tests drive [`DefaultStyle::draw_widget`] through the harness closure —
//! the bridge between `&mut dyn Painter` and the harness's owned painter —
//! exercising `DefaultStyle`'s routing and drawing code directly (AC13).

// Skipped under Miri at the file level: GPU init. The whole-file shape is
// required (not per-test `cfg_attr`) because every test routes through
// `support::harness_or_skip` → `RenderHarnessBuilder::build()` →
// `wgpu::Instance::default()`, and the `quartzite-renderer` dev-dep imports
// reach this module regardless of which individual `#[test]` is enabled.
// Alternative coverage: native `cargo test` exercises this file on the
// `gpu-tests` job. See ai-docs/miri-policy.md § Per-file fallback recipe.
#![cfg(not(miri))]

mod support;

use quartzite_geometry::{Point, Rect, Size};
use quartzite_paint_api::Painter;
use quartzite_style::{DefaultStyle, Style};
use quartzite_style_types::{DARK_PALETTE, Palette};
use quartzite_widgets::{AsWidget, Button, Label, LineEdit, ScrollArea, TextEdit, WidgetExt};

use support::{harness_or_skip, snapshot_assert};

/// Canvas size shared by all snapshot tests. 64 keeps committed PNGs small.
const CANVAS: u32 = 64;

#[allow(
    clippy::cast_possible_wrap,
    reason = "snapshot test index arithmetic within known u16 range"
)]
const fn canvas_rect() -> Rect {
    Rect::new(Point::new(0, 0), Size::new(CANVAS as i32, CANVAS as i32))
}

#[test]
fn button_idle_renders() {
    let Some(mut harness) = harness_or_skip("button_idle_renders") else {
        return;
    };
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("button_idle", &image);
}

#[test]
fn button_checked_renders() {
    let Some(mut harness) = harness_or_skip("button_checked_renders") else {
        return;
    };
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.checked = true;
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("button_checked", &image);
}

#[test]
fn button_disabled_renders() {
    let Some(mut harness) = harness_or_skip("button_disabled_renders") else {
        return;
    };
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_enabled(false);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("button_disabled", &image);
}

#[test]
fn label_renders() {
    let Some(mut harness) = harness_or_skip("label_renders") else {
        return;
    };
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("label", &image);
}

#[test]
fn text_edit_plain_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_plain_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("text_edit_plain", &image);
}

#[test]
fn text_edit_read_only_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_read_only_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.read_only = true;
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("text_edit_read_only", &image);
}

#[test]
fn button_hovered_renders() {
    let Some(mut harness) = harness_or_skip("button_hovered_renders") else {
        return;
    };
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_hovered(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("button_hovered", &image);
}

#[test]
fn button_pressed_renders() {
    let Some(mut harness) = harness_or_skip("button_pressed_renders") else {
        return;
    };
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_pressed(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("button_pressed", &image);
}

#[test]
fn button_focused_renders() {
    let Some(mut harness) = harness_or_skip("button_focused_renders") else {
        return;
    };
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("button_focused", &image);
}

#[test]
fn scroll_area_chrome_renders() {
    let Some(mut harness) = harness_or_skip("scroll_area_chrome_renders") else {
        return;
    };
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("scroll_area_chrome", &image);
}

// ---------------------------------------------------------------------------
// Dark-theme snapshot tests
// ---------------------------------------------------------------------------

/// Runs a dark-palette snapshot test: boots a harness, renders via `build`,
/// and asserts the result against the golden named `name` in
/// `tests/snapshots/shared/dark_<name>.png`.
///
/// Lives here (not in `support/mod.rs`) so the snapshot-helper sync group
/// (`quartzite-style/tests/support/mod.rs` ↔ `quartzite-widgets/tests/support/mod.rs`)
/// stays untouched — the widgets side has no dark-theme use case.
fn render_dark<F: FnOnce(&mut dyn Painter)>(name: &str, build: F) {
    let Some(mut harness) = harness_or_skip(name) else {
        return;
    };
    let image = harness.render_widget(build);
    snapshot_assert(name, &image);
}

#[test]
fn dark_button_idle_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    render_dark("dark_button_idle", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_hovered_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_hovered(true);
    render_dark("dark_button_hovered", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_pressed_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_pressed(true);
    render_dark("dark_button_pressed", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_checked_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.checked = true;
    render_dark("dark_button_checked", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_focused_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    render_dark("dark_button_focused", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_disabled_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_enabled(false);
    render_dark("dark_button_disabled", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_label_renders() {
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    render_dark("dark_label", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_scroll_area_chrome_renders() {
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    render_dark("dark_scroll_area_chrome", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_text_edit_plain_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    render_dark("dark_text_edit_plain", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_text_edit_read_only_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.read_only = true;
    render_dark("dark_text_edit_read_only", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

// ---------------------------------------------------------------------------
// Label / TextEdit / ScrollArea hover / pressed / focused (issue #403 — AC6)
// ---------------------------------------------------------------------------

#[test]
fn label_hovered_renders() {
    let Some(mut harness) = harness_or_skip("label_hovered_renders") else {
        return;
    };
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    w.set_hovered(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("label_hovered", &image);
}

#[test]
fn label_pressed_renders() {
    let Some(mut harness) = harness_or_skip("label_pressed_renders") else {
        return;
    };
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    w.set_pressed(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("label_pressed", &image);
}

#[test]
fn label_focused_renders() {
    let Some(mut harness) = harness_or_skip("label_focused_renders") else {
        return;
    };
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("label_focused", &image);
}

#[test]
fn text_edit_hovered_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_hovered_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.set_hovered(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("text_edit_hovered", &image);
}

#[test]
fn text_edit_pressed_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_pressed_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.set_pressed(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("text_edit_pressed", &image);
}

#[test]
fn text_edit_focused_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_focused_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("text_edit_focused", &image);
}

#[test]
fn scroll_area_hovered_renders() {
    let Some(mut harness) = harness_or_skip("scroll_area_hovered_renders") else {
        return;
    };
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    w.set_hovered(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("scroll_area_hovered", &image);
}

#[test]
fn scroll_area_pressed_renders() {
    let Some(mut harness) = harness_or_skip("scroll_area_pressed_renders") else {
        return;
    };
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    w.set_pressed(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("scroll_area_pressed", &image);
}

#[test]
fn scroll_area_focused_renders() {
    let Some(mut harness) = harness_or_skip("scroll_area_focused_renders") else {
        return;
    };
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("scroll_area_focused", &image);
}

#[test]
fn dark_label_focused_renders() {
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    render_dark("dark_label_focused", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_text_edit_focused_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.set_focused(true);
    render_dark("dark_text_edit_focused", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_scroll_area_focused_renders() {
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    render_dark("dark_scroll_area_focused", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

// ---------------------------------------------------------------------------
// LineEdit single-flag goldens (issue #406; folds in #407)
// ---------------------------------------------------------------------------
//
// `line_edit_disabled.png` is the visible #407 fold-in anchor — half-alpha
// Base fill + half-alpha Text outline + half-alpha Text glyphs, distinct
// from `line_edit_idle.png` (full-alpha equivalent).

#[test]
fn line_edit_idle_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_idle_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_idle", &image);
}

#[test]
fn line_edit_hovered_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_hovered_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_hovered(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_hovered", &image);
}

#[test]
fn line_edit_pressed_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_pressed_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_pressed(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_pressed", &image);
}

#[test]
fn line_edit_focused_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_focused_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_focused", &image);
}

#[test]
fn line_edit_disabled_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_disabled_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_enabled(false);
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_disabled", &image);
}

#[test]
fn line_edit_read_only_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_read_only_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.read_only = true;
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_read_only", &image);
}

#[test]
fn line_edit_placeholder_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_placeholder_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.placeholder = "hint".into();
    let image = harness.render_widget(|painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_placeholder", &image);
}

#[test]
fn dark_line_edit_idle_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    render_dark("dark_line_edit_idle", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_line_edit_focused_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_focused(true);
    render_dark("dark_line_edit_focused", |painter| {
        DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}
