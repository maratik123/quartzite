//! [`GridLayout`] — rows × columns grid layout.

use quartzite_core::{ObjectBase, ObjectId};
use quartzite_geometry::{Point, Rect, Size};
use quartzite_macros::{Extend, Object, object_impl};

use crate::layout::{Layout, WidgetResolver};

/// Placement descriptor for a single widget in a [`GridLayout`].
///
/// # Examples
///
/// ```
/// use quartzite_core::ObjectId;
/// use quartzite_widgets::GridCell;
///
/// let cell = GridCell::new(ObjectId::new(), 0, 1);
/// assert_eq!(cell.row, 0);
/// assert_eq!(cell.col, 1);
/// ```
#[derive(Clone, Debug)]
pub struct GridCell {
    /// Widget placed at this grid position.
    pub widget: ObjectId,
    /// Zero-based row index.
    pub row: u32,
    /// Zero-based column index.
    pub col: u32,
    /// Number of rows this cell spans (minimum 1).
    pub row_span: u32,
    /// Number of columns this cell spans (minimum 1).
    pub col_span: u32,
}

impl GridCell {
    /// Creates a 1×1 cell at `(row, col)`.
    ///
    /// # Parameters
    ///
    /// - `widget`: id of the widget placed at this position.
    /// - `row`: zero-based row index.
    /// - `col`: zero-based column index.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::GridCell;
    ///
    /// let cell = GridCell::new(ObjectId::new(), 1, 2);
    /// assert_eq!(cell.row_span, 1);
    /// assert_eq!(cell.col_span, 1);
    /// ```
    #[inline]
    pub fn new(widget: ObjectId, row: u32, col: u32) -> Self {
        Self {
            widget,
            row,
            col,
            row_span: 1,
            col_span: 1,
        }
    }

    /// Creates a cell that spans `row_span` rows and `col_span` columns.
    ///
    /// # Parameters
    ///
    /// - `widget`: id of the widget placed at this position.
    /// - `row`: zero-based row index of the top-left corner.
    /// - `col`: zero-based column index of the top-left corner.
    /// - `row_span`: number of rows to occupy (minimum 1).
    /// - `col_span`: number of columns to occupy (minimum 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::GridCell;
    ///
    /// let cell = GridCell::spanning(ObjectId::new(), 0, 0, 1, 2);
    /// assert_eq!(cell.col_span, 2);
    /// ```
    #[inline]
    pub fn spanning(widget: ObjectId, row: u32, col: u32, row_span: u32, col_span: u32) -> Self {
        Self {
            widget,
            row,
            col,
            row_span: row_span.max(1),
            col_span: col_span.max(1),
        }
    }
}

/// Lays out child widgets in a rows × columns grid.
///
/// Cells are added via [`GridLayout::add_cell`]. When [`Layout::set_geometry`] is
/// called, `set_geometry` divides the available rect into equal-sized columns and rows,
/// then assigns each cell its sub-rect (expanded by span).
///
/// # Examples
///
/// ```
/// use quartzite_widgets::GridLayout;
///
/// let layout = GridLayout::new();
/// assert_eq!(layout.cell_count(), 0);
/// ```
#[derive(Extend, Object)]
#[root]
pub struct GridLayout {
    #[base]
    object: ObjectBase,
    cells: Vec<GridCell>,
}

