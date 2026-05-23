//! [`StyleClock`] — blink-phase read-seam for caret animation.
//!
//! The clock drives the on/off decision for caret visibility:
//! every half-period (default 530 ms) the visible phase flips.
//! The paint path reads [`StyleClock::caret_visible_now`] rather
//! than computing elapsed time itself, so tests can pin the phase
//! deterministically via [`StyleClock::pinned`].

use std::time::Instant;

/// Default half-period duration in milliseconds (530 ms per UX spec).
const DEFAULT_HALF_PERIOD_MS: u128 = 530;

/// Wall-clock read-seam for caret blink animation.
///
/// The blink phase is computed as `(elapsed_ms / half_period_ms) % 2 == 0`
/// (phase 0 = visible, phase 1 = invisible).  The period starts when the
/// clock is created with [`StyleClock::new`].
///
/// Tests can bypass the wall clock entirely by using [`StyleClock::pinned`],
/// which locks the visible phase to a known constant without reading
/// [`Instant::now`].
///
/// # Examples
///
/// ```
/// use quartzite_style::StyleClock;
///
/// // Default clock — phase depends on real time (non-deterministic in tests).
/// let clock = StyleClock::new();
/// let _ = clock.caret_visible_now(); // true or false depending on elapsed time
///
/// // Pinned clocks — always return the same phase; useful for snapshot tests.
/// assert!(StyleClock::pinned(true).caret_visible_now());
/// assert!(!StyleClock::pinned(false).caret_visible_now());
/// ```
#[derive(Debug, Clone)]
pub struct StyleClock {
    /// The wall-clock instant at which this clock was created (phase origin).
    start: Instant,
    /// Half-period in milliseconds; the caret phase flips every `half_period_ms` ms.
    half_period_ms: u128,
    /// When `Some`, bypasses the wall clock and returns the pinned phase directly.
    pinned: Option<bool>,
}

impl StyleClock {
    /// Constructs a new [`StyleClock`] anchored to [`Instant::now`].
    ///
    /// The caret starts in the *visible* phase (phase 0) and alternates
    /// every 530 ms thereafter.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::StyleClock;
    ///
    /// let clock = StyleClock::new();
    /// // Phase is deterministic relative to creation time; first window is visible.
    /// assert!(clock.caret_visible_now());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            half_period_ms: DEFAULT_HALF_PERIOD_MS,
            pinned: None,
        }
    }

    /// Constructs a [`StyleClock`] with a pinned phase, bypassing the wall clock.
    ///
    /// [`caret_visible_now`](Self::caret_visible_now) always returns `phase`
    /// regardless of elapsed time.  Use this in snapshot tests and paint-logic
    /// unit tests where deterministic output is required.
    ///
    /// # Parameters
    ///
    /// - `phase`: `true` → always visible; `false` → always invisible.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::StyleClock;
    ///
    /// assert!(StyleClock::pinned(true).caret_visible_now());
    /// assert!(!StyleClock::pinned(false).caret_visible_now());
    /// ```
    #[inline]
    pub fn pinned(phase: bool) -> Self {
        Self {
            start: Instant::now(),
            half_period_ms: DEFAULT_HALF_PERIOD_MS,
            pinned: Some(phase),
        }
    }

    /// Returns `true` when the caret should be visible at the current moment.
    ///
    /// If the clock was constructed with [`StyleClock::pinned`], the pinned
    /// phase is returned directly without reading the wall clock.
    ///
    /// Otherwise, visibility is `(elapsed_ms / half_period_ms) % 2 == 0`:
    /// phase 0 (elapsed ∈ `[0, half_period)`) → visible;
    /// phase 1 (elapsed ∈ `[half_period, 2×half_period)`) → invisible; and so on.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::StyleClock;
    ///
    /// // Pinned clocks are deterministic.
    /// assert!(StyleClock::pinned(true).caret_visible_now());
    /// assert!(!StyleClock::pinned(false).caret_visible_now());
    /// ```
    #[inline]
    pub fn caret_visible_now(&self) -> bool {
        if let Some(pinned) = self.pinned {
            return pinned;
        }
        let elapsed_ms = self.start.elapsed().as_millis();
        (elapsed_ms / self.half_period_ms).is_multiple_of(2)
    }

    /// Returns `true` when the platform reports reduced-motion preference.
    ///
    /// Currently always returns `false` — host-side plumbing for the
    /// accessibility preference is out of scope for this crate.  A future
    /// version will query the platform via the runtime layer.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_style::StyleClock;
    ///
    /// assert!(!StyleClock::new().prefers_reduced_motion());
    /// assert!(!StyleClock::pinned(true).prefers_reduced_motion());
    /// ```
    #[inline]
    pub const fn prefers_reduced_motion(&self) -> bool {
        false
    }
}

impl Default for StyleClock {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn pinned_true_returns_true_unconditionally() {
        let clock = StyleClock::pinned(true);
        assert!(clock.caret_visible_now(), "pinned(true) must return true");
        // Check multiple times to confirm no state mutation.
        assert!(clock.caret_visible_now());
        assert!(clock.caret_visible_now());
    }

    #[test]
    fn pinned_false_returns_false_unconditionally() {
        let clock = StyleClock::pinned(false);
        assert!(
            !clock.caret_visible_now(),
            "pinned(false) must return false"
        );
        assert!(!clock.caret_visible_now());
        assert!(!clock.caret_visible_now());
    }

    #[test]
    fn default_clock_alternates_phase_around_530ms() {
        // Mid-on window: 260 ms elapsed → phase 0 → visible.
        let mid_on = StyleClock {
            start: Instant::now()
                .checked_sub(Duration::from_millis(260))
                .unwrap(),
            half_period_ms: DEFAULT_HALF_PERIOD_MS,
            pinned: None,
        };
        assert!(
            mid_on.caret_visible_now(),
            "at 260 ms (mid first half-period) the caret must be visible"
        );

        // Mid-off window: 795 ms elapsed → phase 1 → invisible.
        let mid_off = StyleClock {
            start: Instant::now()
                .checked_sub(Duration::from_millis(795))
                .unwrap(),
            half_period_ms: DEFAULT_HALF_PERIOD_MS,
            pinned: None,
        };
        assert!(
            !mid_off.caret_visible_now(),
            "at 795 ms (mid second half-period) the caret must be invisible"
        );
    }

    #[test]
    fn prefers_reduced_motion_default_false() {
        assert!(!StyleClock::new().prefers_reduced_motion());
        assert!(!StyleClock::pinned(true).prefers_reduced_motion());
        assert!(!StyleClock::pinned(false).prefers_reduced_motion());
    }
}
