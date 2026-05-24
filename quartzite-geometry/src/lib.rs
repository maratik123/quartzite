#![no_std]
//! Geometry primitives for quartzite: integer and float coordinate types.
//!
//! All types are [`Copy`] and stack-allocated — no heap required.

#[cfg(feature = "std")]
extern crate std;

mod alignment;
mod margins;
mod point;
mod rect;
mod size;
mod v_alignment;

pub use alignment::HAlignment;
pub use margins::Margins;
pub use point::{Point, PointF};
pub use rect::{Rect, RectF};
pub use size::{Size, SizeF};
pub use v_alignment::VAlignment;

/// Rounds `x` to the nearest integer, half away from zero.
#[inline]
#[allow(
    clippy::cast_possible_truncation,
    reason = "deliberate truncation within known bounds"
)]
#[cfg(feature = "std")]
pub(crate) const fn round_f32(x: f32) -> i32 {
    x.round() as i32
}

/// Rounds `x` to the nearest integer, half away from zero.
#[inline]
#[allow(
    clippy::cast_possible_truncation,
    reason = "deliberate truncation within known bounds"
)]
#[cfg(not(feature = "std"))]
pub(crate) fn round_f32(x: f32) -> i32 {
    libm::roundf(x) as i32
}
