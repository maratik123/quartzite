#![no_std]
#![deny(missing_docs)]
//! Geometry primitives for quartzite: integer and float coordinate types.
//!
//! All types are [`Copy`] and stack-allocated — no heap required.

#[cfg(feature = "std")]
extern crate std;

mod margins;
mod point;
mod rect;
mod size;

pub use margins::Margins;
pub use point::{Point, PointF};
pub use rect::{Rect, RectF};
pub use size::{Size, SizeF};

/// Rounds `x` to the nearest integer, half away from zero.
#[inline]
pub(crate) fn round_f32(x: f32) -> i32 {
    #[cfg(feature = "std")]
    {
        x.round() as i32
    }
    #[cfg(not(feature = "std"))]
    {
        libm::roundf(x) as i32
    }
}
