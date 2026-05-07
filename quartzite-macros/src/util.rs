use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Ident;

/// Strips the `Base` suffix from `name` if it ends with `Base` (case-sensitive).
/// Returns `""` for the degenerate input `"Base"` — callers must check and emit a compile error.
pub(crate) fn strip_base_suffix(name: &str) -> &str {
    if let Some(stripped) = name.strip_suffix("Base") {
        stripped
    } else {
        name
    }
}

/// Derives the `As{X}` trait ident from a struct type ident.
/// Strips `Base` suffix, then prepends `"As"`.
/// Returns `None` for the degenerate case where the result would be `"As"` alone.
pub(crate) fn as_trait_name(type_ident: &Ident) -> Option<Ident> {
    let name = type_ident.to_string();
    let stripped = strip_base_suffix(&name);
    if stripped.is_empty() {
        return None;
    }
    let trait_name = format!("As{stripped}");
    Some(Ident::new(&trait_name, type_ident.span()))
}

/// Returns the accessor method name: `snake_case` of the **original** type ident (not stripped).
/// E.g. `ObjectBase` → `object_base`, `WidgetBase` → `widget_base`, `Button` → `button`.
pub(crate) fn accessor_name(type_ident: &Ident) -> Ident {
    use heck::ToSnakeCase;
    let snake = type_ident.to_string().to_snake_case();
    Ident::new(&snake, type_ident.span())
}

/// Returns the hidden module ident: `__quartzite_{TypeName}`.
pub(crate) fn hidden_mod_ident(type_ident: &Ident) -> Ident {
    Ident::new(&format!("__quartzite_{}", type_ident), type_ident.span())
}

/// Removes the first `#[name]` attribute from `attrs`; returns whether it was present.
pub(crate) fn extract_attr(attrs: &mut Vec<syn::Attribute>, name: &str) -> bool {
    if let Some(i) = attrs.iter().position(|a| a.path().is_ident(name)) {
        attrs.remove(i);
        true
    } else {
        false
    }
}

