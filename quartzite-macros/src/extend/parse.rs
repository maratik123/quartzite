use syn::{Data, DeriveInput, Fields, Ident, Type, parse2, spanned::Spanned};

use crate::util::{Level, extract_attr, extract_undocumented_per_item, parse_undocumented_kv};

/// How a field tagged with `#[widget_children]` provides children.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum WidgetChildrenKind {
    /// `Vec<ObjectId>` — emit `WidgetChildren::Slice(&self.field)`.
    Slice,
    /// `Option<ObjectId>` — emit `WidgetChildren::Optional(self.field)`.
    Optional,
}

/// A field annotated with `#[widget_children(slice|optional)]`.
#[cfg_attr(test, derive(Debug))]
pub(crate) struct WidgetChildrenField {
    pub ident: Ident,
    pub kind: WidgetChildrenKind,
    /// Whether the field has a `#[doc = "..."]` attribute.
    pub doc_present: bool,
    /// Per-item level from `#[undocumented(allow|warn|deny)]` on this field.
    pub per_item_level: Option<Level>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct ExtendInput {
    pub ident: Ident,
    pub generics: syn::Generics,
    pub is_root: bool,
    pub base_field: Option<BaseField>,
    pub mixin_fields: Vec<MixinField>,
    /// Variant name from `#[widget_view(variant = "X")]`, used by codegen when
    /// `base.ty_ident == "WidgetBase"` to emit the matching `WidgetView::X(self)` arm.
    pub widget_view_variant: Option<String>,
    /// Field that provides children via `#[widget_children(slice|optional)]`.
    pub widget_children_field: Option<WidgetChildrenField>,
    /// Level from `#[extend(undocumented = "...")]` sibling attribute on the struct.
    pub per_invocation_level: Option<Level>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct BaseField {
    pub ident: Ident,
    pub ty_ident: Ident,
    pub ty: Type,
    /// Whether the field has a `#[doc = "..."]` attribute.
    pub doc_present: bool,
    /// Per-item level from `#[undocumented(allow|warn|deny)]` on this field.
    pub per_item_level: Option<Level>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct MixinField {
    pub ident: Ident,
    pub ty_ident: Ident,
    pub ty: Type,
    /// Whether the field has a `#[doc = "..."]` attribute.
    pub doc_present: bool,
    /// Per-item level from `#[undocumented(allow|warn|deny)]` on this field.
    pub per_item_level: Option<Level>,
}

/// Removes and parses `#[widget_view(variant = "X")]` from `attrs`.
/// Returns `Ok(None)` if the attribute is absent; `Ok(Some("X"))` on success;
/// `Err` if the attribute is present but malformed.
fn extract_widget_view_variant(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Option<String>> {
    let Some(pos) = attrs.iter().position(|a| a.path().is_ident("widget_view")) else {
        return Ok(None);
    };
    let attr = attrs.remove(pos);
    let span = attr.span();
    let mut variant: Option<String> = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("variant") {
            let value = meta.value()?;
            let s: syn::LitStr = value.parse()?;
            variant = Some(s.value());
            Ok(())
        } else {
            Err(meta.error("expected `variant = \"VariantName\"`"))
        }
    })?;
    if variant.is_none() {
        return Err(syn::Error::new(
            span,
            "#[widget_view] requires `variant = \"VariantName\"`",
        ));
    }
    Ok(variant)
}

/// Removes and parses `#[widget_children(slice|optional)]` from `attrs`.
/// Returns `Ok(None)` if absent; `Ok(Some(kind))` on success; `Err` if malformed.
fn extract_widget_children_kind(
    attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<Option<WidgetChildrenKind>> {
    let Some(pos) = attrs
        .iter()
        .position(|a| a.path().is_ident("widget_children"))
    else {
        return Ok(None);
    };
    let attr = attrs.remove(pos);
    let span = attr.span();
    let mut kind: Option<WidgetChildrenKind> = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("slice") {
            kind = Some(WidgetChildrenKind::Slice);
            Ok(())
        } else if meta.path.is_ident("optional") {
            kind = Some(WidgetChildrenKind::Optional);
            Ok(())
        } else {
            Err(meta.error("expected `slice` or `optional`"))
        }
    })?;
    if kind.is_none() {
        return Err(syn::Error::new(
            span,
            "#[widget_children] requires `slice` or `optional`",
        ));
    }
    Ok(kind)
}

/// Extracts the last path-segment ident from a `Type::Path`.
fn extract_last_ident(ty: &Type, context: &Ident) -> syn::Result<Ident> {
    let seg = match ty {
        Type::Path(tp) => tp.path.segments.last().cloned(),
        _ => None,
    };
    seg.map(|s| s.ident).ok_or_else(|| {
        syn::Error::new(
            context.span(),
            "expected a simple type path for #[base] / #[mixin] field",
        )
    })
}

pub(crate) fn parse(input: proc_macro2::TokenStream) -> syn::Result<ExtendInput> {
    let mut derive: DeriveInput = parse2(input)?;

    // Extract per-invocation level from `#[extend(undocumented = "...")]` sibling attribute.
    let per_invocation_level = extract_extend_invocation_level(&mut derive.attrs)?;

    // Must be a named-field struct.
    let fields = match &derive.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.clone(),
            _ => {
                return Err(syn::Error::new(
                    derive.ident.span(),
                    "#[derive(Extend)] only supports named-field structs",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                derive.ident.span(),
                "#[derive(Extend)] only supports structs",
            ));
        }
    };

