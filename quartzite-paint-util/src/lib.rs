#![no_std]
//! RAII painting utilities for quartzite.
//!
//! This crate is `no_std`-compatible and provides thin wrappers around
//! [`quartzite_paint_api::Painter`] primitives. It depends only on
//! `quartzite-paint-api` and `quartzite-geometry`.

extern crate alloc;
