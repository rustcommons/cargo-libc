use proc_macro::TokenStream;

#[proc_macro]
pub fn message(_input: TokenStream) -> TokenStream {
    // This shared library is loaded by the host rustc, not the target runtime.
    "\"host procedural macro ran\"".parse().unwrap()
}
