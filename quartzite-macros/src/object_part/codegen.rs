use proc_macro2::TokenStream;

use crate::object_impl::codegen::emit_impl_block;
use crate::object_impl::parse::ObjectImplInput;

/// Emits only the cleaned impl block — no metadata statics or `impl Object`.
#[inline]
pub(crate) fn codegen(ir: ObjectImplInput) -> TokenStream {
    emit_impl_block(ir.trait_path.as_ref(), &ir.self_ty, &ir.other_items)
}

#[cfg(test)]
mod tests {
    use quote::quote;

    // AC1: inherent impl block → only cleaned impl block, no MetaObject, no impl Object.
    #[test]
    fn inherent_impl_emits_only_impl_block() {
        let ir = crate::object_part::parse::parse(
            quote! {},
            quote! {
                impl Foo {
                    #[slot]
                    fn reset(&mut self) {}
                }
            },
        )
        .expect("parse ok");
        let out = super::codegen(ir).to_string();
        assert!(out.contains("impl Foo"), "missing impl block: {out}");
        assert!(out.contains("fn reset"), "missing slot fn: {out}");
        assert!(!out.contains("META_Foo"), "unexpected MetaObject: {out}");
        assert!(
            !out.contains("impl :: quartzite :: core :: Object for Foo"),
            "unexpected Object impl: {out}"
        );
        assert!(
            !out.contains("__METHODS__Foo"),
            "unexpected methods static: {out}"
        );
    }

    // AC2: trait impl block → impl Trait for Type { … }, no MetaObject, no impl Object.
    #[test]
    fn trait_impl_emits_only_impl_block() {
        let ir = crate::object_part::parse::parse(
            quote! {},
            quote! {
                impl MyTrait for Foo {
                    #[slot]
                    fn on_event(&mut self) {}
                }
            },
        )
        .expect("parse ok");
        let out = super::codegen(ir).to_string();
        assert!(
            out.contains("impl MyTrait for Foo"),
            "missing trait impl header: {out}"
        );
        assert!(out.contains("fn on_event"), "missing method: {out}");
        assert!(!out.contains("META_Foo"), "unexpected MetaObject: {out}");
        assert!(
            !out.contains("impl :: quartzite :: core :: Object for Foo"),
            "unexpected Object impl: {out}"
        );
    }
}
