//! Widget system for quartzite: base widget hierarchy, layout system, and built-in widgets.
//!
//! # Overview
//!
//! The core types are [`WidgetBase`] (the hierarchy root) and [`WidgetExt`] (the blanket
//! extension trait). All concrete widget types derive [`quartzite_macros::Extend`] with
//! `#[base] widget_base: WidgetBase`, inheriting the full [`AsWidget`] and [`quartzite_core::AsObject`] chains.
//!
//! Layout is handled by [`Layout`] — a resolver-parameterised trait implemented by
//! [`BoxLayout`] and [`GridLayout`]. Geometry distribution requires a [`WidgetResolver`]
//! that maps [`quartzite_core::ObjectId`] to `&mut dyn AsWidget`.
//!
//! # Features
//!
//! - [`widgets`] — all built-in concrete widgets ([`Label`], [`Button`], [`LineEdit`], …)
//! - [`layout`] — layout types ([`BoxLayout`], [`GridLayout`])

pub mod enums;
pub mod layout;
pub mod widget_base;
pub mod widget_ext;
pub mod widgets;

pub use enums::{CursorShape, FocusPolicy, SizePolicy};
pub use layout::{BoxLayout, Direction, GridCell, GridLayout, Layout, WidgetResolver};
pub use quartzite_geometry::{HAlignment, VAlignment};
pub use quartzite_paint::{Font, FontWeight};
pub use quartzite_style_types::{ColorRole, Palette};
pub use widget_base::{
    AsWidget, WidgetBase, WidgetChildren, WidgetChildrenIter, WidgetState, WidgetStates, WidgetView,
};
pub use widget_ext::WidgetExt;
pub use widgets::{Button, Container, Label, LineEdit, ScrollArea, ScrollPolicy, TextEdit};
