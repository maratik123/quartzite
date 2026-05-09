//! AC13 contract: `quartzite-widgets` re-exports `Alignment` / `Font` / `Palette` /
//! `ColorRole` from upstream crates rather than defining them locally.
//!
//! These tests use [`core::any::TypeId`] to assert that the type the widgets crate
//! exposes is identically the upstream type — not a wrapper, not a newtype.

use core::any::TypeId;

#[test]
fn widgets_alignment_is_geometry_alignment() {
    assert_eq!(
        TypeId::of::<quartzite_widgets::Alignment>(),
        TypeId::of::<quartzite_geometry::Alignment>(),
    );
}

#[test]
fn widgets_font_is_paint_api_font() {
    assert_eq!(
        TypeId::of::<quartzite_widgets::Font>(),
        TypeId::of::<quartzite_paint_api::Font>(),
    );
}

#[test]
fn widgets_font_weight_is_paint_api_font_weight() {
    assert_eq!(
        TypeId::of::<quartzite_widgets::FontWeight>(),
        TypeId::of::<quartzite_paint_api::FontWeight>(),
    );
}

#[test]
fn widgets_palette_is_style_types_palette() {
    assert_eq!(
        TypeId::of::<quartzite_widgets::Palette>(),
        TypeId::of::<quartzite_style_types::Palette>(),
    );
}

#[test]
fn widgets_color_role_is_style_types_color_role() {
    assert_eq!(
        TypeId::of::<quartzite_widgets::ColorRole>(),
        TypeId::of::<quartzite_style_types::ColorRole>(),
    );
}
