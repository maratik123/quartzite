#![no_std]
//! Leaf crate for the quartzite styling system.
//!
//! Holds the bottom-of-the-stack styling vocabulary — [`Palette`] and
//! [`ColorRole`] — so it can be imported by `quartzite-widgets` without
//! pulling in the downstream `quartzite-style` crate. This split is the
//! cycle-break that lets `quartzite-style` depend on `quartzite-widgets`
//! while widgets still re-exports the palette types from a stable upstream
//! location.
//!
//! The crate is `no_std + alloc` and depends only on `quartzite-paint-api`
//! (for [`quartzite_paint_api::Color`]).

extern crate alloc;

mod color_role;
mod palette;

pub use color_role::ColorRole;
pub use palette::Palette;
