use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::parse::{BaseField, ExtendInput, MixinField};
use crate::util::{accessor_name, as_trait_name, crate_root};

pub(crate) fn codegen(ir: ExtendInput) -> TokenStream {
    let mut out = TokenStream::new();

    // NOTE: The original struct is NOT re-emitted here.
    // Derive macros append to the item; re-emitting would cause duplicate definitions.
    // Helper attributes (#[root]/#[base]/#[mixin]) are inert and harmless.

    match (&ir.is_root, &ir.base_field) {
        (true, None) => {
            // Terminal root with no parent: define As{Self} trait + self-ref impl.
            out.extend(emit_root_trait_and_impl(&ir));
        }
        (true, Some(base)) => {
            // Root with parent: trait + self-ref + direct parent-chain impls.
            out.extend(emit_root_trait_and_impl(&ir));
            // Root structs always have empty generics (enforced by parse).
            out.extend(emit_parent_chain_impls(&ir.ident, base, &ir.generics));
        }
        (false, Some(base)) => {
            // Concrete type: parent delegation + AsObject delegation.
            out.extend(emit_delegation_impl(&ir.ident, base, &ir.generics));
            out.extend(emit_as_object_impl(&ir.ident, base, &ir.generics));
        }
        (false, None) => {
            // Standalone capabilities: only mixin leaf impls (validated: ≥1 mixin).
        }
    }

    // Mixin leaf impls for every #[mixin] field.
    for mixin in &ir.mixin_fields {
        out.extend(emit_mixin_impl(&ir.ident, mixin, &ir.generics));
    }

    out
}

