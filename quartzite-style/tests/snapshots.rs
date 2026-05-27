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

use std::collections::HashMap;

use quartzite_core::ObjectId;
use quartzite_geometry::{Point, Rect, Size};
use quartzite_paint_api::Painter;
use quartzite_style::{DefaultStyle, Style, StyleClock, StyleRegistry};
use quartzite_style_dispatch::{WidgetResolver, dispatch_paint};
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_hovered_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_hovered(true);
    render_dark("dark_button_hovered", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_pressed_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_pressed(true);
    render_dark("dark_button_pressed", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_checked_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.checked = true;
    render_dark("dark_button_checked", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_focused_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    render_dark("dark_button_focused", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_button_disabled_renders() {
    let mut w = Button::new("OK".into());
    w.set_geometry(canvas_rect());
    w.set_enabled(false);
    render_dark("dark_button_disabled", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_label_renders() {
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    render_dark("dark_label", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_scroll_area_chrome_renders() {
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    render_dark("dark_scroll_area_chrome", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_text_edit_plain_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    render_dark("dark_text_edit_plain", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_text_edit_read_only_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.read_only = true;
    render_dark("dark_text_edit_read_only", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("scroll_area_focused", &image);
}

#[test]
fn dark_label_focused_renders() {
    let mut w = Label::new("hi".into());
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    render_dark("dark_label_focused", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_text_edit_focused_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "abc".into();
    w.set_focused(true);
    render_dark("dark_text_edit_focused", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_scroll_area_focused_renders() {
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    render_dark("dark_scroll_area_focused", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("line_edit_placeholder", &image);
}

#[test]
fn dark_line_edit_idle_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    render_dark("dark_line_edit_idle", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

#[test]
fn dark_line_edit_focused_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_focused(true);
    render_dark("dark_line_edit_focused", |painter| {
        DefaultStyle::new().draw_widget(&w as &dyn AsWidget, painter, &DARK_PALETTE);
    });
}

// ---------------------------------------------------------------------------
// TextEdit caret + selection snapshot goldens (issue #317)
// ---------------------------------------------------------------------------
//
// All four light tests use `DefaultStyle::with_clock(StyleClock::pinned(true))`
// so the caret is always visible and the golden is deterministic.

#[test]
fn text_edit_focused_caret_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_focused_caret_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Hello world".into();
    w.caret = 5;
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("text_edit_focused_caret", &image);
}

#[test]
fn text_edit_selection_wrap_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_selection_wrap_renders") else {
        return;
    };
    // Text long enough to wrap on a 64px canvas (≥ 9 chars).
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Hello world".into();
    w.caret = 9;
    w.selection_anchor = Some(0);
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("text_edit_selection_wrap", &image);
}

#[test]
fn text_edit_read_only_selection_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_read_only_selection_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Read only".into();
    w.read_only = true;
    w.caret = 4;
    w.selection_anchor = Some(0);
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("text_edit_read_only_selection", &image);
}

#[test]
fn text_edit_unfocused_selection_renders() {
    let Some(mut harness) = harness_or_skip("text_edit_unfocused_selection_renders") else {
        return;
    };
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Hello".into();
    w.caret = 5;
    w.selection_anchor = Some(0);
    // Not focused — selection alpha is half.
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("text_edit_unfocused_selection", &image);
}

#[test]
fn dark_text_edit_focused_caret_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Hello world".into();
    w.caret = 5;
    w.set_focused(true);
    render_dark("dark_text_edit_focused_caret", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

#[test]
fn dark_text_edit_selection_wrap_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Hello world".into();
    w.caret = 9;
    w.selection_anchor = Some(0);
    w.set_focused(true);
    render_dark("dark_text_edit_selection_wrap", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

#[test]
fn dark_text_edit_read_only_selection_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Read only".into();
    w.read_only = true;
    w.caret = 4;
    w.selection_anchor = Some(0);
    render_dark("dark_text_edit_read_only_selection", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

#[test]
fn dark_text_edit_unfocused_selection_renders() {
    let mut w = TextEdit::new();
    w.set_geometry(canvas_rect());
    w.plain_text = "Hello".into();
    w.caret = 5;
    w.selection_anchor = Some(0);
    // Not focused — selection alpha is half.
    render_dark("dark_text_edit_unfocused_selection", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

// ---------------------------------------------------------------------------
// LineEdit caret + selection snapshot goldens (issue #405)
// ---------------------------------------------------------------------------
//
// All light tests use `DefaultStyle::with_clock(StyleClock::pinned(true))`
// so the caret is always visible and the golden is deterministic.

#[test]
fn line_edit_focused_empty_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_focused_empty_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("line_edit_focused_empty", &image);
}

#[test]
fn line_edit_focused_caret_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_focused_caret_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_caret(2);
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("line_edit_focused_caret", &image);
}

#[test]
fn line_edit_focused_selection_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_focused_selection_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_caret(1);
    w.set_selection_anchor(Some(3));
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("line_edit_focused_selection", &image);
}

#[test]
fn line_edit_unfocused_selection_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_unfocused_selection_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_caret(1);
    w.set_selection_anchor(Some(3));
    // Not focused — selection alpha is half.
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("line_edit_unfocused_selection", &image);
}

#[test]
fn line_edit_read_only_selection_renders() {
    let Some(mut harness) = harness_or_skip("line_edit_read_only_selection_renders") else {
        return;
    };
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.caret = 1;
    w.selection_anchor = Some(3);
    w.read_only = true;
    w.set_focused(true);
    let image = harness.render_widget(|painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &Palette::default(),
        );
    });
    snapshot_assert("line_edit_read_only_selection", &image);
}

#[test]
fn dark_line_edit_focused_empty_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.set_focused(true);
    render_dark("dark_line_edit_focused_empty", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

#[test]
fn dark_line_edit_focused_caret_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_caret(2);
    w.set_focused(true);
    render_dark("dark_line_edit_focused_caret", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

#[test]
fn dark_line_edit_focused_selection_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_caret(1);
    w.set_selection_anchor(Some(3));
    w.set_focused(true);
    render_dark("dark_line_edit_focused_selection", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

#[test]
fn dark_line_edit_unfocused_selection_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.set_caret(1);
    w.set_selection_anchor(Some(3));
    // Not focused — selection alpha is half.
    render_dark("dark_line_edit_unfocused_selection", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

#[test]
fn dark_line_edit_read_only_selection_renders() {
    let mut w = LineEdit::new();
    w.set_geometry(canvas_rect());
    w.text = "abc".into();
    w.caret = 1;
    w.selection_anchor = Some(3);
    w.read_only = true;
    w.set_focused(true);
    render_dark("dark_line_edit_read_only_selection", |painter| {
        DefaultStyle::with_clock(StyleClock::pinned(true)).draw_widget(
            &w as &dyn AsWidget,
            painter,
            &DARK_PALETTE,
        );
    });
}

// ---------------------------------------------------------------------------
// AC12 end-to-end clip snapshot — dispatch_paint path
// ---------------------------------------------------------------------------

/// A minimal [`WidgetResolver`] backed by a [`HashMap`], used by the AC12
/// clip snapshot test to supply `dispatch_paint` with a small widget tree.
struct MapResolver(HashMap<ObjectId, Box<dyn AsWidget>>);

impl MapResolver {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn insert<W: AsWidget + 'static>(&mut self, id: ObjectId, widget: W) {
        self.0.insert(id, Box::new(widget));
    }
}

impl WidgetResolver for MapResolver {
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
        self.0.get(&id).map(|b| b.as_ref() as &dyn AsWidget)
    }
}

/// Snapshot test (AC12): a `ScrollArea` whose `content_widget` has geometry
/// extending beyond the `ScrollArea`'s own bounds renders with the content
/// visually clipped at `content_rect()`.
///
/// Routes through `dispatch_paint` (the full Vello end-to-end path), not
/// `DefaultStyle::draw_widget` directly, so the clip layer inserted by
/// `TranslateGuard::with_clip` is exercised.
#[test]
fn scroll_area_clips_oversized_content_renders() {
    let _lock = quartzite_test_helpers::test_lock();
    let Some(mut harness) = harness_or_skip("scroll_area_clips_oversized_content_renders") else {
        return;
    };
    StyleRegistry::set_style(Box::new(DefaultStyle::new()));

    let area_id = ObjectId::new();
    let label_id = ObjectId::new();

    let mut area = ScrollArea::new();
    area.set_geometry(canvas_rect());
    area.show();
    area.content_widget = Some(label_id);

    // Label geometry is 200×200 — extends well beyond the 64×64 canvas so any
    // ink that escapes the clip layer would be visible in the golden.
    let mut label = Label::new("content".into());
    label.set_geometry(Rect::new(Point::new(0, 0), Size::new(200, 200)));
    label.show();

    let mut resolver = MapResolver::new();
    resolver.insert(area_id, area);
    resolver.insert(label_id, label);

    let image = harness.render_widget(|painter| {
        dispatch_paint(area_id, &resolver, painter, &Palette::default());
    });
    snapshot_assert("scroll_area_clips_oversized_content", &image);
}
