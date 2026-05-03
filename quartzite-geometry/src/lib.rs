#![no_std]
#![deny(missing_docs)]
//! Geometry primitives for quartzite: integer and float coordinate types.
//!
//! All types are [`Copy`] and stack-allocated — no heap required.

mod margins;
mod point;
mod rect;
mod size;

pub use margins::Margins;
pub use point::{Point, PointF};
pub use rect::{Rect, RectF};
pub use size::{Size, SizeF};