/// For a root struct, emits:
///   - `As{Self}` trait (with supertrait if base exists)
///   - self-ref impl
fn emit_root_trait_and_impl(ir: &ExtendInput) -> TokenStream {
    let cr = crate_root();
    let self_ident = &ir.ident;
    let self_trait = match as_trait_name(self_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(self_ident),
    };
    let acc = accessor_name(self_ident);
    let acc_mut = acc_mut_ident(&acc);

    let supertrait = ir.base_field.as_ref().and_then(|b| {
        if b.ty_ident == "ObjectBase" {
            Some(quote! { : #cr::AsObject })
        } else {
            as_trait_name(&b.ty_ident).map(|parent_trait| quote! { : #parent_trait })
        }
    });

    quote! {
        pub trait #self_trait #supertrait {
            #[doc = " Returns a shared reference to this object."]
            fn #acc(&self) -> &#self_ident;
            #[doc = " Returns a mutable reference to this object."]
            fn #acc_mut(&mut self) -> &mut #self_ident;
        }

        impl #self_trait for #self_ident {
            #[inline]
            fn #acc(&self) -> &#self_ident { self }
            #[inline]
            fn #acc_mut(&mut self) -> &mut #self_ident { self }
        }
    }
}

/// Returns a copy of `generics` with all per-param bounds, defaults, and the
/// where-clause stripped — minimal-bounds policy for delegation impls.
fn bare_generics(generics: &syn::Generics) -> syn::Generics {
    let mut bare = generics.clone();
    for param in &mut bare.params {
        match param {
            syn::GenericParam::Type(tp) => {
                tp.bounds.clear();
                tp.default = None;
            }
            syn::GenericParam::Lifetime(lp) => {
                lp.bounds.clear();
            }
            syn::GenericParam::Const(cp) => {
                cp.default = None;
            }
        }
    }
    bare.where_clause = None;
    bare
}

/// Emits direct `impl As{Parent} for Self` + `impl AsObject for Self` for a root type.
/// Uses direct field access for `ObjectBase`, delegation for higher-level parents.
fn emit_parent_chain_impls(
    self_ident: &Ident,
    base: &BaseField,
    generics: &syn::Generics,
) -> TokenStream {
    let mut out = emit_as_object_impl(self_ident, base, generics);
    // For non-ObjectBase parents, also emit the intermediate delegation impl.
    if base.ty_ident != "ObjectBase" {
        out.extend(emit_delegation_impl(self_ident, base, generics));
    }
    out
}

/// Emits `impl AsObject for {Self}` — always delegates `object_base` through the base field.
/// For a direct `ObjectBase` field, accesses it directly; otherwise delegates through the accessor.
fn emit_as_object_impl(
    self_ident: &Ident,
    base: &BaseField,
    generics: &syn::Generics,
) -> TokenStream {
    let cr = crate_root();
    let base_field = &base.ident;
    let (object_base_expr, object_base_mut_expr) = if base.ty_ident == "ObjectBase" {
        (
            quote! { &self.#base_field },
            quote! { &mut self.#base_field },
        )
    } else {
        (
            quote! { self.#base_field.object_base() },
            quote! { self.#base_field.object_base_mut() },
        )
    };
    let bare = bare_generics(generics);
    let (impl_generics, ty_generics, _) = bare.split_for_impl();
    quote! {
        impl #impl_generics #cr::AsObject for #self_ident #ty_generics {
            #[inline]
            fn object_base(&self) -> &#cr::ObjectBase {
                #object_base_expr
            }
            #[inline]
            fn object_base_mut(&mut self) -> &mut #cr::ObjectBase {
                #object_base_mut_expr
            }
            #[inline]
            fn as_any(&self) -> &dyn ::core::any::Any { self }
            #[inline]
            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any { self }
        }
    }
}

/// Emits `impl As{Parent} for {Self}` via field delegation — no `as_any` (that's in `AsObject`).
fn emit_delegation_impl(
    self_ident: &Ident,
    base: &BaseField,
    generics: &syn::Generics,
) -> TokenStream {
    let parent_trait = match as_trait_name(&base.ty_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(&base.ty_ident),
    };
    // Only emit for non-ObjectBase parents (AsObject is handled by emit_as_object_impl).
    if parent_trait == "AsObject" {
        return TokenStream::new();
    }
    let parent_ty = &base.ty;
    let base_field_ident = &base.ident;
    let parent_acc = accessor_name(&base.ty_ident);
    let parent_acc_mut = acc_mut_ident(&parent_acc);
    let bare = bare_generics(generics);
    let (impl_generics, ty_generics, _) = bare.split_for_impl();

    quote! {
        impl #impl_generics #parent_trait for #self_ident #ty_generics {
            #[inline]
            fn #parent_acc(&self) -> &#parent_ty {
                self.#base_field_ident.#parent_acc()
            }
            #[inline]
            fn #parent_acc_mut(&mut self) -> &mut #parent_ty {
                self.#base_field_ident.#parent_acc_mut()
            }
        }
    }
}

/// Emits `impl As{Mixin} for {Self}` — leaf impl only.
fn emit_mixin_impl(
    self_ident: &Ident,
    mixin: &MixinField,
    generics: &syn::Generics,
) -> TokenStream {
    let mixin_trait = match as_trait_name(&mixin.ty_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(&mixin.ty_ident),
    };
    let mixin_ty = &mixin.ty;
    let mixin_field = &mixin.ident;
    let mixin_acc = accessor_name(&mixin.ty_ident);
    let mixin_acc_mut = acc_mut_ident(&mixin_acc);
    let bare = bare_generics(generics);
    let (impl_generics, ty_generics, _) = bare.split_for_impl();

    quote! {
        impl #impl_generics #mixin_trait for #self_ident #ty_generics {
            #[inline]
            fn #mixin_acc(&self) -> &#mixin_ty { &self.#mixin_field }
            #[inline]
            fn #mixin_acc_mut(&mut self) -> &mut #mixin_ty { &mut self.#mixin_field }
        }
    }
}

fn acc_mut_ident(acc: &Ident) -> Ident {
    Ident::new(&format!("{}_mut", acc), acc.span())
}

