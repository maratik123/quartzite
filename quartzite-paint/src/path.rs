/// A vector graphics path (stub; full implementation in the paint-style plan #47).
///
/// # Examples
///
/// ```
/// use quartzite_paint::Path;
///
/// let _path = Path::new();
/// ```
#[derive(Clone, Debug, Default)]
pub struct Path;

impl Path {
    /// Creates an empty path.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint::Path;
    ///
    /// let path = Path::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_constructible() {
        let _p = Path::new();
    }
}
