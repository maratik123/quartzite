use crate::Color;
use quartzite_geometry::Point;

/// Describes how a shape's interior is filled.
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{Brush, BrushKind, Color};
/// use quartzite_geometry::Point;
///
/// let brush = Brush::solid(Color::RED);
/// assert_eq!(brush.kind(), &BrushKind::Solid(Color::RED));
///
/// let linear = Brush::linear_gradient(
///     Point::new(0, 0),
///     Point::new(10, 0),
///     Color::RED,
///     Color::BLUE,
/// );
/// assert!(matches!(linear.kind(), BrushKind::LinearGradient { .. }));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Brush {
    kind: BrushKind,
}

/// The fill pattern variant for a [`Brush`].
///
/// # Examples
///
/// ```
/// use quartzite_paint_api::{BrushKind, Color};
///
/// let kind = BrushKind::Solid(Color::BLUE);
/// // BrushKind is Clone
/// let _k2 = kind.clone();
/// let _k3 = kind;
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum BrushKind {
    /// A solid fill with a single color.
    Solid(Color),
    /// A two-stop linear gradient between two points.
    ///
    /// Coordinates are in widget-local pixels, matching other [`Painter`](crate::Painter) inputs.
    LinearGradient {
        /// Start point of the gradient axis.
        start: Point,
        /// End point of the gradient axis.
        end: Point,
        /// Color at `start` (offset 0.0).
        start_color: Color,
        /// Color at `end` (offset 1.0).
        end_color: Color,
    },
    /// A two-stop radial gradient radiating outward from a centre point.
    ///
    /// Coordinates are in widget-local pixels.
    RadialGradient {
        /// Centre of the gradient circle.
        centre: Point,
        /// Radius of the gradient circle in pixels.
        radius: f32,
        /// Color at the centre (offset 0.0).
        start_color: Color,
        /// Color at the outer edge (offset 1.0).
        end_color: Color,
    },
    /// A rich gradient using a [`peniko::Gradient`] directly.
    ///
    /// Use this escape hatch for gradients with more than two stops, non-default
    /// extend modes ([`peniko::Extend::Reflect`] / [`peniko::Extend::Repeat`]),
    /// affine transforms, or two-circle radial gradients.
    ///
    /// **Coordinate space:** the caller is responsible for supplying coordinates
    /// in the renderer's physical-pixel space (i.e. already scaled by the
    /// device-pixel ratio). No automatic scaling is applied by `VelloPainter`.
    Custom(peniko::Gradient),
}

impl Brush {
    /// Creates a solid-color brush.
    ///
    /// # Parameters
    ///
    /// - `color`: the fill color.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Brush, BrushKind, Color};
    ///
    /// let brush = Brush::solid(Color::GREEN);
    /// assert_eq!(brush.kind(), &BrushKind::Solid(Color::GREEN));
    /// ```
    #[inline]
    pub const fn solid(color: Color) -> Self {
        Self {
            kind: BrushKind::Solid(color),
        }
    }

    /// Creates a two-stop linear-gradient brush between `start` and `end`.
    ///
    /// The gradient interpolates from `start_color` at `start` to `end_color` at `end`,
    /// using `Pad` extend mode. For more complex gradients use [`Brush::custom_gradient`].
    ///
    /// # Parameters
    ///
    /// - `start`: start point in widget-local pixels.
    /// - `end`: end point in widget-local pixels.
    /// - `start_color`: color at `start` (offset 0.0).
    /// - `end_color`: color at `end` (offset 1.0).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Brush, BrushKind, Color};
    /// use quartzite_geometry::Point;
    ///
    /// const BRUSH: Brush = Brush::linear_gradient(
    ///     Point::new(0, 0),
    ///     Point::new(10, 0),
    ///     Color::RED,
    ///     Color::BLUE,
    /// );
    /// assert!(matches!(BRUSH.kind(), BrushKind::LinearGradient { .. }));
    /// ```
    #[inline]
    pub const fn linear_gradient(
        start: Point,
        end: Point,
        start_color: Color,
        end_color: Color,
    ) -> Self {
        Self {
            kind: BrushKind::LinearGradient {
                start,
                end,
                start_color,
                end_color,
            },
        }
    }

