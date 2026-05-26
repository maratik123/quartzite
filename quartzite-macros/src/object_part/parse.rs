use syn::{ImplItem, ItemImpl, parse2};

use crate::object_impl::parse::{MethodItem, ObjectImplInput, extract_params};
use crate::util::{Level, extract_attr, extract_undocumented_per_item, parse_undocumented_kv};

pub(crate) fn parse(
    attr: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> syn::Result<ObjectImplInput> {
    // Accept zero arguments OR exactly `undocumented = "..."` as the sole key-value.
    let mut per_invocation_level: Option<Level> = None;
    if !attr.is_empty() {
        let attr_result = syn::parse2::<syn::MetaNameValue>(attr.clone());
        match attr_result {
            Ok(ref nv) if nv.path.is_ident("undocumented") => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(ref s),
                    ..
                }) = nv.value
                {
                    per_invocation_level = Some(parse_undocumented_kv(s)?);
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`#[object_part]` takes no arguments",
                ));
            }
        }
    }
    let mut item: ItemImpl = parse2(input)?;

    let generics = item.generics.clone();
    let trait_path = item.trait_.as_ref().map(|(_, path, _)| path.clone());
    let self_ty = *item.self_ty;
    let self_ty_ident = extract_self_ty_ident_from(&self_ty)?;

    let mut methods = Vec::new();
    let mut other_items = Vec::new();

    for impl_item in item.items.drain(..) {
        match impl_item {
            ImplItem::Fn(mut fn_item) => {
                let is_slot = extract_attr(&mut fn_item.attrs, "slot");
                let is_invoke = extract_attr(&mut fn_item.attrs, "invoke");

                if is_slot || is_invoke {
                    let ident = fn_item.sig.ident.clone();
                    let doc_present =
                        fn_item.attrs.iter().any(|a| a.path().is_ident("doc"));
                    let per_item_level =
                        extract_undocumented_per_item(&mut fn_item.attrs)?;
                    let params = extract_params(&fn_item.sig.inputs)?;
                    let ret_ty = fn_item.sig.output.clone();
                    methods.push(MethodItem {
                        ident,
                        params,
                        ret_ty,
                        doc_present,
                        per_item_level,
                    });
                }
                other_items.push(ImplItem::Fn(fn_item));
            }
            other => other_items.push(other),
        }
    }

    Ok(ObjectImplInput {
        self_ty,
        self_ty_ident,
        generics,
        trait_path,
        methods,
        other_items,
        per_invocation_level,
    })
}

fn extract_self_ty_ident_from(self_ty: &syn::Type) -> syn::Result<syn::Ident> {
    use syn::spanned::Spanned;
    let err = || {
        syn::Error::new(
            self_ty.span(),
            "#[object_part] self type must be a simple path (e.g. `Foo` or `my_mod::Foo`)",
        )
    };
    let syn::Type::Path(tp) = self_ty else {
        return Err(err());
    };
    tp.path
        .segments
        .last()
        .map(|s| s.ident.clone())
        .ok_or_else(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    fn parse_ok(ts: TokenStream) -> ObjectImplInput {
        parse(quote! {}, ts).expect("should parse successfully")
    }

    // AC8: non-empty attr produces compile error.
    #[test]
    fn non_empty_attr_errors() {
        let err = parse(quote! { something }, quote! { impl Foo {} })
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("#[object_part]"),
            "error should mention #[object_part]: {err}"
        );
        assert!(
            err.contains("no arguments"),
            "error should mention 'no arguments': {err}"
        );
    }

    #[test]
    fn inherent_impl_parsed() {
        let ir = parse_ok(quote! {
            impl Foo {
                #[slot]
                fn reset(&mut self) {}
            }
        });
        assert_eq!(ir.self_ty_ident, "Foo");
        assert!(ir.trait_path.is_none());
        assert_eq!(ir.methods.len(), 1);
        assert_eq!(ir.methods[0].ident, "reset");
    }

    #[test]
    fn trait_impl_parsed() {
        let ir = parse_ok(quote! {
            impl MyTrait for Foo {
                #[slot]
                fn on_event(&mut self) {}
            }
        });
        assert_eq!(ir.self_ty_ident, "Foo");
        assert!(ir.trait_path.is_some());
        assert_eq!(ir.methods.len(), 1);
    }

    #[test]
    fn invoke_method_classified() {
        // Proves #[invoke] (the post-rename name) is correctly parsed by object_part.
        let ir = parse_ok(quote! {
            impl Foo {
                #[invoke]
                fn compute(&self, x: i32) -> i32 { x }
            }
        });
        assert_eq!(ir.methods.len(), 1);
        assert_eq!(ir.methods[0].ident, "compute");
    }

    #[test]
    fn undocumented_allow_on_invoke_method_parses_cleanly() {
        // Proves that #[undocumented(allow)] on an #[invoke] method does not cause a
        // parse error — it is an inert helper attribute that the parser skips.
        let ir = parse_ok(quote! {
            impl Foo {
                #[undocumented(allow)]
                #[invoke]
                fn compute(&self, x: i32) -> i32 { x }
            }
        });
        assert_eq!(ir.methods.len(), 1);
        assert_eq!(ir.methods[0].per_item_level, Some(Level::Allow));
    }

    #[test]
    fn per_invocation_deny_sets_level() {
        // Proves #[object_part(undocumented = "deny")] is accepted.
        let ir = parse(
            quote! { undocumented = "deny" },
            quote! { impl Foo { fn bar(&self) {} } },
        )
        .expect("should parse successfully");
        assert_eq!(ir.per_invocation_level, Some(Level::Deny));
    }

    #[test]
    fn doc_present_false_on_slot_without_doc() {
        let ir = parse_ok(quote! {
            impl Foo {
                #[slot]
                fn reset(&mut self) {}
            }
        });
        assert!(!ir.methods[0].doc_present);
    }

    #[test]
    fn doc_present_true_on_slot_with_doc() {
        let ir = parse_ok(quote! {
            impl Foo {
                /// Reset the counter.
                #[slot]
                fn reset(&mut self) {}
            }
        });
        assert!(ir.methods[0].doc_present);
    }
}
