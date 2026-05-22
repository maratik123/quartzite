#![no_std]
//! RAII painting utilities for quartzite.
//!
//! This crate is `no_std`-compatible and provides thin wrappers around
//! [`quartzite_paint_api::Painter`] primitives. It depends only on
//! `quartzite-paint-api` and `quartzite-geometry`.

extern crate alloc;

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use quartzite_geometry::Point;
    use quartzite_paint_api::Painter;

    use crate::TranslateGuard;

    // Local minimal RecordingPainter stub — re-using the one in
    // quartzite-paint-api via a dev-dependency cycle is not viable.
    #[derive(Debug, PartialEq)]
    enum PaintEvent {
        Save,
        Restore,
        Translate(Point),
        FillRect,
        Other,
    }

    struct RecordingPainter {
        events: Vec<PaintEvent>,
    }

    impl RecordingPainter {
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl Painter for RecordingPainter {
        fn draw_rect(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _pen: &quartzite_paint_api::Pen,
            _brush: &quartzite_paint_api::Brush,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn fill_rect(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _brush: &quartzite_paint_api::Brush,
        ) {
            self.events.push(PaintEvent::FillRect);
        }

        fn draw_line(&mut self, _from: Point, _to: Point, _pen: &quartzite_paint_api::Pen) {
            self.events.push(PaintEvent::Other);
        }

        fn clip_rect(&mut self, _rect: quartzite_geometry::Rect) {
            self.events.push(PaintEvent::Other);
        }

        fn translate(&mut self, delta: Point) {
            self.events.push(PaintEvent::Translate(delta));
        }

        fn save(&mut self) {
            self.events.push(PaintEvent::Save);
        }

        fn restore(&mut self) {
            self.events.push(PaintEvent::Restore);
        }

        fn draw_text(
            &mut self,
            _pos: Point,
            _text: &str,
            _font: &quartzite_paint_api::Font,
            _brush: &quartzite_paint_api::Brush,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn draw_text_in(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _text: &str,
            _font: &quartzite_paint_api::Font,
            _brush: &quartzite_paint_api::Brush,
            _alignment: quartzite_geometry::Alignment,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn draw_image(
            &mut self,
            _rect: quartzite_geometry::Rect,
            _image: &quartzite_paint_api::Image,
        ) {
            self.events.push(PaintEvent::Other);
        }

        fn draw_path(
            &mut self,
            _path: &quartzite_paint_api::Path,
            _pen: &quartzite_paint_api::Pen,
            _brush: &quartzite_paint_api::Brush,
        ) {
            self.events.push(PaintEvent::Other);
        }
    }

    #[test]
    fn constructor_records_save_then_translate() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(3, 4);
        let guard = TranslateGuard::new(&mut p, origin);
        // Before drop: expect [Save, Translate(origin)]
        assert_eq!(p.events, [PaintEvent::Save, PaintEvent::Translate(origin)]);
        drop(guard);
    }

    #[test]
    fn drop_records_exactly_one_restore() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(1, 2);
        {
            let _guard = TranslateGuard::new(&mut p, origin);
        }
        // Exactly one Restore at the end
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(origin),
                PaintEvent::Restore,
            ]
        );
        // Only one Restore
        let restore_count = p
            .events
            .iter()
            .filter(|e| **e == PaintEvent::Restore)
            .count();
        assert_eq!(restore_count, 1);
    }

    #[test]
    fn full_lifecycle_records_save_translate_restore_in_order() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(10, 20);
        {
            let _guard = TranslateGuard::new(&mut p, origin);
        }
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(origin),
                PaintEvent::Restore,
            ]
        );
    }

    #[test]
    fn painter_accessor_returns_same_painter() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(5, 6);
        {
            let mut guard = TranslateGuard::new(&mut p, origin);
            // Access the painter through the guard and call fill_rect
            use quartzite_geometry::{Rect, Size};
            use quartzite_paint_api::Brush;
            guard.painter().fill_rect(
                Rect::new(Point::new(0, 0), Size::new(10, 10)),
                &Brush::solid(quartzite_paint_api::Color::WHITE),
            );
        }
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(origin),
                PaintEvent::FillRect,
                PaintEvent::Restore,
            ]
        );
    }

    #[test]
    fn translate_origin_zero() {
        let mut p = RecordingPainter::new();
        let origin = Point::new(0, 0);
        {
            let _guard = TranslateGuard::new(&mut p, origin);
        }
        // Zero origin must still record Translate (no zero-skipping)
        assert_eq!(
            p.events,
            [
                PaintEvent::Save,
                PaintEvent::Translate(Point::new(0, 0)),
                PaintEvent::Restore,
            ]
        );
    }
}
