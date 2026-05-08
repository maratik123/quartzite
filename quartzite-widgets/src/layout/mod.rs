//! Layout system: [`WidgetResolver`], [`Layout`] trait, [`BoxLayout`], and [`GridLayout`].

mod box_layout;
mod grid_layout;

pub use box_layout::{BoxLayout, Direction};
pub use grid_layout::{GridCell, GridLayout};

use quartzite_core::ObjectId;
use quartzite_geometry::{Rect, Size};

use crate::widget_base::WidgetBase;

/// Resolves [`ObjectId`] values to mutable [`WidgetBase`] references for geometry updates.
///
/// Implemented by the renderer's `ObjectTree` wrapper during layout passes
/// (plan #47). For unit tests, a `HashMap<ObjectId, WidgetBase>`-backed stub
/// is used instead.
pub trait WidgetResolver {
    /// Returns a mutable reference to the [`WidgetBase`] identified by `id`, or `None`
    /// if the id is not present in this resolver.
    ///
    /// # Parameters
    ///
    /// - `id`: the unique identifier of the widget to look up.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::{WidgetBase, WidgetResolver};
    ///
    /// # struct MyResolver;
    /// # impl WidgetResolver for MyResolver {
    /// #     fn resolve_widget_mut(&mut self, id: ObjectId) -> Option<&mut WidgetBase> { None }
    /// # }
    /// let mut resolver = MyResolver;
    /// let id = ObjectId::new();
    /// assert!(resolver.resolve_widget_mut(id).is_none());
    /// ```
    fn resolve_widget_mut(&mut self, id: ObjectId) -> Option<&mut WidgetBase>;
}

/// Distributes geometry among child widgets.
///
/// Both concrete layout types ([`BoxLayout`], [`GridLayout`]) implement this trait.
/// The renderer calls [`Layout::set_geometry`] with a [`WidgetResolver`] that maps
/// child [`ObjectId`]s to `&mut dyn AsWidget`.
///
/// `add_widget` / `remove_widget` are intentionally absent from this trait because
/// layout mutation happens at construction time via the concrete types' own methods;
/// the trait covers only geometry calculation.
pub trait Layout {
    /// Distributes `rect` among this layout's children via `resolver`.
    ///
    /// # Parameters
    ///
    /// - `resolver`: maps child `ObjectId`s to mutable widget references.
    /// - `rect`: the total rectangle to distribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use quartzite_core::ObjectId;
    /// use quartzite_geometry::{Point, Rect, Size};
    /// use quartzite_widgets::{BoxLayout, Layout, WidgetBase, WidgetResolver};
    ///
    /// struct StubResolver(HashMap<ObjectId, WidgetBase>);
    /// impl WidgetResolver for StubResolver {
    ///     fn resolve_widget_mut(&mut self, id: ObjectId) -> Option<&mut WidgetBase> {
    ///         self.0.get_mut(&id)
    ///     }
    /// }
    ///
    /// let mut layout = BoxLayout::new(quartzite_widgets::Direction::Horizontal);
    /// let mut resolver = StubResolver(HashMap::new());
    /// layout.set_geometry(&mut resolver, Rect::new(Point::new(0, 0), Size::new(100, 100)));
    /// ```
    fn set_geometry(&mut self, resolver: &mut dyn WidgetResolver, rect: Rect);

    /// Returns the preferred size for this layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{GridLayout, Layout};
    /// use quartzite_geometry::Size;
    ///
    /// let layout = GridLayout::new();
    /// assert_eq!(layout.size_hint(), Size::default());
    /// ```
    fn size_hint(&self) -> Size;

    /// Returns the minimum size this layout can be rendered at.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::{GridLayout, Layout};
    /// use quartzite_geometry::Size;
    ///
    /// let layout = GridLayout::new();
    /// assert_eq!(layout.minimum_size(), Size::default());
    /// ```
    fn minimum_size(&self) -> Size;
}
