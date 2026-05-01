use syn::{
    Data, DeriveInput, Field, Fields, GenericArgument, Ident, PathArguments, Type, parse2,
    spanned::Spanned,
};

use crate::util::extract_attr;

#[cfg_attr(test, derive(Debug))]
pub(crate) struct ObjectInput {
    pub ident: Ident,
    pub props: Vec<PropField>,
    pub signals: Vec<SignalField>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct PropField {
    pub ident: Ident,
    pub ty: Type,
    pub notify: Option<Ident>,
    pub read_only: bool,
    pub stored: bool,
    pub designable: bool,
    pub user: bool,
    pub constant: bool,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct SignalField {
    pub ident: Ident,
    pub args_ty: Type,
}

pub(crate) fn parse(input: proc_macro2::TokenStream) -> syn::Result<ObjectInput> {
    let mut derive: DeriveInput = parse2(input)?;

    let fields = match &derive.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.clone(),
            _ => {
                return Err(syn::Error::new(
                    derive.ident.span(),
                    "#[derive(Object)] only supports named-field structs",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                derive.ident.span(),
                "#[derive(Object)] only supports structs",
            ));
        }
    };

    if !derive.generics.params.is_empty() {
        return Err(syn::Error::new(
            derive.generics.span(),
            "generic structs not yet supported by #[derive(Object)]",
        ));
    }

    let _ = &mut derive.attrs; // struct-level attrs not consumed here

    let mut props = Vec::new();
    let mut signals = Vec::new();

    for mut field in fields.named.into_iter() {
        let has_prop = has_attr(&field, "prop");
        let has_signal = extract_attr(&mut field.attrs, "signal");

        if has_prop {
            props.push(parse_prop_field(field)?);
        } else if has_signal {
            let field_ident = field.ident.clone().expect("named field has ident");
            let args_ty = extract_signal_args(&field.ty, &field_ident)?;
            signals.push(SignalField {
                ident: field_ident,
                args_ty,
            });
        }
    }

    Ok(ObjectInput {
        ident: derive.ident,
        props,
        signals,
    })
}

/// Returns true if the field has a `#[prop]` or `#[prop(...)]` attribute (without removing it).
fn has_attr(field: &Field, name: &str) -> bool {
    field.attrs.iter().any(|a| a.path().is_ident(name))
}

fn parse_prop_field(mut field: Field) -> syn::Result<PropField> {
    let field_ident = field.ident.clone().expect("named field has ident");

    // Find and remove the #[prop] or #[prop(...)] attribute.
    let prop_attr_idx = field
        .attrs
        .iter()
        .position(|a| a.path().is_ident("prop"))
        .expect("checked above");
    let prop_attr = field.attrs.remove(prop_attr_idx);

    let mut notify: Option<Ident> = None;
    let mut read_only = false;
    let mut stored = true;
    let mut designable = true;
    let mut user = false;
    let mut constant = false;

    // #[prop] with no parens is valid; #[prop(...)] is parsed with nested meta.
    match &prop_attr.meta {
        syn::Meta::Path(_) => {
            // bare #[prop], all defaults
        }
        syn::Meta::List(_) => {
            prop_attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("notify") {
                    meta.input.parse::<syn::Token![=]>()?;
                    notify = Some(meta.input.parse::<Ident>()?);
                } else if meta.path.is_ident("read_only") {
                    read_only = true;
                } else if meta.path.is_ident("stored") {
                    meta.input.parse::<syn::Token![=]>()?;
                    let val: syn::LitBool = meta.input.parse()?;
                    stored = val.value;
                } else if meta.path.is_ident("designable") {
                    meta.input.parse::<syn::Token![=]>()?;
                    let val: syn::LitBool = meta.input.parse()?;
                    designable = val.value;
                } else if meta.path.is_ident("user") {
                    user = true;
                } else if meta.path.is_ident("constant") {
                    constant = true;
                } else {
                    return Err(meta.error(format!(
                        "unknown #[prop] option `{}`",
                        meta.path
                            .get_ident()
                            .map_or("?".to_owned(), |i| i.to_string())
                    )));
                }
                Ok(())
            })?;
        }
        syn::Meta::NameValue(_) => {
            return Err(syn::Error::new(
                prop_attr.span(),
                "#[prop] does not support name-value syntax; use #[prop(notify = name)]",
            ));
        }
    }

    if constant && notify.is_some() {
        return Err(syn::Error::new(
            field_ident.span(),
            "constant property cannot have a notify signal",
        ));
    }

    Ok(PropField {
        ident: field_ident,
        ty: field.ty,
        notify,
        read_only,
        stored,
        designable,
        user,
        constant,
    })
}

