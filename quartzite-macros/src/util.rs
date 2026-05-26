use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Ident;

/// Strips the `Base` suffix from `name` if it ends with `Base` (case-sensitive).
/// Returns `""` for the degenerate input `"Base"` — callers must check and emit a compile error.
pub(crate) fn strip_base_suffix(name: &str) -> &str {
    name.strip_suffix("Base").unwrap_or(name)
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
    Ident::new(&format!("__quartzite_{type_ident}"), type_ident.span())
}

/// Removes the first `#[name]` attribute from `attrs`; returns whether it was present.
pub(crate) fn extract_attr(attrs: &mut Vec<syn::Attribute>, name: &str) -> bool {
    attrs
        .iter()
        .position(|a| a.path().is_ident(name))
        .is_some_and(|i| {
            attrs.remove(i);
            true
        })
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

/// Returns the leading path fragment for all `::quartzite::widgets::*` references in generated code.
///
/// Resolution order (facade-first):
/// 1. `quartzite` facade found → `::name::widgets` (Name) or absolute path via pkg name (Itself)
/// 2. `quartzite-widgets` found → `::name` (Name) or absolute path via pkg name (Itself)
/// 3. Neither found → silent fallback to `::quartzite_widgets`
///
/// Used to qualify `WidgetView` and `WidgetChildren` in emitted code so they resolve
/// correctly both from within `quartzite-widgets` and from third-party crates.
pub(crate) fn widgets_root() -> TokenStream {
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "quartzite".into());
    let facade = crate_name("quartzite").ok();
    let widgets = if facade.is_none() {
        crate_name("quartzite-widgets").ok()
    } else {
        None
    };
    widgets_root_from(facade, widgets, &pkg_name)
}

