use proc_macro::TokenStream;

extern "C" {
    fn host_glibc_minor() -> std::ffi::c_int;
}

/// The glibc minor version of the headers the host C code was compiled with.
#[proc_macro]
pub fn host_header_minor(_: TokenStream) -> TokenStream {
    // SAFETY: a C function without arguments returning an int.
    let minor = unsafe { host_glibc_minor() };
    minor.to_string().parse().unwrap()
}
