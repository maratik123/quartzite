mod parse;
mod codegen;

pub(crate) fn expand(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match parse::parse(input) {
        Ok(ir) => codegen::codegen(ir),
        Err(e) => e.to_compile_error(),
    }
}