pub(crate) fn widgets_root_from(
    facade: Option<FoundCrate>,
    widgets: Option<FoundCrate>,
    _pkg_name: &str,
) -> TokenStream {
    match facade {
        // Compiling quartzite facade itself: module `widgets` lives at `crate::widgets`.
        Some(FoundCrate::Itself) => quote!(crate::widgets),
        Some(FoundCrate::Name(n)) => {
            let ident = Ident::new(&n, Span::call_site());
            quote!(::#ident::widgets)
        }
        None => match widgets {
            // Compiling quartzite-widgets itself: types live in `crate::`.
            Some(FoundCrate::Itself) => quote!(crate),
            Some(FoundCrate::Name(n)) => {
                let ident = Ident::new(&n, Span::call_site());
                quote!(::#ident)
            }
            None => quote!(::quartzite_widgets),
        },
    }
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

macro_rules! make_expand {
    () => {
        pub(crate) fn expand(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
            match parse::parse(input) {
                Ok(ir) => codegen::codegen(&ir),
                Err(e) => e.to_compile_error(),
            }
        }
    };
}
pub(crate) use make_expand;

// ──────────────────────────────────────────────────────────────────────────
// Tri-state undocumented diagnostic helpers
// ──────────────────────────────────────────────────────────────────────────

/// Tri-state diagnostic level for the `#[undocumented]` cascade.
///
/// Mirrors rust's native lint-level vocabulary (`allow` / `warn` / `deny`).
/// Default when no scope overrides: `Warn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Level {
    /// Silent — no diagnostic emitted.
    Allow,
    /// Non-error warning (synthesised `#[deprecated]`).
    Warn,
    /// Hard compile error (`compile_error!`).
    Deny,
}

/// Parses the per-item `#[undocumented(allow|warn|deny)]` argument.
///
/// Called from inside a `parse_nested_meta` closure; `meta` is the inner path
/// (`allow`, `warn`, or `deny`). Bare `#[undocumented]` never reaches this function
/// because the outer `extract_undocumented_per_item` only calls it when the attribute
/// has a nested-meta body.
pub(crate) fn parse_undocumented_level(meta: &syn::meta::ParseNestedMeta) -> syn::Result<Level> {
    if meta.path.is_ident("allow") {
        Ok(Level::Allow)
    } else if meta.path.is_ident("warn") {
        Ok(Level::Warn)
    } else if meta.path.is_ident("deny") {
        Ok(Level::Deny)
    } else {
        Err(meta.error("expected `allow`, `warn`, or `deny`"))
    }
}

/// Parses the per-invocation `undocumented = "allow|warn|deny"` value.
///
/// # Errors
///
/// Returns an error if the string is not one of `"allow"`, `"warn"`, `"deny"`.
pub(crate) fn parse_undocumented_kv(value: &syn::LitStr) -> syn::Result<Level> {
    match value.value().as_str() {
        "allow" => Ok(Level::Allow),
        "warn" => Ok(Level::Warn),
        "deny" => Ok(Level::Deny),
        other => Err(syn::Error::new(
            value.span(),
            format!("expected `\"allow\"`, `\"warn\"`, or `\"deny\"`; got `\"{other}\"`"),
        )),
    }
}

/// Removes and parses a `#[undocumented(allow|warn|deny)]` attribute from `attrs`.
///
/// Returns `Ok(None)` if no `#[undocumented]` is present.
/// Returns `Ok(Some(level))` when the attribute is found and valid.
/// Returns `Err` when:
/// - `#[undocumented]` is bare (no argument) — ambiguous, rejected.
/// - The argument is not one of `allow`, `warn`, `deny`.
///
/// # Errors
///
/// Returns a [`syn::Error`] if the attribute is malformed.
pub(crate) fn extract_undocumented_per_item(
    attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<Option<Level>> {
    let Some(pos) = attrs.iter().position(|a| a.path().is_ident("undocumented")) else {
        return Ok(None);
    };
    let attr = attrs.remove(pos);
    let mut level: Option<Level> = None;
    attr.parse_nested_meta(|meta| {
        level = Some(parse_undocumented_level(&meta)?);
        Ok(())
    })?;
    level.map_or_else(
        || {
            Err(syn::Error::new_spanned(
                &attr,
                "`#[undocumented]` requires an argument: `allow`, `warn`, or `deny`",
            ))
        },
        |l| Ok(Some(l)),
    )
}

/// Returns the global diagnostic level from cargo features and `QUARTZITE_UNDOCUMENTED`.
///
/// Resolution order:
/// 1. `option_env!("QUARTZITE_UNDOCUMENTED")` — baked at `quartzite-macros` compile time.
///    Rebuild `quartzite-macros` to change (e.g. `cargo clean -p quartzite-macros && \
///    QUARTZITE_UNDOCUMENTED=allow cargo build`).
/// 2. Cargo feature `undocumented-allow` → `Allow`.
/// 3. Cargo feature `undocumented-deny`  → `Deny`.
/// 4. Both features set → `compile_error!` emitted; function panics in tests.
/// 5. None set → returns `None` (falls through to built-in default `Warn`).
///
/// # Panics
///
/// Panics in test contexts when both `undocumented-allow` AND `undocumented-deny` are set,
/// mirroring the `compile_error!` emission in production use.
pub(crate) fn global_undocumented_level() -> Option<Level> {
    // Env var beats features (per KD9).
    if let Some(val) = option_env!("QUARTZITE_UNDOCUMENTED") {
        return match val {
            "allow" => Some(Level::Allow),
            "warn" => Some(Level::Warn),
            "deny" => Some(Level::Deny),
            _ => None, // malformed env var — fall through to features
        };
    }
    let allow_feature = cfg!(feature = "undocumented-allow");
    let deny_feature = cfg!(feature = "undocumented-deny");
    assert!(
        !(allow_feature && deny_feature),
        "quartzite-macros: features 'undocumented-allow' and 'undocumented-deny' are mutually exclusive"
    );
    if allow_feature {
        Some(Level::Allow)
    } else if deny_feature {
        Some(Level::Deny)
    } else {
        None
    }
}

/// Resolves the effective diagnostic level from the cascade.
///
/// Cascade: per-item > per-invocation > global > built-in default (`Warn`).
/// See [`global_undocumented_level`] for the global scope resolution.
// _Simple._
pub(crate) fn resolve_undocumented_level(
    per_item: Option<Level>,
    per_invocation: Option<Level>,
) -> Level {
    per_item
        .or(per_invocation)
        .or_else(global_undocumented_level)
        .unwrap_or(Level::Warn)
}

/// Emits the undocumented-item diagnostic token-stream for a missing `///` doc.
///
/// Returns an empty `TokenStream` when `level` is `Allow`.
/// Returns a `#[deprecated]` synthesis block at `ident_span` when `level` is `Warn`.
/// Returns a `compile_error!` at `ident_span` when `level` is `Deny`.
///
/// `type_name` is the containing type ident (for the diagnostic message).
/// `item_name` is the field / method ident (for the diagnostic message and unique fn name).
/// `ident_span` is `field.ident.span()` / `method.sig.ident.span()` — load-bearing per KD1.
pub(crate) fn emit_undocumented_diagnostic(
    level: Level,
    type_name: &Ident,
    item_name: &Ident,
    ident_span: proc_macro2::Span,
) -> TokenStream {
    match level {
        Level::Allow => TokenStream::new(),
        Level::Warn => {
            // Synthesise a `#[deprecated]` const fn + immediate const-block call.
            // The call fires the `deprecated` lint (probe confirmed 2026-05-26).
            // quote_spanned! pins the span to the user's field/method ident so
            // rustc highlights the correct source location.
            // Include type_name (lowercased) in the fn ident to avoid collisions when
            // multiple structs in the same expansion share identically-named fields.
            use heck::ToSnakeCase;
            let type_name_snake = type_name.to_string().to_snake_case();
            let warn_fn_ident = Ident::new(
                &format!("__quartzite_warn_{type_name_snake}_{item_name}"),
                ident_span,
            );
            let msg = format!(
                "annotated item `{type_name}::{item_name}` lacks `///` doc; \
                 see ai-docs/doc-convention.md § Annotated items"
            );
            quote_spanned! { ident_span =>
                #[deprecated = #msg]
                const fn #warn_fn_ident() {}
                const _: () = { #warn_fn_ident(); };
            }
        }
        Level::Deny => {
            let msg = format!(
                "Annotated item `{type_name}::{item_name}` is missing `///` documentation. \
                 Opt out via `#[undocumented(allow)]` or set the lint level to warn/allow via \
                 `#[object_impl(undocumented = \"warn\")]` or feature `undocumented-allow`."
            );
            quote_spanned! { ident_span =>
                const _: () = ::core::compile_error!(#msg);
            }
        }
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

    fn wts(facade: Option<FoundCrate>, widgets: Option<FoundCrate>) -> String {
        widgets_root_from(facade, widgets, "quartzite-widgets").to_string()
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
    fn widgets_root_facade_itself() {
        assert_eq!(wts(Some(FoundCrate::Itself), None), "crate :: widgets");
    }

    #[test]
    fn widgets_root_facade_name() {
        assert_eq!(
            wts(Some(FoundCrate::Name("my_quartzite".into())), None),
            ":: my_quartzite :: widgets"
        );
    }

    #[test]
    fn widgets_root_widgets_itself() {
        assert_eq!(wts(None, Some(FoundCrate::Itself)), "crate");
    }

    #[test]
    fn widgets_root_widgets_name() {
        assert_eq!(
            wts(None, Some(FoundCrate::Name("quartzite_widgets".into()))),
            ":: quartzite_widgets"
        );
    }

    #[test]
    fn widgets_root_fallback() {
        assert_eq!(wts(None, None), ":: quartzite_widgets");
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

    // ── Cascade helper tests ──────────────────────────────────────────────

    #[test]
    fn cascade_per_item_beats_per_invocation() {
        assert_eq!(
            resolve_undocumented_level(Some(Level::Allow), Some(Level::Deny)),
            Level::Allow
        );
    }

    #[test]
    fn cascade_per_invocation_beats_global_default() {
        assert_eq!(
            resolve_undocumented_level(None, Some(Level::Deny)),
            Level::Deny
        );
    }

    #[test]
    fn cascade_default_is_warn_when_no_overrides() {
        // No env var set during cargo test; no features set → default Warn.
        assert_eq!(resolve_undocumented_level(None, None), Level::Warn);
    }

    #[test]
    fn per_item_allow_parses() {
        let mut attrs: Vec<syn::Attribute> = syn::parse_quote! { #[undocumented(allow)] };
        let level = extract_undocumented_per_item(&mut attrs).unwrap();
        assert_eq!(level, Some(Level::Allow));
        assert!(attrs.is_empty(), "attribute should be consumed");
    }

    #[test]
    fn per_item_deny_parses() {
        let mut attrs: Vec<syn::Attribute> = syn::parse_quote! { #[undocumented(deny)] };
        let level = extract_undocumented_per_item(&mut attrs).unwrap();
        assert_eq!(level, Some(Level::Deny));
    }

    #[test]
    fn per_item_warn_parses() {
        let mut attrs: Vec<syn::Attribute> = syn::parse_quote! { #[undocumented(warn)] };
        let level = extract_undocumented_per_item(&mut attrs).unwrap();
        assert_eq!(level, Some(Level::Warn));
    }

    #[test]
    fn per_item_bare_undocumented_errors() {
        // Bare #[undocumented] with no argument is ambiguous and must be rejected.
        // syn::Attribute::parse_nested_meta returns a parse error for a bare attribute,
        // and our code also errors with "requires an argument" — either error is valid.
        let mut attrs: Vec<syn::Attribute> = syn::parse_quote! { #[undocumented] };
        let result = extract_undocumented_per_item(&mut attrs);
        // Bare #[undocumented] must produce an error (either syn parse error or our check).
        assert!(result.is_err(), "bare #[undocumented] must fail");
    }

    #[test]
    fn per_item_invalid_arg_errors() {
        let mut attrs: Vec<syn::Attribute> = syn::parse_quote! { #[undocumented(invalid)] };
        let err = extract_undocumented_per_item(&mut attrs).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("allow") || msg.contains("warn") || msg.contains("deny"),
            "error should mention valid values, got: {msg}"
        );
    }

    #[test]
    fn per_item_absent_returns_none() {
        let mut attrs: Vec<syn::Attribute> = syn::parse_quote! { #[doc = "hello"] };
        let level = extract_undocumented_per_item(&mut attrs).unwrap();
        assert_eq!(level, None);
        // The doc attr should be untouched.
        assert_eq!(attrs.len(), 1);
    }

    #[test]
    fn per_invocation_kv_allow_parses() {
        let lit = syn::LitStr::new("allow", Span::call_site());
        assert_eq!(parse_undocumented_kv(&lit).unwrap(), Level::Allow);
    }

    #[test]
    fn per_invocation_kv_warn_parses() {
        let lit = syn::LitStr::new("warn", Span::call_site());
        assert_eq!(parse_undocumented_kv(&lit).unwrap(), Level::Warn);
    }

    #[test]
    fn per_invocation_kv_deny_parses() {
        let lit = syn::LitStr::new("deny", Span::call_site());
        assert_eq!(parse_undocumented_kv(&lit).unwrap(), Level::Deny);
    }

    #[test]
    fn per_invocation_kv_invalid_errors() {
        let lit = syn::LitStr::new("nope", Span::call_site());
        let err = parse_undocumented_kv(&lit).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("nope"),
            "error should mention the invalid value, got: {msg}"
        );
    }
}
