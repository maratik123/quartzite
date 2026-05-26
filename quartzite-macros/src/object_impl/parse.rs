use syn::{FnArg, Ident, ImplItem, ItemImpl, Pat, ReturnType, Type, parse2, spanned::Spanned};

use crate::util::extract_attr;

#[cfg_attr(test, derive(Debug))]
pub(crate) struct ObjectImplInput {
    pub self_ty: Type,
    pub self_ty_ident: Ident,
    pub generics: syn::Generics,
    pub trait_path: Option<syn::Path>,
    pub methods: Vec<MethodItem>,
    pub other_items: Vec<ImplItem>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub(crate) struct MethodItem {
    pub ident: Ident,
    pub params: Vec<ParamMeta>,
    pub ret_ty: ReturnType,
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub(crate) struct ParamMeta {
    pub ident: Ident,
    pub ty: Type,
}

pub(crate) fn parse(
    attr: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> syn::Result<ObjectImplInput> {
    // Accept zero arguments OR exactly `undocumented = "..."` as the sole key-value.
    // Any other non-empty attribute body is an error with the #[object_part] hint.
    if !attr.is_empty() {
        // Try to parse as `undocumented = "..."` — if it succeeds, accept it; otherwise
        // fall through to the error.
        let is_undocumented_kv = syn::parse2::<syn::MetaNameValue>(attr.clone())
            .ok()
            .filter(|nv| nv.path.is_ident("undocumented"))
            .is_some();
        if !is_undocumented_kv {
            return Err(syn::Error::new_spanned(
                attr,
                "`#[object_impl]` takes no arguments — use `#[object_part]` for accumulating blocks",
            ));
        }
    }
    let mut item: ItemImpl = parse2(input)?;

    let generics = item.generics.clone();
    let trait_path = item.trait_.as_ref().map(|(_, path, _)| path.clone());
    let self_ty = *item.self_ty;
    let self_ty_ident = extract_self_ty_ident(&self_ty)?;

    let mut methods = Vec::new();
    let mut other_items = Vec::new();

    for impl_item in item.items.drain(..) {
        match impl_item {
            ImplItem::Fn(mut fn_item) => {
                let is_slot = extract_attr(&mut fn_item.attrs, "slot");
                let is_invoke = extract_attr(&mut fn_item.attrs, "invoke");

                if is_slot || is_invoke {
                    let ident = fn_item.sig.ident.clone();
                    let params = extract_params(&fn_item.sig.inputs)?;
                    let ret_ty = fn_item.sig.output.clone();
                    methods.push(MethodItem {
                        ident,
                        params,
                        ret_ty,
                    });
                }
                // Re-push the (possibly slot/invoke-stripped) fn so it ends up in the impl block.
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
    })
}

/// Extracts the last path-segment ident from the impl's self type.
fn extract_self_ty_ident(self_ty: &Type) -> syn::Result<Ident> {
    let err = || {
        syn::Error::new(
            self_ty.span(),
            "#[object_impl] self type must be a simple path (e.g. `Foo` or `my_mod::Foo`)",
        )
    };
    let Type::Path(tp) = self_ty else {
        return Err(err());
    };
    tp.path
        .segments
        .last()
        .map(|s| s.ident.clone())
        .ok_or_else(err)
}

/// Extracts `(ident, type)` pairs from fn inputs, skipping the receiver.
pub(crate) fn extract_params(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
) -> syn::Result<Vec<ParamMeta>> {
    let mut params = Vec::new();
    for arg in inputs {
        match arg {
            FnArg::Receiver(_) => {}
            FnArg::Typed(pat_ty) => {
                let ident = match &*pat_ty.pat {
                    Pat::Ident(pi) => pi.ident.clone(),
                    other => {
                        return Err(syn::Error::new(
                            other.span(),
                            "#[slot]/#[invoke] method parameters must be simple named bindings",
                        ));
                    }
                };
                params.push(ParamMeta {
                    ident,
                    ty: *pat_ty.ty.clone(),
                });
            }
        }
    }
    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    fn parse_ok(ts: TokenStream) -> ObjectImplInput {
        parse(quote! {}, ts).expect("should parse successfully")
    }

    #[test]
    fn extracts_self_ty_ident() {
        let ir = parse_ok(quote! {
            impl Foo {
                fn bar(&self) {}
            }
        });
        assert_eq!(ir.self_ty_ident, "Foo");
    }

    #[test]
    fn slot_method_classified() {
        let ir = parse_ok(quote! {
            impl Foo {
                #[slot]
                fn on_click(&mut self, x: i32) {}
            }
        });
        assert_eq!(ir.methods.len(), 1);
        assert_eq!(ir.methods[0].ident, "on_click");
        assert_eq!(ir.methods[0].params.len(), 1);
        assert_eq!(ir.methods[0].params[0].ident, "x");
    }

    #[test]
    fn invokable_method_classified() {
        let ir = parse_ok(quote! {
            impl Foo {
                #[invoke]
                fn compute(&self, a: i32, b: i32) -> i32 { a + b }
            }
        });
        assert_eq!(ir.methods.len(), 1);
        assert_eq!(ir.methods[0].ident, "compute");
        assert_eq!(ir.methods[0].params.len(), 2);
    }

    #[test]
    fn non_annotated_method_goes_to_other_items() {
        let ir = parse_ok(quote! {
            impl Foo {
                fn helper(&self) -> i32 { 42 }
            }
        });
        assert_eq!(ir.methods.len(), 0);
        assert_eq!(ir.other_items.len(), 1);
    }

    #[test]
    fn annotated_method_also_in_other_items_for_reemit() {
        let ir = parse_ok(quote! {
            impl Foo {
                #[slot]
                fn do_thing(&mut self) {}
            }
        });
        // methods for codegen
        assert_eq!(ir.methods.len(), 1);
        // other_items for re-emit (stripped of #[slot])
        assert_eq!(ir.other_items.len(), 1);
    }

    #[test]
    fn undocumented_kv_in_attr_accepted() {
        // Proves that #[object_impl(undocumented = "allow")] is accepted by the parser.
        let result = parse(
            quote! { undocumented = "allow" },
            quote! { impl Foo { fn bar(&self) {} } },
        );
        assert!(
            result.is_ok(),
            "expected #[object_impl(undocumented = \"allow\")] to parse successfully, got: {:?}",
            result.unwrap_err()
        );
    }

    #[test]
    fn trait_impl_accepted() {
        let ir = parse_ok(quote! {
            impl MyTrait for Foo {
                fn foo(&self) {}
            }
        });
        assert_eq!(ir.self_ty_ident, "Foo");
        assert!(ir.trait_path.is_some(), "trait_path should be set");
    }

    #[test]
    fn trait_impl_stores_trait_path() {
        let ir = parse_ok(quote! {
            impl MyTrait for Foo {
                #[slot]
                fn on_event(&mut self) {}
            }
        });
        let trait_path = ir.trait_path.expect("trait_path should be set");
        let path_str = quote! { #trait_path }.to_string();
        assert!(path_str.contains("MyTrait"), "unexpected path: {path_str}");
    }

    #[test]
    fn inherent_impl_trait_path_is_none() {
        let ir = parse_ok(quote! {
            impl Foo { fn bar(&self) {} }
        });
        assert!(ir.trait_path.is_none());
    }

    #[test]
    fn non_empty_attr_errors_with_object_part_hint() {
        let err = parse(quote! { partial }, quote! { impl Foo {} })
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("#[object_part]"),
            "error should mention #[object_part], got: {err}"
        );
    }

    #[test]
    fn no_receiver_params_extracted() {
        let ir = parse_ok(quote! {
            impl Foo {
                #[invoke]
                fn greet(&self, name: String) -> String { name }
            }
        });
        // only `name: String`, not `&self`
        assert_eq!(ir.methods[0].params.len(), 1);
        assert_eq!(ir.methods[0].params[0].ident, "name");
    }
}
