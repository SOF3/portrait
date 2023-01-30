use proc_macro::TokenStream;

mod util;

mod make;
#[proc_macro_attribute]
pub fn make(attr: TokenStream, item: TokenStream) -> TokenStream {
    make::run(attr.into(), item.into()).unwrap_or_else(|err| err.into_compile_error()).into()
}

mod fill;
#[proc_macro_attribute]
pub fn fill(attr: TokenStream, item: TokenStream) -> TokenStream {
    fill::run(attr.into(), item.into()).unwrap_or_else(|err| err.into_compile_error()).into()
}

mod default;
#[proc_macro]
pub fn default(input: TokenStream) -> TokenStream {
    portrait_framework::completer_filler(input, default::GeneratorT)
}
