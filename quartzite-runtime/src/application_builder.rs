//! Builder for the [`Application`] singleton.
use std::time::Duration;

use crate::application::{Application, ApplicationError};
use crate::event_loop::EventLoop;

/// Builder for the [`Application`] singleton.
///
/// Obtain a builder via [`Application::builder()`] and call [`build`](Self::build) to
/// install the singleton. By default the inner [`EventLoop`] is **tickless** — it blocks
/// on [`Receiver::recv`](std::sync::mpsc::Receiver::recv) until a closure arrives or
/// [`Application::quit`] is called. Use [`tick_duration`](Self::tick_duration) to
/// request a tick-based loop instead.
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::Application;
///
/// // For the tickless default use Application::new() directly:
/// let app = Application::new().expect("only one Application per process");
/// app.quit();
///
/// // Use the builder when you need options, e.g. a tick-based loop:
/// # let _ = (|| -> Result<_, _> {
/// use std::time::Duration;
/// let app = Application::builder()
///     .tick_duration(Some(Duration::from_millis(50)))
///     .build()?;
/// # Ok::<_, quartzite_runtime::ApplicationError>(app)
/// # })();
/// ```
#[derive(Debug)]
#[must_use]
pub struct ApplicationBuilder {
    /// Tick duration for the inner [`EventLoop`].
    ///
    /// `None` (the default) produces a tickless loop; `Some(d)` produces a tick-based
    /// loop that wakes at most every `d` even when no closure is posted.
    tick: Option<Duration>,
}

impl ApplicationBuilder {
    /// Creates a new [`ApplicationBuilder`] with tickless default.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::application_builder::ApplicationBuilder;
    ///
    /// let builder = ApplicationBuilder::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self { tick: None }
    }

    /// Sets the tick duration for the inner [`EventLoop`].
    ///
    /// - `None` → tickless (default): the loop blocks on `recv()` until a closure arrives.
    /// - `Some(d)` → tick-based: the loop wakes at most every `d` via `recv_timeout(d)`.
    ///
    /// Passing `Some(Duration::ZERO)` is silently normalised to `None` (tickless) because
    /// a zero-duration timeout would busy-loop without doing useful work.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::builder()
    ///     .tick_duration(Some(Duration::from_millis(50)))
    ///     .build()
    ///     .expect("only one Application per process");
    /// ```
    pub const fn tick_duration(mut self, tick: Option<Duration>) -> Self {
        self.tick = match tick {
            Some(d) if d.is_zero() => None,
            other => other,
        };
        self
    }

    /// Builds the [`Application`] singleton, installing the event-loop, queued dispatcher,
    /// and object factory.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError::AlreadyExists`] if an [`Application`] has already been
    /// installed in this process. Only one [`Application`] may exist per process.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::Application;
    ///
    /// let app = Application::builder().build().expect("only one Application per process");
    /// ```
    pub fn build(self) -> Result<Application, ApplicationError> {
        Application::build_from(EventLoop::with_tick(self.tick))
    }
}

impl Default for ApplicationBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_equals_new() {
        let via_default = ApplicationBuilder::default();
        let via_new = ApplicationBuilder::new();
        assert_eq!(via_default.tick, via_new.tick);
    }
}
