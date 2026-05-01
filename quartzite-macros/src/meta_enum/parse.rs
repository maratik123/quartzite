use syn::{Data, DeriveInput, Expr, Fields, Ident, Lit, parse2, spanned::Spanned};

#[cfg_attr(test, derive(Debug))]
pub(crate) struct MetaEnumInput {
    pub ident: Ident,
    pub variants: Vec<EnumVariant>,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct EnumVariant {
    pub ident: Ident,
    pub value: i64,
}

pub(crate) fn parse(input: proc_macro2::TokenStream) -> syn::Result<MetaEnumInput> {
    let derive: DeriveInput = parse2(input)?;

    let variants_data = match derive.data {
        Data::Enum(e) => e.variants,
        _ => {
            return Err(syn::Error::new(
                derive.ident.span(),
                "#[derive(MetaEnum)] only supports enums",
            ));
        }
    };

    let mut variants = Vec::new();
    let mut next_value: i64 = 0;

    for variant in variants_data {
        match &variant.fields {
            Fields::Unit => {}
            _ => {
                return Err(syn::Error::new(
                    variant.ident.span(),
                    "#[derive(MetaEnum)] only supports unit variants (no tuple or struct fields)",
                ));
            }
        }

        let value = if let Some((_, expr)) = &variant.discriminant {
            match expr {
                Expr::Lit(el) => match &el.lit {
                    Lit::Int(n) => n.base10_parse::<i64>().map_err(|_| {
                        syn::Error::new(n.span(), "discriminant value out of range for i64")
                    })?,
                    other => {
                        return Err(syn::Error::new(
                            other.span(),
                            "#[derive(MetaEnum)] only supports integer literal discriminants",
                        ));
                    }
                },
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "#[derive(MetaEnum)] only supports integer literal discriminants",
                    ));
                }
            }
        } else {
            next_value
        };

        next_value = value.checked_add(1).ok_or_else(|| {
            syn::Error::new(
                variant.ident.span(),
                "discriminant overflow: this variant has value i64::MAX; \
                 any subsequent variant without an explicit discriminant would overflow",
            )
        })?;
        variants.push(EnumVariant {
            ident: variant.ident,
            value,
        });
    }

    Ok(MetaEnumInput {
        ident: derive.ident,
        variants,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    fn parse_ok(ts: TokenStream) -> MetaEnumInput {
        parse(ts).expect("should parse successfully")
    }

    fn parse_err(ts: TokenStream) -> String {
        parse(ts).unwrap_err().to_string()
    }

    #[test]
    fn unit_variants_auto_discriminant() {
        let ir = parse_ok(quote! {
            enum Color { Red, Green, Blue }
        });
        assert_eq!(ir.variants.len(), 3);
        assert_eq!(ir.variants[0].value, 0);
        assert_eq!(ir.variants[1].value, 1);
        assert_eq!(ir.variants[2].value, 2);
    }

    #[test]
    fn explicit_discriminant() {
        let ir = parse_ok(quote! {
            enum Status { Ok = 200, NotFound = 404 }
        });
        assert_eq!(ir.variants[0].value, 200);
        assert_eq!(ir.variants[1].value, 404);
    }

    #[test]
    fn mixed_discriminants_auto_increment_from_last() {
        let ir = parse_ok(quote! {
            enum Foo { A = 10, B, C }
        });
        assert_eq!(ir.variants[0].value, 10);
        assert_eq!(ir.variants[1].value, 11);
        assert_eq!(ir.variants[2].value, 12);
    }

    #[test]
    fn tuple_variant_errors() {
        let err = parse_err(quote! {
            enum Bad { Good, Bad(i32) }
        });
        assert!(err.contains("unit variants"), "unexpected: {err}");
    }

    #[test]
    fn struct_variant_errors() {
        let err = parse_err(quote! {
            enum Bad { Good, Bad { x: i32 } }
        });
        assert!(err.contains("unit variants"), "unexpected: {err}");
    }

    #[test]
    fn non_enum_errors() {
        let err = parse_err(quote! {
            struct Foo {}
        });
        assert!(err.contains("only supports enums"), "unexpected: {err}");
    }
}
