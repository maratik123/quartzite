//! GPU snapshot tests for [`DefaultStyle`].
//!
//! Each test drives [`DefaultStyle::draw_widget`] directly inside a
//! [`RenderHarness`] closure and compares the rendered image against a
//! committed golden PNG via the local [`support::snapshot_assert`].
//!
//! Tests deliberately do **not** call `WidgetExt::paint` — the harness
//! closure is the bridge between `&mut dyn Painter` and the harness's
//! owned painter, exercising `DefaultStyle`'s routing and drawing code
//! rather than the widget's own (no-op) paint method (AC13).

mod support;

use quartzite_geometry::{Point, Rect, Size};
use quartzite_style::{DefaultStyle, Style};
use quartzite_style_types::Palette;
use quartzite_widgets::{AsWidget, Button, Label, ScrollArea, TextEdit, WidgetExt};

use support::{harness_or_skip, snapshot_assert};

/// Canvas size shared by all snapshot tests. 64 keeps committed PNGs small.
const CANVAS: u32 = 64;

fn canvas_rect() -> Rect {
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
        DefaultStyle::default().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::default().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::default().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::default().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::default().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
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
        DefaultStyle::default().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("text_edit_read_only", &image);
}

#[test]
fn scroll_area_chrome_renders() {
    let Some(mut harness) = harness_or_skip("scroll_area_chrome_renders") else {
        return;
    };
    let mut w = ScrollArea::new();
    w.set_geometry(canvas_rect());
    let image = harness.render_widget(|painter| {
        DefaultStyle::default().draw_widget(&w as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("scroll_area_chrome", &image);
}
