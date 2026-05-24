//! Widget-tree paint dispatcher for quartzite.
//!
//! This crate bridges [`quartzite_style`] and [`quartzite_widgets`] without
//! coupling either to the renderer. The sole entry point is [`dispatch_paint`],
//! a free function that walks a widget subtree and invokes
//! [`Style::draw_widget`][quartzite_style::Style::draw_widget] once per visible
//! node.
//!
//! # Quick start
//!
//! ```
//! use std::collections::HashMap;
//! use quartzite_core::ObjectId;
//! use quartzite_widgets::{AsWidget, Button, Container, Label, WidgetExt};
//! use quartzite_style::{DefaultStyle, StyleRegistry};
//! use quartzite_style::Palette;
//! use quartzite_style_dispatch::{WidgetResolver, dispatch_paint};
//!
//! // 1. Install a style (once per process, before painting).
//! StyleRegistry::set_style(Box::new(DefaultStyle::new()));
//!
//! // 2. Build a widget tree.
//! let root_id  = ObjectId::new();
//! let label_id = ObjectId::new();
//!
//! let mut root = Container::new();
//! root.show();
//! root.add_child(label_id);
//!
//! let mut label = Label::new("Hello".into());
//! label.show();
//!
//! // 3. Implement WidgetResolver over your backing store.
//! struct MapResolver(HashMap<ObjectId, Box<dyn AsWidget>>);
//!
//! impl WidgetResolver for MapResolver {
//!     fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
//!         self.0.get(&id).map(|b| b.as_ref() as &dyn AsWidget)
//!     }
//! }
//!
//! let mut map = HashMap::new();
//! map.insert(root_id,  Box::new(root)  as Box<dyn AsWidget>);
//! map.insert(label_id, Box::new(label) as Box<dyn AsWidget>);
//! let resolver = MapResolver(map);
//!
//! // 4. Call dispatch_paint with the root id, resolver, painter, and palette.
//! struct NullCaret;
//! impl quartzite_paint_api::TextCaretCursor for NullCaret {
//!     fn advance_to(&mut self, _: usize) {}
//!     fn caret_x(&self) -> i32 { 0 }
//!     fn line_top(&self) -> i32 { 0 }
//!     fn line_height(&self) -> i32 { 0 }
//! }
//! struct NullLines;
//! impl quartzite_paint_api::TextVisualLineCursor for NullLines {
//!     fn next_line(&mut self) -> Option<quartzite_paint_api::TextVisualLine> { None }
//! }
//! struct NullPainter { caret: NullCaret, lines: NullLines }
//! impl quartzite_paint_api::Painter for NullPainter {
//!     fn draw_rect(&mut self, _r: quartzite_geometry::Rect, _p: &quartzite_paint_api::Pen, _b: &quartzite_paint_api::Brush) {}
//!     fn fill_rect(&mut self, _r: quartzite_geometry::Rect, _b: &quartzite_paint_api::Brush) {}
//!     fn draw_line(&mut self, _a: quartzite_geometry::Point, _b: quartzite_geometry::Point, _p: &quartzite_paint_api::Pen) {}
//!     fn clip_rect(&mut self, _r: quartzite_geometry::Rect) {}
//!     fn translate(&mut self, _d: quartzite_geometry::Point) {}
//!     fn save(&mut self) {}
//!     fn restore(&mut self) {}
//!     fn draw_text(&mut self, _p: quartzite_geometry::Point, _t: &str, _f: &quartzite_paint_api::Font, _b: &quartzite_paint_api::Brush) {}
//!     fn draw_text_in(&mut self, _r: quartzite_geometry::Rect, _t: &str, _f: &quartzite_paint_api::Font, _b: &quartzite_paint_api::Brush, _h: quartzite_geometry::Alignment, _v: quartzite_geometry::Alignment) {}
//!     fn draw_image(&mut self, _r: quartzite_geometry::Rect, _i: &quartzite_paint_api::Image) {}
//!     fn draw_path(&mut self, _p: &quartzite_paint_api::Path, _pe: &quartzite_paint_api::Pen, _b: &quartzite_paint_api::Brush) {}
//!     fn text_carets(&mut self, _t: &str, _f: &quartzite_paint_api::Font) -> &mut dyn quartzite_paint_api::TextCaretCursor { &mut self.caret }
//!     fn text_visual_lines(&mut self, _t: &str, _f: &quartzite_paint_api::Font, _w: i32) -> &mut dyn quartzite_paint_api::TextVisualLineCursor { &mut self.lines }
//! }
//!
//! let mut painter = NullPainter { caret: NullCaret, lines: NullLines };
//! dispatch_paint(root_id, &resolver, &mut painter, &Palette::default());
//! ```

mod dispatch;

pub use dispatch::{WidgetResolver, dispatch_paint};