    // Check for #[root] on the struct itself; strip it.
    let is_root = extract_attr(&mut derive.attrs, "root");
    let widget_view_variant = extract_widget_view_variant(&mut derive.attrs)?;

    // Root + generic is unsupported: the generated `As{Self}` trait's return type
    // would reference the bare ident without type params, causing a type mismatch.
    if is_root && !derive.generics.params.is_empty() {
        return Err(syn::Error::new(
            derive.generics.span(),
            "#[derive(Extend)] with #[root] does not support generic structs",
        ));
    }

    // Classify fields.
    let mut base_fields: Vec<BaseField> = Vec::new();
    let mut mixin_fields: Vec<MixinField> = Vec::new();
    let mut widget_children_field: Option<WidgetChildrenField> = None;

    for mut field in fields.named {
        // Capture doc_present and per_item_level before consuming attributes.
        let doc_present = field.attrs.iter().any(|a| a.path().is_ident("doc"));
        let per_item_level = extract_undocumented_per_item(&mut field.attrs)?;
        let is_base = extract_attr(&mut field.attrs, "base");
        let is_mixin = extract_attr(&mut field.attrs, "mixin");
        let wc_kind = extract_widget_children_kind(&mut field.attrs)?;
        let field_ident = field.ident.clone().expect("named field has ident");

        if is_base {
            let ty_ident = extract_last_ident(&field.ty, &field_ident)?;
            base_fields.push(BaseField {
                ident: field_ident.clone(),
                ty_ident,
                ty: field.ty.clone(),
                doc_present,
                per_item_level,
            });
        } else if is_mixin {
            let ty_ident = extract_last_ident(&field.ty, &field_ident)?;
            mixin_fields.push(MixinField {
                ident: field_ident.clone(),
                ty_ident,
                ty: field.ty.clone(),
                doc_present,
                per_item_level,
            });
        }

        if let Some(kind) = wc_kind {
            if widget_children_field.is_some() {
                return Err(syn::Error::new(
                    field_ident.span(),
                    "at most one #[widget_children] field allowed",
                ));
            }
            // Share the same doc_present / per_item_level already captured above
            // (the #[undocumented] attr was already consumed from this field).
            widget_children_field = Some(WidgetChildrenField {
                ident: field_ident,
                kind,
                doc_present,
                per_item_level,
            });
        }
    }

    if base_fields.len() >= 2 {
        return Err(syn::Error::new(
            base_fields[1].ident.span(),
            "at most one #[base] field allowed",
        ));
    }

    let base_field = base_fields.into_iter().next();

    if !is_root && base_field.is_none() && mixin_fields.is_empty() {
        return Err(syn::Error::new(
            derive.ident.span(),
            "#[derive(Extend)] requires #[root], at least one #[base], or at least one #[mixin] field",
        ));
    }

    Ok(ExtendInput {
        ident: derive.ident,
        generics: derive.generics,
        is_root,
        base_field,
        mixin_fields,
        widget_view_variant,
        widget_children_field,
        per_invocation_level,
    })
}