fn emit_degenerate_error(ident: &Ident) -> TokenStream {
    crate::util::emit_compile_error(
        ident.span(),
        &format!(
            "type name '{}' alone is too generic after stripping 'Base'; choose a more descriptive name",
            ident
        ),
    )
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    fn emit(ts: TokenStream) -> String {
        let ir = crate::extend::parse::parse(ts).expect("parse ok");
        super::codegen(ir).to_string()
    }

    // Case 1: #[root] with no base — emits As{Self} trait + self-ref impl, nothing else.
    #[test]
    fn root_no_base_emits_trait_and_self_impl() {
        let out = emit(quote! {
            #[root]
            struct Widget { x: i32 }
        });
        assert!(out.contains("pub trait AsWidget"), "missing trait: {out}");
        assert!(out.contains("fn widget"), "missing accessor: {out}");
        assert!(
            out.contains("impl AsWidget for Widget"),
            "missing self-impl: {out}"
        );
        assert!(!out.contains("AsObject"), "unexpected AsObject: {out}");
    }

    // Case 2: #[root] with ObjectBase base — supertrait, self-ref impl, and direct AsObject impl.
    #[test]
    fn root_with_object_base_emits_supertrait_and_as_object() {
        let out = emit(quote! {
            #[root]
            struct Widget {
                #[base]
                object_base: ObjectBase,
            }
        });
        assert!(out.contains("pub trait AsWidget"), "missing trait: {out}");
        assert!(
            out.contains("AsObject"),
            "missing AsObject supertrait: {out}"
        );
        assert!(
            out.contains("impl AsWidget for Widget"),
            "missing self-impl: {out}"
        );
        assert!(
            out.contains("AsObject for Widget"),
            "missing AsObject impl: {out}"
        );
        // Direct field access (not via accessor) for ObjectBase.
        assert!(
            out.contains("& self . object_base"),
            "expected direct field access: {out}"
        );
        assert!(
            !out.contains("AsObjectBase"),
            "unexpected AsObjectBase: {out}"
        );
    }

    // Case 3: #[root] with non-ObjectBase base — self trait + parent delegation + AsObject via chain.
    #[test]
    fn root_with_widget_base_emits_delegation_and_as_object() {
        let out = emit(quote! {
            #[root]
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(
            out.contains("pub trait AsButton"),
            "missing self trait: {out}"
        );
        assert!(
            out.contains("impl AsButton for Button"),
            "missing self-impl: {out}"
        );
        assert!(
            out.contains("impl AsWidget for Button"),
            "missing parent delegation: {out}"
        );
        assert!(
            out.contains("AsObject for Button"),
            "missing AsObject impl: {out}"
        );
        // Delegation calls through the field.
        assert!(
            out.contains("self . widget . widget"),
            "missing delegation call: {out}"
        );
        // AsObject access delegates through widget field.
        assert!(
            out.contains("self . widget . object_base"),
            "expected chained object_base: {out}"
        );
    }

    // Case 4: non-root with base — delegation + AsObject, no new trait definition.
    #[test]
    fn child_with_base_emits_delegation_and_as_object() {
        let out = emit(quote! {
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(
            !out.contains("pub trait"),
            "unexpected trait definition: {out}"
        );
        assert!(
            out.contains("impl AsWidget for Button"),
            "missing delegation: {out}"
        );
        assert!(
            out.contains("AsObject for Button"),
            "missing AsObject impl: {out}"
        );
    }

    // Case 5: mixin only — leaf impl only, no trait def, no AsObject.
    #[test]
    fn mixin_only_emits_leaf_impl() {
        let out = emit(quote! {
            struct Panel {
                #[mixin]
                layout_base: LayoutBase,
            }
        });
        assert!(
            out.contains("impl AsLayout for Panel"),
            "missing mixin impl: {out}"
        );
        assert!(!out.contains("pub trait"), "unexpected trait: {out}");
        assert!(!out.contains("AsObject"), "unexpected AsObject: {out}");
    }

    // Multiple mixin fields each get their own leaf impl.
    #[test]
    fn multiple_mixins_emit_each_impl() {
        let out = emit(quote! {
            struct Mixed {
                #[mixin]
                layout_base: LayoutBase,
                #[mixin]
                style_base: StyleBase,
            }
        });
        assert!(
            out.contains("impl AsLayout for Mixed"),
            "missing LayoutBase: {out}"
        );
        assert!(
            out.contains("impl AsStyle for Mixed"),
            "missing StyleBase: {out}"
        );
    }

    // AC9: self-ref accessor pair carries #[inline].
    #[test]
    fn self_ref_accessors_are_inline() {
        let out = emit(quote! {
            #[root]
            struct Widget { x: i32 }
        });
        assert!(
            out.contains("# [inline]"),
            "missing #[inline] on accessor: {out}"
        );
    }

    // AC9: AsObject impl methods carry #[inline].
    #[test]
    fn as_object_impl_methods_are_inline() {
        let out = emit(quote! {
            #[root]
            struct Widget {
                #[base]
                object_base: ObjectBase,
            }
        });
        let count = out.matches("# [inline]").count();
        assert!(
            count >= 4,
            "expected >=4 #[inline] (object_base x2, as_any x2), got {count}: {out}"
        );
    }

    // AC9: parent delegation methods carry #[inline].
    #[test]
    fn delegation_methods_are_inline() {
        let out = emit(quote! {
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(
            out.contains("# [inline]"),
            "missing #[inline] on delegation: {out}"
        );
    }

    // AC9: mixin leaf accessor pair carries #[inline].
    #[test]
    fn mixin_accessors_are_inline() {
        let out = emit(quote! {
            struct Panel {
                #[mixin]
                layout_base: LayoutBase,
            }
        });
        assert!(
            out.contains("# [inline]"),
            "missing #[inline] on mixin accessor: {out}"
        );
    }

    // AC7: generic non-root struct emits `impl<T> AsWidget for Foo<T>` (no where clause).
    #[test]
    fn generic_non_root_emits_impl_with_type_params() {
        let out = emit(quote! {
            struct Foo<T> {
                #[base]
                widget: Widget,
            }
        });
        assert!(
            out.contains("impl < T > AsWidget for Foo < T >"),
            "missing generic impl header: {out}"
        );
        assert!(out.contains("impl < T >"), "missing impl_generics: {out}");
        assert!(!out.contains("where"), "unexpected where clause: {out}");
    }

    // AC7: generic with lifetime param emits correct impl header.
    #[test]
    fn generic_lifetime_emits_impl_with_lifetime() {
        let out = emit(quote! {
            struct Foo<'a> {
                #[base]
                widget: Widget,
                data: &'a str,
            }
        });
        assert!(
            out.contains("impl < 'a > AsWidget for Foo < 'a >"),
            "missing lifetime impl header: {out}"
        );
        assert!(!out.contains("where"), "unexpected where clause: {out}");
    }

    // AC7: non-generic struct output unchanged (regression).
    #[test]
    fn non_generic_regression_no_angle_brackets() {
        let out = emit(quote! {
            struct Button {
                #[base]
                widget: Widget,
            }
        });
        assert!(
            out.contains("impl AsWidget for Button"),
            "missing plain impl: {out}"
        );
        // No spurious angle brackets added.
        assert!(
            !out.contains("impl < >"),
            "unexpected empty angle brackets: {out}"
        );
    }

    // Doc-convention contract (TDD lock for subtask 7): the generated
    // `pub trait As{Self}` definition is user-facing public API; per the
    // convention, every method declared inside a `pub trait` body must carry
    // a doc comment. The two accessor methods (`#acc`, `#acc_mut`) emitted
    // by `emit_root_trait_and_impl` must each have a `///`/`#[doc = "..."]`
    // attribute. Today the codegen emits no doc on them — this assertion
    // fails until subtask 7 lands.
    #[test]
    fn root_trait_methods_carry_docs() {
        let out = emit(quote! {
            #[root]
            struct Widget { x: i32 }
        });
        // `quote!` lowers `///` to `# [doc = "..."]` in the rendered token
        // stream. Use that form for the substring check — the no-signal /
        // no-base root case emits nothing else that carries docs, so any
        // `# [doc` occurrence comes from the trait-def methods.
        assert!(
            out.contains("# [doc"),
            "missing doc attribute on root-trait accessor methods: {out}"
        );
    }
}
