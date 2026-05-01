use syn::{parse2, spanned::Spanned, Data, DeriveInput, Fields, Ident, Type};

use crate::util::extract_attr;

#[cfg_attr(test, derive(Debug))]
pub(crate) struct ExtendInput {
    pub ident: Ident,
    pub is_root: bool,
    pub base_field: Option<BaseField>,
    pub mixin_fields: Vec<MixinField>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct BaseField {
    pub ident: Ident,
    pub ty_ident: Ident,
    pub ty: Type,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct MixinField {
    pub ident: Ident,
    pub ty_ident: Ident,
    pub ty: Type,
}

pub(crate) fn parse(input: proc_macro2::TokenStream) -> syn::Result<ExtendInput> {
    let mut derive: DeriveInput = parse2(input)?;

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

    // Reject generic type or lifetime parameters.
    if !derive.generics.params.is_empty() {
        return Err(syn::Error::new(
            derive.generics.span(),
            "generic structs not yet supported by #[derive(Extend)]",
        ));
    }

    // Check for #[root] on the struct itself; strip it.
    let is_root = extract_attr(&mut derive.attrs, "root");

    // Classify fields.
    let mut base_fields: Vec<BaseField> = Vec::new();
    let mut mixin_fields: Vec<MixinField> = Vec::new();

    for mut field in fields.named.into_iter() {
        let is_base = extract_attr(&mut field.attrs, "base");
        let is_mixin = extract_attr(&mut field.attrs, "mixin");
        let field_ident = field.ident.clone().expect("named field has ident");

        if is_base {
            let ty_ident = extract_last_ident(&field.ty, &field_ident)?;
            base_fields.push(BaseField {
                ident: field_ident,
                ty_ident,
                ty: field.ty.clone(),
            });
        } else if is_mixin {
            let ty_ident = extract_last_ident(&field.ty, &field_ident)?;
            mixin_fields.push(MixinField {
                ident: field_ident,
                ty_ident,
                ty: field.ty.clone(),
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
        is_root,
        base_field,
        mixin_fields,
    })
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
    fn generic_struct_errors() {
        let err = parse_err(quote! {
            #[root]
            struct Foo<T> { x: T }
        });
        assert!(err.contains("generic"), "unexpected: {err}");
    }
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