/// Extracts the per-invocation level from `#[extend(undocumented = "...")]` on the struct.
fn extract_extend_invocation_level(
    attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<Option<Level>> {
    let Some(pos) = attrs.iter().position(|a| a.path().is_ident("extend")) else {
        return Ok(None);
    };
    let attr = attrs.remove(pos);
    let mut level: Option<Level> = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("undocumented") {
            let value = meta.value()?;
            let s: syn::LitStr = value.parse()?;
            level = Some(parse_undocumented_kv(&s)?);
            Ok(())
        } else {
            Err(meta.error("unknown `#[extend(...)]` key; expected `undocumented = \"...\"`"))
        }
    })?;
    Ok(level)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    fn parse_ok(ts: TokenStream) -> super::ExtendInput {
        super::parse(ts).expect("should parse successfully")
    }

    fn parse_err(ts: TokenStream) -> String {
        super::parse(ts).unwrap_err().to_string()
    }

    // AC3: two #[base] fields → compile error.
    #[test]
    fn two_base_fields_errors() {
        let err = parse_err(quote! {
            struct Button {
                #[base]
                widget: Widget,
                #[base]
                other: Other,
            }
        });
        assert!(
            err.contains("at most one #[base] field allowed"),
            "unexpected: {err}"
        );
    }

    // AC4: no #[root], no #[base], no #[mixin] → compile error.
    #[test]
    fn no_markers_errors() {
        let err = parse_err(quote! {
            struct Plain { x: i32 }
        });
        assert!(
            err.contains("#[derive(Extend)] requires"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn root_with_no_fields_ok() {
        let ir = parse_ok(quote! {
            #[root]
            struct ObjectBase {}
        });
        assert!(ir.is_root);
        assert!(ir.base_field.is_none());
        assert!(ir.mixin_fields.is_empty());
    }

    #[test]
    fn base_field_classified() {
        let ir = parse_ok(quote! {
            struct Button {
                #[base]
                widget: Widget,
                pub x: i32,
            }
        });
        assert!(!ir.is_root);
        assert_eq!(ir.base_field.as_ref().unwrap().ident, "widget");
        assert_eq!(ir.base_field.as_ref().unwrap().ty_ident, "Widget");
        assert!(ir.mixin_fields.is_empty());
    }

    #[test]
    fn mixin_field_classified() {
        let ir = parse_ok(quote! {
            struct Panel {
                #[mixin]
                layout_base: LayoutBase,
            }
        });
        assert_eq!(ir.mixin_fields.len(), 1);
        assert_eq!(ir.mixin_fields[0].ident, "layout_base");
        assert!(ir.base_field.is_none());
    }

    #[test]
    fn generic_root_struct_errors() {
        let err = parse_err(quote! {
            #[root]
            struct Foo<T> { x: T }
        });
        assert!(err.contains("generic"), "unexpected: {err}");
    }

    #[test]
    fn generic_non_root_struct_parses_ok() {
        let ir = parse_ok(quote! {
            struct Foo<T> {
                #[base]
                widget: Widget,
                data: T,
            }
        });
        assert!(!ir.generics.params.is_empty(), "generics should be stored");
        assert!(ir.base_field.is_some());
    }

    #[test]
    fn generic_with_lifetime_parses_ok() {
        let ir = parse_ok(quote! {
            struct Foo<'a> {
                #[base]
                widget: Widget,
                data: &'a str,
            }
        });
        assert!(
            !ir.generics.params.is_empty(),
            "lifetime param should be stored"
        );
    }

    #[test]
    fn widget_view_variant_parsed() {
        let ir = parse_ok(quote! {
            #[widget_view(variant = "Button")]
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert_eq!(ir.widget_view_variant.as_deref(), Some("Button"));
    }

    #[test]
    fn widget_view_variant_absent_is_none() {
        let ir = parse_ok(quote! {
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(ir.widget_view_variant.is_none());
    }

    #[test]
    fn widget_view_missing_variant_key_errors() {
        let err = parse_err(quote! {
            #[widget_view()]
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(err.contains("#[widget_view] requires"), "unexpected: {err}");
    }

    #[test]
    fn widget_view_unknown_key_errors() {
        let err = parse_err(quote! {
            #[widget_view(name = "Button")]
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(err.contains("expected `variant"), "unexpected: {err}");
    }

    #[test]
    fn widget_children_slice_parsed() {
        let ir = parse_ok(quote! {
            struct Container {
                #[base]
                widget: Widget,
                #[widget_children(slice)]
                children: Vec<ObjectId>,
            }
        });
        let wc = ir.widget_children_field.expect("widget_children_field set");
        assert_eq!(wc.ident, "children");
        assert_eq!(wc.kind, super::WidgetChildrenKind::Slice);
    }

    #[test]
    fn widget_children_optional_parsed() {
        let ir = parse_ok(quote! {
            struct ScrollArea {
                #[base]
                widget: Widget,
                #[widget_children(optional)]
                content: Option<ObjectId>,
            }
        });
        let wc = ir.widget_children_field.expect("widget_children_field set");
        assert_eq!(wc.ident, "content");
        assert_eq!(wc.kind, super::WidgetChildrenKind::Optional);
    }

    #[test]
    fn widget_children_absent_is_none() {
        let ir = parse_ok(quote! {
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(ir.widget_children_field.is_none());
    }

    #[test]
    fn two_widget_children_fields_errors() {
        let err = parse_err(quote! {
            struct Bad {
                #[base]
                widget: Widget,
                #[widget_children(slice)]
                children: Vec<ObjectId>,
                #[widget_children(optional)]
                other: Option<ObjectId>,
            }
        });
        assert!(
            err.contains("at most one #[widget_children] field allowed"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn widget_children_empty_errors() {
        let err = parse_err(quote! {
            struct Bad {
                #[base]
                widget: Widget,
                #[widget_children()]
                children: Vec<ObjectId>,
            }
        });
        assert!(
            err.contains("#[widget_children] requires"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn widget_children_unknown_key_errors() {
        let err = parse_err(quote! {
            struct Bad {
                #[base]
                widget: Widget,
                #[widget_children(vec)]
                children: Vec<ObjectId>,
            }
        });
        assert!(
            err.contains("expected `slice` or `optional`"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn widget_children_on_base_field_ok() {
        // A field may be both #[base] and #[widget_children] — both are recorded.
        let ir = parse_ok(quote! {
            struct Foo {
                #[base]
                #[widget_children(slice)]
                widget: Widget,
            }
        });
        assert!(ir.base_field.is_some());
        assert!(ir.widget_children_field.is_some());
    }
}
