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

#[cfg(feature = "default-filler")]
mod default;
#[cfg(feature = "default-filler")]
#[proc_macro]
pub fn default(input: TokenStream) -> TokenStream {
    portrait_framework::completer_filler(input, default::Generator)
}

#[cfg(feature = "delegate-filler")]
mod delegate;
#[cfg(feature = "delegate-filler")]
#[proc_macro]
pub fn delegate(input: TokenStream) -> TokenStream {
    portrait_framework::completer_filler(input, delegate::Generator)
}