    /// Creates a two-stop radial-gradient brush centred at `centre` with the given `radius`.
    ///
    /// The gradient interpolates from `start_color` at the centre to `end_color` at the
    /// outer edge (radius), using `Pad` extend mode. For more complex gradients use
    /// [`Brush::custom_gradient`].
    ///
    /// # Parameters
    ///
    /// - `centre`: centre point in widget-local pixels.
    /// - `radius`: radius of the gradient circle in pixels.
    /// - `start_color`: color at the centre (offset 0.0).
    /// - `end_color`: color at the outer edge (offset 1.0).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Brush, BrushKind, Color};
    /// use quartzite_geometry::Point;
    ///
    /// const BRUSH: Brush = Brush::radial_gradient(
    ///     Point::new(5, 5),
    ///     10.0,
    ///     Color::WHITE,
    ///     Color::BLACK,
    /// );
    /// assert!(matches!(BRUSH.kind(), BrushKind::RadialGradient { .. }));
    /// ```
    #[inline]
    pub const fn radial_gradient(
        centre: Point,
        radius: f32,
        start_color: Color,
        end_color: Color,
    ) -> Self {
        Self {
            kind: BrushKind::RadialGradient {
                centre,
                radius,
                start_color,
                end_color,
            },
        }
    }

    /// Creates a brush backed by a [`peniko::Gradient`] for complex gradient cases.
    ///
    /// Use this for gradients with more than two stops, non-default extend modes,
    /// affine transforms, or two-circle radial shapes.
    ///
    /// **Coordinate space:** supply coordinates already scaled by the device-pixel
    /// ratio — no automatic scaling is applied by `VelloPainter`.
    ///
    /// # Parameters
    ///
    /// - `gradient`: the peniko gradient to use.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Brush, BrushKind};
    /// use peniko::Gradient;
    ///
    /// let g = Gradient::new_linear((0.0, 0.0), (10.0, 0.0));
    /// let brush = Brush::custom_gradient(g.clone());
    /// assert!(matches!(brush.kind(), BrushKind::Custom(_)));
    /// ```
    #[inline]
    pub fn custom_gradient(gradient: peniko::Gradient) -> Self {
        Self {
            kind: BrushKind::Custom(gradient),
        }
    }

    /// Returns a reference to the fill kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_paint_api::{Brush, BrushKind, Color};
    ///
    /// assert_eq!(Brush::solid(Color::WHITE).kind(), &BrushKind::Solid(Color::WHITE));
    /// ```
    #[inline]
    pub fn kind(&self) -> &BrushKind {
        &self.kind
    }
}

impl Default for Brush {
    /// Returns a solid opaque white brush.
    #[inline]
    fn default() -> Self {
        Self::solid(Color::WHITE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use peniko::Gradient;

    #[test]
    fn solid_stores_color() {
        let brush = Brush::solid(Color::RED);
        assert_eq!(brush.kind(), &BrushKind::Solid(Color::RED));
    }

    #[test]
    fn default_is_solid_white() {
        assert_eq!(Brush::default().kind(), &BrushKind::Solid(Color::WHITE));
    }

    #[test]
    fn linear_gradient_stores_fields() {
        let start = Point::new(0, 0);
        let end = Point::new(10, 0);
        let brush = Brush::linear_gradient(start, end, Color::RED, Color::BLUE);
        assert!(
            matches!(
                brush.kind(),
                BrushKind::LinearGradient {
                    start_color,
                    end_color,
                    ..
                } if *start_color == Color::RED && *end_color == Color::BLUE
            ),
            "expected LinearGradient with RED/BLUE"
        );
    }

    #[test]
    fn radial_gradient_stores_fields() {
        let centre = Point::new(5, 5);
        let brush = Brush::radial_gradient(centre, 3.0, Color::WHITE, Color::BLACK);
        assert!(
            matches!(
                brush.kind(),
                BrushKind::RadialGradient {
                    radius,
                    start_color,
                    end_color,
                    ..
                } if (*radius - 3.0).abs() < f32::EPSILON
                    && *start_color == Color::WHITE
                    && *end_color == Color::BLACK
            ),
            "expected RadialGradient with radius 3.0, WHITE/BLACK"
        );
    }

    #[test]
    fn linear_gradient_is_const_fn() {
        const BRUSH: Brush =
            Brush::linear_gradient(Point::new(0, 0), Point::new(10, 0), Color::RED, Color::BLUE);
        assert!(matches!(BRUSH.kind(), BrushKind::LinearGradient { .. }));
    }

    #[test]
    fn radial_gradient_is_const_fn() {
        const BRUSH: Brush =
            Brush::radial_gradient(Point::new(5, 5), 10.0, Color::WHITE, Color::BLACK);
        assert!(matches!(BRUSH.kind(), BrushKind::RadialGradient { .. }));
    }

    #[test]
    fn custom_gradient_round_trips() {
        let g = Gradient::new_linear((0.0f64, 0.0f64), (10.0f64, 0.0f64));
        let brush = Brush::custom_gradient(g.clone());
        assert!(
            matches!(brush.kind(), BrushKind::Custom(got) if got == &g),
            "custom gradient did not round-trip"
        );
    }
}
