use std::cell::RefCell;
use std::collections::HashMap;

use proc_macro2::TokenStream;

use super::parse::MethodItem;
use crate::util::emit_compile_error;

thread_local! {
    static ACCUMULATOR: RefCell<HashMap<String, Vec<MethodItem>>> = RefCell::new(HashMap::new());
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
            if let Some(existing) = acc.iter().find(|m| m.ident == method.ident) {
                errors.extend(emit_compile_error(
                    method.ident.span(),
                    &format!(
                        "duplicate method `{}` across `#[object_impl(partial)]` blocks",
                        existing.ident
                    ),
                ));
            } else {
                acc.push(method);
            }
        }
        errors
    })
}

/// Drains all accumulated methods for `type_name` and returns them.
pub(crate) fn drain(type_name: &str) -> Vec<MethodItem> {
    let key = make_key(type_name);
    ACCUMULATOR.with(|cell| {
        let mut map = cell.borrow_mut();
        map.remove(&key).unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::Ident;

    fn make_method(name: &str) -> MethodItem {
        MethodItem {
            ident: Ident::new(name, Span::call_site()),
            params: vec![],
            ret_ty: syn::ReturnType::Default,
        }
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
        // Push first batch
        let e1 = push(type_name, vec![make_method("reset")]);
        assert!(e1.is_empty());
        // Push duplicate
        let e2 = push(type_name, vec![make_method("reset")]);
        assert!(!e2.is_empty(), "expected compile_error token");
        // Clean up to avoid leaking into other tests
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
}