/// Extracts the `Args` type from `Signal<Args>` (last path segment named `Signal`).
fn extract_signal_args(ty: &Type, context: &Ident) -> syn::Result<Type> {
    let err = || {
        syn::Error::new(
            context.span(),
            "#[signal] field must have type `Signal<Args>` (e.g. `Signal<(i32,)>`)",
        )
    };
    let tp = match ty {
        Type::Path(tp) => tp,
        _ => return Err(err()),
    };
    let seg = tp.path.segments.last().ok_or_else(err)?;
    if seg.ident != "Signal" {
        return Err(err());
    }
    let angle = match &seg.arguments {
        PathArguments::AngleBracketed(a) => a,
        _ => return Err(err()),
    };
    match angle.args.first() {
        Some(GenericArgument::Type(inner)) => Ok(inner.clone()),
        _ => Err(err()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    fn parse_ok(ts: TokenStream) -> ObjectInput {
        parse(ts).expect("should parse successfully")
    }

    fn parse_err(ts: TokenStream) -> String {
        parse(ts).unwrap_err().to_string()
    }

    #[test]
    fn bare_prop_defaults() {
        let ir = parse_ok(quote! {
            struct Foo {
                #[prop]
                pub count: i32,
            }
        });
        assert_eq!(ir.props.len(), 1);
        let p = &ir.props[0];
        assert_eq!(p.ident, "count");
        assert!(!p.read_only);
        assert!(p.stored);
        assert!(p.designable);
        assert!(!p.user);
        assert!(!p.constant);
        assert!(p.notify.is_none());
    }

    #[test]
    fn prop_with_notify() {
        let ir = parse_ok(quote! {
            struct Foo {
                #[prop(notify = count_changed)]
                pub count: i32,
            }
        });
        let p = &ir.props[0];
        assert_eq!(p.notify.as_ref().unwrap(), "count_changed");
    }

    #[test]
    fn prop_read_only() {
        let ir = parse_ok(quote! {
            struct Foo {
                #[prop(read_only)]
                pub val: i32,
            }
        });
        assert!(ir.props[0].read_only);
    }

    #[test]
    fn prop_stored_false() {
        let ir = parse_ok(quote! {
            struct Foo {
                #[prop(stored = false)]
                pub val: i32,
            }
        });
        assert!(!ir.props[0].stored);
    }

    #[test]
    fn prop_constant() {
        let ir = parse_ok(quote! {
            struct Foo {
                #[prop(constant)]
                pub val: i32,
            }
        });
        assert!(ir.props[0].constant);
    }

    #[test]
    fn prop_constant_with_notify_errors() {
        let err = parse_err(quote! {
            struct Foo {
                #[prop(constant, notify = changed)]
                pub val: i32,
            }
        });
        assert!(
            err.contains("constant property cannot have a notify signal"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn signal_field_extracted() {
        let ir = parse_ok(quote! {
            struct Foo {
                #[signal]
                pub value_changed: Signal<(i32,)>,
            }
        });
        assert_eq!(ir.signals.len(), 1);
        assert_eq!(ir.signals[0].ident, "value_changed");
    }

    #[test]
    fn signal_wrong_type_errors() {
        let err = parse_err(quote! {
            struct Foo {
                #[signal]
                pub value_changed: i32,
            }
        });
        assert!(err.contains("Signal<Args>"), "unexpected: {err}");
    }

    #[test]
    fn mixed_props_and_signals() {
        let ir = parse_ok(quote! {
            struct Foo {
                #[prop(notify = count_changed)]
                pub count: i32,
                #[signal]
                pub count_changed: Signal<(i32,)>,
                pub other: String,
            }
        });
        assert_eq!(ir.props.len(), 1);
        assert_eq!(ir.signals.len(), 1);
    }

    #[test]
    fn unknown_prop_option_errors() {
        let err = parse_err(quote! {
            struct Foo {
                #[prop(unknown_opt)]
                pub val: i32,
            }
        });
        assert!(err.contains("unknown #[prop] option"), "unexpected: {err}");
    }
}