impl GridLayout {
    /// Creates a new empty [`GridLayout`].
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::GridLayout;
    ///
    /// let layout = GridLayout::new();
    /// assert_eq!(layout.cell_count(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            object: ObjectBase::new(),
            cells: Vec::new(),
        }
    }

    /// Adds a 1×1 cell at `(row, col)` for `widget`.
    ///
    /// # Parameters
    ///
    /// - `widget`: id of the widget to place.
    /// - `row`: zero-based row index.
    /// - `col`: zero-based column index.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::GridLayout;
    ///
    /// let mut layout = GridLayout::new();
    /// layout.add_cell(ObjectId::new(), 0, 0);
    /// assert_eq!(layout.cell_count(), 1);
    /// ```
    pub fn add_cell(&mut self, widget: ObjectId, row: u32, col: u32) {
        self.cells.push(GridCell::new(widget, row, col));
    }

    /// Adds a spanning cell for `widget`.
    ///
    /// # Parameters
    ///
    /// - `widget`: id of the widget to place.
    /// - `row`: zero-based row index.
    /// - `col`: zero-based column index.
    /// - `row_span`: number of rows to span.
    /// - `col_span`: number of columns to span.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_core::ObjectId;
    /// use quartzite_widgets::GridLayout;
    ///
    /// let mut layout = GridLayout::new();
    /// layout.add_cell_spanning(ObjectId::new(), 0, 0, 2, 3);
    /// assert_eq!(layout.cell_count(), 1);
    /// ```
    pub fn add_cell_spanning(
        &mut self,
        widget: ObjectId,
        row: u32,
        col: u32,
        row_span: u32,
        col_span: u32,
    ) {
        self.cells
            .push(GridCell::spanning(widget, row, col, row_span, col_span));
    }

    /// Returns the number of cells in this layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::GridLayout;
    ///
    /// let layout = GridLayout::new();
    /// assert_eq!(layout.cell_count(), 0);
    /// ```
    #[inline]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    fn grid_dimensions(&self) -> (u32, u32) {
        let rows = self
            .cells
            .iter()
            .map(|c| c.row + c.row_span)
            .max()
            .unwrap_or(1);
        let cols = self
            .cells
            .iter()
            .map(|c| c.col + c.col_span)
            .max()
            .unwrap_or(1);
        (rows, cols)
    }
}

impl Default for GridLayout {
    /// Returns a new empty `GridLayout`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartzite_widgets::GridLayout;
    ///
    /// let layout = GridLayout::default();
    /// assert_eq!(layout.cell_count(), 0);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for GridLayout {
    #[allow(
        clippy::cast_possible_wrap,
        reason = "deliberate i32/usize casts within grid layout bounds"
    )]
    fn set_geometry(&mut self, resolver: &mut dyn WidgetResolver, rect: Rect) {
        if self.cells.is_empty() {
            return;
        }
        let (n_rows, n_cols) = self.grid_dimensions();
        let cell_w = rect.size().width() / n_cols as i32;
        let cell_h = rect.size().height() / n_rows as i32;

        let cells: Vec<GridCell> = self.cells.clone();
        for cell in &cells {
            let x = rect.left() + cell.col as i32 * cell_w;
            let y = rect.top() + cell.row as i32 * cell_h;
            let w = cell_w * cell.col_span as i32;
            let h = cell_h * cell.row_span as i32;
            let child_rect = Rect::new(Point::new(x, y), Size::new(w, h));
            if let Some(wb) = resolver.resolve_widget_mut(cell.widget) {
                wb.geometry = child_rect;
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> Size {
        Size::default()
    }

    #[inline]
    fn minimum_size(&self) -> Size {
        Size::default()
    }
}

#[object_impl]
impl GridLayout {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::WidgetBase;

    struct StubResolver(HashMap<ObjectId, WidgetBase>);

    impl WidgetResolver for StubResolver {
        fn resolve_widget_mut(&mut self, id: ObjectId) -> Option<&mut WidgetBase> {
            self.0.get_mut(&id)
        }
    }

    #[test]
    fn widget_at_col1_row0_placed_correctly() {
        let id = ObjectId::new();
        let mut layout = GridLayout::new();
        layout.add_cell(id, 0, 1);

        let mut map = HashMap::new();
        map.insert(id, WidgetBase::new());
        let mut resolver = StubResolver(map);

        // cell at col=1 → grid is 2 columns wide; each column is 150px wide, 200px tall
        layout.set_geometry(
            &mut resolver,
            Rect::new(Point::new(0, 0), Size::new(300, 200)),
        );
        let g = resolver.0[&id].geometry;
        assert_eq!(g.left(), 150, "col=1 should start at x=150");
        assert_eq!(g.top(), 0);
        assert_eq!(g.size().width(), 150);
        assert_eq!(g.size().height(), 200);
    }

    #[test]
    fn spanning_cell_gets_double_width() {
        let id = ObjectId::new();
        let mut layout = GridLayout::new();
        layout.add_cell_spanning(id, 0, 0, 1, 2);

        let mut map = HashMap::new();
        map.insert(id, WidgetBase::new());
        let mut resolver = StubResolver(map);

        // 2 columns → cell_w = 100
        layout.set_geometry(
            &mut resolver,
            Rect::new(Point::new(0, 0), Size::new(200, 100)),
        );
        let g = resolver.0[&id].geometry;
        assert_eq!(g.size().width(), 200, "col_span=2 should give double width");
    }
}
