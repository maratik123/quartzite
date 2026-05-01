use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
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

/// Returns the accessor method name: snake_case of the **original** type ident (not stripped).
/// E.g. `ObjectBase` → `object_base`, `WidgetBase` → `widget_base`, `Button` → `button`.
pub(crate) fn accessor_name(type_ident: &Ident) -> Ident {
    use heck::ToSnakeCase;
    let snake = type_ident.to_string().to_snake_case();
    Ident::new(&snake, type_ident.span())
}

/// Returns the hidden module ident: `__quartzite_{TypeName}`.
#[expect(dead_code)]
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

/// Emits a `compile_error!` at the given span.
pub(crate) fn emit_compile_error(span: Span, msg: &str) -> TokenStream {
    let msg_lit = syn::LitStr::new(msg, span);
    quote_spanned!(span => compile_error!(#msg_lit);)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

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
