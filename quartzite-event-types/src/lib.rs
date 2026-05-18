#![no_std]
//! Core event vocabulary for quartzite: the [`Event`] trait, [`EventType`] discriminant,
//! [`EventFilter`] interceptor trait, and [`TimerEvent`].
//!
//! This crate is `no_std` + alloc and sits between `quartzite-core` and `quartzite-events`
//! in the dependency graph, allowing `quartzite-runtime` to use typed timer event payloads
//! without depending on the heavier `quartzite-events` crate.

extern crate alloc;

mod event;
mod timer;

pub use event::{Event, EventFilter, EventType, KeyEventKind, MouseEventKind};
pub use timer::TimerEvent;

pub use quartzite_core::ObjectId;
