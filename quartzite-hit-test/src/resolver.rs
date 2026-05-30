//! The read-only [`WidgetResolver`] trait shared by hit-testing and paint dispatch.

use quartzite_core::ObjectId;
use quartzite_widgets::AsWidget;

/// Maps an [`ObjectId`] to the widget it refers to.
///
/// Implemented by callers to bridge the gap between the identifier-based widget
/// tree (where `Container::children()` returns `&[ObjectId]`) and the
/// paint-time need for `&dyn AsWidget` references.
///
/// This trait is deliberately separate from
/// [`quartzite_widgets::layout::WidgetResolver`], which resolves
/// `ObjectId` → `&mut WidgetBase` for mutable layout-time operations; this one
/// resolves to an immutable `&dyn AsWidget` for read-only paint-time access.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use quartzite_core::ObjectId;
/// use quartzite_widgets::{AsWidget, WidgetBase};
/// use quartzite_hit_test::WidgetResolver;
///
/// struct MapResolver(HashMap<ObjectId, Box<dyn AsWidget>>);
///
/// impl WidgetResolver for MapResolver {
///     fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
///         self.0.get(&id).map(|b| b.as_ref() as &dyn AsWidget)
///     }
/// }
/// ```
pub trait WidgetResolver {
    /// Returns the widget identified by `id`, or `None` if it is not present in
    /// this resolver's backing store.
    ///
    /// Implementations should be cheap (e.g. a hash-map lookup); callers may
    /// call `resolve` more than once per child during a single traversal.
    ///
    /// # Parameters
    ///
    /// - `id`: the unique identifier of the widget to look up.
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget>;
}

/// Blanket implementation so a closure `|id| …` can act as a [`WidgetResolver`].
///
/// # Lifetime caveat
///
/// This impl is parsed as `for<'a> Fn(ObjectId) -> Option<&'a dyn AsWidget>`,
/// which means the closure must return a reference valid for any lifetime — in
/// practice only `&'static` references satisfy this (e.g. leaked boxes or
/// `static` widget arrays). Callers that need to return borrows tied to `self`
/// should implement [`WidgetResolver`] directly on their tree-wrapper type.
impl<F> WidgetResolver for F
where
    F: Fn(ObjectId) -> Option<&'static dyn AsWidget>,
{
    // _Simple._
    fn resolve(&self, id: ObjectId) -> Option<&dyn AsWidget> {
        self(id)
    }
}
