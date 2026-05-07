use syn::{ImplItem, ItemImpl, parse2};

use crate::object_impl::parse::{MethodItem, ObjectImplInput, extract_params};
use crate::util::extract_attr;

pub(crate) fn parse(
    attr: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> syn::Result<ObjectImplInput> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "`#[object_part]` takes no arguments",
        ));
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
                let is_invokable = extract_attr(&mut fn_item.attrs, "invokable");

                if is_slot || is_invokable {
                    let ident = fn_item.sig.ident.clone();
                    let params = extract_params(&fn_item.sig.inputs)?;
                    let ret_ty = fn_item.sig.output.clone();
                    methods.push(MethodItem {
                        ident,
                        params,
                        ret_ty,
                    });
                    other_items.push(ImplItem::Fn(fn_item));
                } else {
                    other_items.push(ImplItem::Fn(fn_item));
                }
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
    let tp = match self_ty {
        syn::Type::Path(tp) => tp,
        _ => return Err(err()),
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
}
