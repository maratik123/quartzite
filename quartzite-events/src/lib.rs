#![no_std]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]
//! Event model for quartzite: mouse, keyboard, window, and timer events.
//!
//! All event types implement [`Event<T>`](event::Event), which is object-safe for a fixed `T`.
//! Use `T = ()` (the default) when your application has no custom user events.

extern crate alloc;

mod event;
mod keyboard;
mod mouse;
mod timer;
mod window;

pub use event::{Event, EventFilter, EventType, KeyEventKind, MouseEventKind};
pub use keyboard::{Key, KeyEvent, KeyModifier, KeyModifiers};
pub use mouse::{MouseButton, MouseButtons, MouseEvent};
pub use timer::TimerEvent;
pub use window::{CloseEvent, ResizeEvent};

pub use quartzite_core::ObjectId;
