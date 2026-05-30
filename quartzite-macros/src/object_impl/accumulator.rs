use std::cell::RefCell;
use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::parse::{MethodItem, ParamMeta};
use crate::util::{Level, emit_compile_error};

/// Span-free representation stored across proc-macro invocation boundaries.
///
/// `proc_macro::Span` objects are only valid for the lifetime of one macro invocation.
/// Storing them across invocations (in a thread-local shared between `#[object_part]`
/// and `#[object_impl]`) causes use-after-free. This struct serialises method info
/// into plain strings; `from_stored` rebuilds `MethodItem` with fresh call-site spans
/// when `drain` is called from the second invocation.
struct StoredMethod {
    name: String,
    params: Vec<(String, String)>,
    ret_ty: Option<String>,
    doc_present: bool,
    per_item_level: Option<Level>,
}

fn to_stored(method: MethodItem) -> StoredMethod {
    StoredMethod {
        name: method.ident.to_string(),
        params: method
            .params
            .into_iter()
            .map(|p| {
                let ty = &p.ty;
                (p.ident.to_string(), quote! { #ty }.to_string())
            })
            .collect(),
        ret_ty: match method.ret_ty {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(quote! { #ty }.to_string()),
        },
        doc_present: method.doc_present,
        per_item_level: method.per_item_level,
    }
}

fn from_stored(stored: StoredMethod) -> MethodItem {
    let cs = Span::call_site();
    let params = stored
        .params
        .into_iter()
        .map(|(name, ty_str)| {
            let ty: syn::Type =
                syn::parse_str(&ty_str).expect("stored type string should be parseable");
            ParamMeta {
                ident: Ident::new(&name, cs),
                ty,
            }
        })
        .collect();
    let ret_ty = stored.ret_ty.map_or_else(
        || syn::ReturnType::Default,
        |ty_str| {
            let ty: syn::Type =
                syn::parse_str(&ty_str).expect("stored return type string should be parseable");
            syn::ReturnType::Type(syn::token::RArrow { spans: [cs, cs] }, Box::new(ty))
        },
    );
    MethodItem {
        ident: Ident::new(&stored.name, cs),
        params,
        ret_ty,
        doc_present: stored.doc_present,
        per_item_level: stored.per_item_level,
    }
}

thread_local! {
    static ACCUMULATOR: RefCell<HashMap<String, Vec<StoredMethod>>> =
        RefCell::new(HashMap::new());
}

fn make_key(type_name: &str) -> String {
    let pkg = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    format!("{pkg}::{type_name}")
}

/// Pushes `methods` into the accumulator for `type_name`.
///
/// Returns a `TokenStream` of `compile_error!` tokens for any duplicate method names;
/// returns an empty `TokenStream` when all methods are new.
pub(crate) fn push(type_name: &str, methods: Vec<MethodItem>) -> TokenStream {
    let key = make_key(type_name);
    ACCUMULATOR.with(|cell| {
        let mut map = cell.borrow_mut();
        let acc = map.entry(key).or_default();
        let mut errors = TokenStream::new();
        for method in methods {
            if let Some(existing) = acc.iter().find(|m| method.ident == m.name) {
                errors.extend(emit_compile_error(
                    method.ident.span(),
                    &format!(
                        "duplicate method `{}` across `#[object_part]` blocks",
                        existing.name
                    ),
                ));
            } else {
                acc.push(to_stored(method));
            }
        }
        errors
    })
}

/// Returns `true` if there are accumulated methods for `type_name` without consuming them.
#[inline]
pub(crate) fn peek(type_name: &str) -> bool {
    let key = make_key(type_name);
    ACCUMULATOR.with(|cell| cell.borrow().get(&key).is_some_and(|v| !v.is_empty()))
}

/// Drains all accumulated methods for `type_name` and returns them.
pub(crate) fn drain(type_name: &str) -> Vec<MethodItem> {
    let key = make_key(type_name);
    ACCUMULATOR.with(|cell| {
        let mut map = cell.borrow_mut();
        map.remove(&key)
            .unwrap_or_default()
            .into_iter()
            .map(from_stored)
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    fn make_method(name: &str) -> MethodItem {
        MethodItem {
            ident: Ident::new(name, Span::call_site()),
            params: vec![],
            ret_ty: syn::ReturnType::Default,
            doc_present: false,
            per_item_level: None,
        }
    }

    #[test]
    fn peek_empty_returns_false() {
        assert!(!peek("__test_peek_empty__"));
    }

    #[test]
    fn peek_after_push_returns_true() {
        let type_name = "__test_peek_after_push__";
        push(type_name, vec![make_method("foo")]);
        assert!(peek(type_name));
        drain(type_name);
    }

    #[test]
    fn peek_does_not_consume() {
        let type_name = "__test_peek_no_consume__";
        push(type_name, vec![make_method("foo")]);
        let _ = peek(type_name);
        let drained = drain(type_name);
        assert_eq!(drained.len(), 1, "peek must not consume methods");
    }

    #[test]
    fn peek_after_drain_returns_false() {
        let type_name = "__test_peek_after_drain__";
        push(type_name, vec![make_method("foo")]);
        drain(type_name);
        assert!(!peek(type_name));
    }

    #[test]
    fn push_and_drain_returns_methods() {
        let type_name = "__test_push_and_drain__";
        let errors = push(type_name, vec![make_method("foo"), make_method("bar")]);
        assert!(errors.is_empty(), "unexpected errors: {errors}");
        let drained = drain(type_name);
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].ident, "foo");
        assert_eq!(drained[1].ident, "bar");
    }

    #[test]
    fn drain_empty_returns_empty() {
        let type_name = "__test_drain_empty__";
        let result = drain(type_name);
        assert!(result.is_empty());
    }

    #[test]
    fn duplicate_method_produces_error_token() {
        let type_name = "__test_duplicate__";
        let e1 = push(type_name, vec![make_method("reset")]);
        assert!(e1.is_empty());
        let e2 = push(type_name, vec![make_method("reset")]);
        assert!(!e2.is_empty(), "expected compile_error token");
        drain(type_name);
    }

    #[test]
    fn push_accumulates_across_calls() {
        let type_name = "__test_accumulate__";
        push(type_name, vec![make_method("a")]);
        push(type_name, vec![make_method("b")]);
        let all = drain(type_name);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn drain_clears_state() {
        let type_name = "__test_drain_clears__";
        push(type_name, vec![make_method("x")]);
        drain(type_name);
        let after = drain(type_name);
        assert!(after.is_empty(), "accumulator not cleared after drain");
    }

    #[test]
    fn round_trip_preserves_typed_param() {
        let type_name = "__test_round_trip__";
        let method = MethodItem {
            ident: Ident::new("compute", Span::call_site()),
            params: vec![ParamMeta {
                ident: Ident::new("x", Span::call_site()),
                ty: syn::parse_str("i32").expect("parse i32"),
            }],
            ret_ty: {
                let ty: syn::Type = syn::parse_str("bool").expect("parse bool");
                syn::ReturnType::Type(
                    syn::token::RArrow {
                        spans: [Span::call_site(), Span::call_site()],
                    },
                    Box::new(ty),
                )
            },
            doc_present: false,
            per_item_level: None,
        };
        push(type_name, vec![method]);
        let drained = drain(type_name);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].ident, "compute");
        assert_eq!(drained[0].params.len(), 1);
        assert_eq!(drained[0].params[0].ident, "x");
        assert_matches!(drained[0].ret_ty, syn::ReturnType::Type(_, _));
    }
}
