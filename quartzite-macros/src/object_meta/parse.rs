use syn::{Ident, ItemImpl, Type, parse2, spanned::Spanned};

/// Parsed input for `#[object_meta]` — applied to an empty `impl Counter {}` block.
#[cfg_attr(test, derive(Debug))]
pub(crate) struct ObjectMetaInput {
    pub self_ty: Type,
    pub self_ty_ident: Ident,
}

pub(crate) fn parse(input: proc_macro2::TokenStream) -> syn::Result<ObjectMetaInput> {
    let item: ItemImpl = parse2(input)?;

    if item.trait_.is_some() {
        return Err(syn::Error::new(
            item.self_ty.span(),
            "#[object_meta] must be applied to an inherent impl block, not a trait impl",
        ));
    }

    let self_ty = *item.self_ty;
    let self_ty_ident = extract_self_ty_ident(&self_ty)?;

    Ok(ObjectMetaInput {
        self_ty,
        self_ty_ident,
    })
}

fn extract_self_ty_ident(self_ty: &Type) -> syn::Result<Ident> {
    let err = || {
        syn::Error::new(
            self_ty.span(),
            "#[object_meta] self type must be a simple path (e.g. `Counter`)",
        )
    };
    let tp = match self_ty {
        Type::Path(tp) => tp,
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
    use quote::quote;

    #[test]
    fn parses_inherent_impl() {
        let ir = parse(quote! { impl Counter {} }).expect("parse ok");
        assert_eq!(ir.self_ty_ident, "Counter");
    }

    #[test]
    fn rejects_trait_impl() {
        let err = parse(quote! { impl MyTrait for Counter {} })
            .unwrap_err()
            .to_string();
        assert!(err.contains("inherent"), "unexpected: {err}");
    }

    #[test]
    fn parses_qualified_path() {
        let ir = parse(quote! { impl my_mod::Counter {} }).expect("parse ok");
        assert_eq!(ir.self_ty_ident, "Counter");
    }
}
