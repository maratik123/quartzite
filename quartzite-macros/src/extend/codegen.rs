use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::parse::{BaseField, ExtendInput, MixinField};
use crate::util::{accessor_name, as_trait_name};

pub(crate) fn codegen(ir: ExtendInput) -> TokenStream {
    let mut out = TokenStream::new();

    // Re-emit the struct (attrs already stripped of #[root]/#[base]/#[mixin]).
    out.extend(emit_struct(&ir));

    // Generate trait + impls based on the decision table.
    match (&ir.is_root, &ir.base_field) {
        (true, None) => {
            // Terminal root: define As{Self} trait + self-ref impl.
            out.extend(emit_root_trait_and_impl(&ir));
        }
        (true, Some(base)) => {
            // New hierarchy level: trait with supertrait + self-ref + blanket impl.
            out.extend(emit_root_trait_and_impl(&ir));
            out.extend(emit_blanket_impl(&ir.ident, base));
        }
        (false, Some(base)) => {
            // Concrete type: delegation impl for parent trait only.
            out.extend(emit_delegation_impl(&ir.ident, base));
        }
        (false, None) => {
            // Standalone capabilities: only mixin leaf impls (validated: ≥1 mixin).
        }
    }

    // Mixin leaf impls for every #[mixin] field.
    for mixin in &ir.mixin_fields {
        out.extend(emit_mixin_impl(&ir.ident, mixin));
    }

    out
}

/// Re-emits the original struct with helper attrs already stripped.
fn emit_struct(ir: &ExtendInput) -> TokenStream {
    let vis = &ir.vis;
    let ident = &ir.ident;
    let attrs = &ir.attrs;
    let other_fields = &ir.other_fields;

    // Collect base + mixin fields back (attrs already stripped).
    let base_iter = ir.base_field.iter().map(|b| {
        let fi = &b.ident;
        let ty = &b.ty;
        quote! { pub #fi: #ty, }
    });
    let mixin_iter = ir.mixin_fields.iter().map(|m| {
        let fi = &m.ident;
        let ty = &m.ty;
        quote! { pub #fi: #ty, }
    });
    let other_iter = other_fields.iter().map(|f| quote! { #f, });

    quote! {
        #(#attrs)*
        #vis struct #ident {
            #(#base_iter)*
            #(#mixin_iter)*
            #(#other_iter)*
        }
    }
}

/// For a root struct, emits:
///   - `As{Self}` trait (with supertrait if base exists)
///   - self-ref impl
fn emit_root_trait_and_impl(ir: &ExtendInput) -> TokenStream {
    let self_ident = &ir.ident;
    let self_trait = match as_trait_name(self_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(self_ident),
    };
    let acc = accessor_name(self_ident);
    let acc_mut = acc_mut_ident(&acc);

    let supertrait = ir.base_field.as_ref().and_then(|b| {
        as_trait_name(&b.ty_ident).map(|parent_trait| {
            // Use quartzite_core prefix if the parent is AsObject (core-defined).
            // Otherwise emit bare (user-defined hierarchy level in same crate).
            if parent_trait == "AsObject" {
                quote! { : ::quartzite_core::AsObject }
            } else {
                quote! { : #parent_trait }
            }
        })
    });

    quote! {
        pub trait #self_trait #supertrait {
            fn #acc(&self) -> &#self_ident;
            fn #acc_mut(&mut self) -> &mut #self_ident;
        }

        impl #self_trait for #self_ident {
            fn #acc(&self) -> &#self_ident { self }
            fn #acc_mut(&mut self) -> &mut #self_ident { self }
        }
    }
}

/// Emits the blanket `impl<T: As{Self}> As{Parent} for T` for transitive ancestor satisfaction.
fn emit_blanket_impl(self_ident: &Ident, base: &BaseField) -> TokenStream {
    let self_trait = match as_trait_name(self_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(self_ident),
    };
    let parent_trait = match as_trait_name(&base.ty_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(&base.ty_ident),
    };
    let parent_ty = &base.ty;
    let self_acc = accessor_name(self_ident);
    let self_acc_mut = acc_mut_ident(&self_acc);
    let parent_acc = accessor_name(&base.ty_ident);
    let parent_acc_mut = acc_mut_ident(&parent_acc);

    // AsObject is defined in quartzite_core — use full path.
    let parent_trait_path = if parent_trait == "AsObject" {
        quote! { ::quartzite_core::AsObject }
    } else {
        quote! { #parent_trait }
    };
    let parent_ty_path = if parent_trait == "AsObject" {
        quote! { ::quartzite_core::ObjectBase }
    } else {
        quote! { #parent_ty }
    };

    quote! {
        impl<T: #self_trait> #parent_trait_path for T {
            fn #parent_acc(&self) -> &#parent_ty_path {
                self.#self_acc().#parent_acc()
            }
            fn #parent_acc_mut(&mut self) -> &mut #parent_ty_path {
                self.#self_acc_mut().#parent_acc_mut()
            }
            fn as_any(&self) -> &dyn ::core::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any { self }
        }
    }
}

/// Emits direct delegation `impl As{Parent} for {Self}` for a concrete (non-root) type.
fn emit_delegation_impl(self_ident: &Ident, base: &BaseField) -> TokenStream {
    let parent_trait = match as_trait_name(&base.ty_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(&base.ty_ident),
    };
    let parent_ty = &base.ty;
    let base_field_ident = &base.ident;
    let parent_acc = accessor_name(&base.ty_ident);
    let parent_acc_mut = acc_mut_ident(&parent_acc);

    let parent_trait_path = if parent_trait == "AsObject" {
        quote! { ::quartzite_core::AsObject }
    } else {
        quote! { #parent_trait }
    };
    let parent_ty_path = if parent_trait == "AsObject" {
        quote! { ::quartzite_core::ObjectBase }
    } else {
        quote! { #parent_ty }
    };

    quote! {
        impl #parent_trait_path for #self_ident {
            fn #parent_acc(&self) -> &#parent_ty_path {
                self.#base_field_ident.#parent_acc()
            }
            fn #parent_acc_mut(&mut self) -> &mut #parent_ty_path {
                self.#base_field_ident.#parent_acc_mut()
            }
            fn as_any(&self) -> &dyn ::core::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any { self }
        }
    }
}

/// Emits `impl As{Mixin} for {Self}` — leaf impl only, no ancestor propagation.
fn emit_mixin_impl(self_ident: &Ident, mixin: &MixinField) -> TokenStream {
    let mixin_trait = match as_trait_name(&mixin.ty_ident) {
        Some(t) => t,
        None => return emit_degenerate_error(&mixin.ty_ident),
    };
    let mixin_ty = &mixin.ty;
    let mixin_field = &mixin.ident;
    let mixin_acc = accessor_name(&mixin.ty_ident);
    let mixin_acc_mut = acc_mut_ident(&mixin_acc);

    quote! {
        impl #mixin_trait for #self_ident {
            fn #mixin_acc(&self) -> &#mixin_ty { &self.#mixin_field }
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