/// Returns `#[inline]` when the user struct/enum is concrete (`generics.params` empty), else `{}`.
pub(crate) fn inline_if_concrete(generics: &syn::Generics) -> TokenStream {
    if generics.params.is_empty() {
        quote! { #[inline] }
    } else {
        quote! {}
    }
}

/// Emits a `compile_error!` at the given span.
pub(crate) fn emit_compile_error(span: Span, msg: &str) -> TokenStream {
    let msg_lit = syn::LitStr::new(msg, span);
    quote_spanned!(span => compile_error!(#msg_lit);)
}

/// Returns the leading path fragment for all `::quartzite::core::*` references in generated code.
///
/// Resolution order (facade-first):
/// 1. `quartzite` facade found → `::name::core` (Name) or absolute path via pkg name (Itself)
/// 2. `quartzite-core` found → `::name` (Name) or absolute path via pkg name (Itself)
/// 3. Neither found → silent fallback to `::quartzite_core`
///
/// Always emits absolute paths — `crate::` is intentionally avoided because proc-macros run
/// for example and binary targets where `crate` refers to the binary, not the library.
pub(crate) fn crate_root() -> TokenStream {
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "quartzite".into());
    let facade = crate_name("quartzite").ok();
    let core = if facade.is_none() {
        crate_name("quartzite-core").ok()
    } else {
        None
    };
    crate_root_from(facade, core, &pkg_name)
}

pub(crate) fn crate_root_from(
    facade: Option<FoundCrate>,
    core: Option<FoundCrate>,
    pkg_name: &str,
) -> TokenStream {
    match facade {
        Some(FoundCrate::Itself) => {
            let name = pkg_name.replace('-', "_");
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident::core)
        }
        Some(FoundCrate::Name(n)) => {
            let ident = Ident::new(&n, Span::call_site());
            quote!(::#ident::core)
        }
        None => match core {
            Some(FoundCrate::Itself) => {
                let name = pkg_name.replace('-', "_");
                let ident = Ident::new(&name, Span::call_site());
                quote!(::#ident)
            }
            Some(FoundCrate::Name(n)) => {
                let ident = Ident::new(&n, Span::call_site());
                quote!(::#ident)
            }
            None => quote!(::quartzite_core),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_crate::FoundCrate;
    use proc_macro2::Span;

    fn ts(facade: Option<FoundCrate>, core: Option<FoundCrate>) -> String {
        crate_root_from(facade, core, "quartzite").to_string()
    }

    #[test]
    fn crate_root_facade_itself() {
        assert_eq!(ts(Some(FoundCrate::Itself), None), ":: quartzite :: core");
    }

    #[test]
    fn crate_root_facade_name() {
        assert_eq!(
            ts(Some(FoundCrate::Name("my_quartzite".into())), None),
            ":: my_quartzite :: core"
        );
    }

    #[test]
    fn crate_root_core_only_name() {
        assert_eq!(
            ts(None, Some(FoundCrate::Name("quartzite_core".into()))),
            ":: quartzite_core"
        );
    }

    #[test]
    fn crate_root_core_itself() {
        assert_eq!(
            crate_root_from(None, Some(FoundCrate::Itself), "quartzite-core").to_string(),
            ":: quartzite_core"
        );
    }

    #[test]
    fn crate_root_fallback() {
        assert_eq!(ts(None, None), ":: quartzite_core");
    }

    #[test]
    fn inline_if_concrete_no_params_returns_inline() {
        let generics: syn::Generics = syn::parse_quote! {};
        let out = super::inline_if_concrete(&generics).to_string();
        assert_eq!(out, "# [inline]");
    }

    #[test]
    fn inline_if_concrete_type_param_returns_empty() {
        let generics: syn::Generics = syn::parse_quote! { <T> };
        let out = super::inline_if_concrete(&generics).to_string();
        assert!(out.is_empty(), "expected empty, got: {out}");
    }

    fn ident(s: &str) -> Ident {
        Ident::new(s, Span::call_site())
    }

    #[test]
    fn strip_base_suffix_widget_base() {
        assert_eq!(strip_base_suffix("WidgetBase"), "Widget");
    }

    #[test]
    fn strip_base_suffix_object_base() {
        assert_eq!(strip_base_suffix("ObjectBase"), "Object");
    }

    #[test]
    fn strip_base_suffix_no_suffix() {
        assert_eq!(strip_base_suffix("Button"), "Button");
    }

    #[test]
    fn strip_base_suffix_ends_in_e_not_base() {
        assert_eq!(strip_base_suffix("MyDatabase"), "MyDatabase");
    }

    #[test]
    fn strip_base_suffix_degenerate() {
        assert_eq!(strip_base_suffix("Base"), "");
    }

    #[test]
    fn as_trait_name_widget_base() {
        assert_eq!(as_trait_name(&ident("WidgetBase")).unwrap(), "AsWidget");
    }

    #[test]
    fn as_trait_name_object_base() {
        assert_eq!(as_trait_name(&ident("ObjectBase")).unwrap(), "AsObject");
    }

    #[test]
    fn as_trait_name_button() {
        assert_eq!(as_trait_name(&ident("Button")).unwrap(), "AsButton");
    }

    #[test]
    fn as_trait_name_my_database() {
        assert_eq!(as_trait_name(&ident("MyDatabase")).unwrap(), "AsMyDatabase");
    }

    #[test]
    fn as_trait_name_degenerate_base() {
        assert!(as_trait_name(&ident("Base")).is_none());
    }

    #[test]
    fn accessor_name_object_base() {
        assert_eq!(accessor_name(&ident("ObjectBase")), "object_base");
    }

    #[test]
    fn accessor_name_widget_base() {
        assert_eq!(accessor_name(&ident("WidgetBase")), "widget_base");
    }

    #[test]
    fn accessor_name_button() {
        assert_eq!(accessor_name(&ident("Button")), "button");
    }

    #[test]
    fn accessor_name_foo() {
        assert_eq!(accessor_name(&ident("Foo")), "foo");
    }
}
