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

mod derive;
#[proc_macro_attribute]
pub fn derive(attr: TokenStream, item: TokenStream) -> TokenStream {
    derive::run(attr.into(), item.into()).unwrap_or_else(|err| err.into_compile_error()).into()
}

macro_rules! fillers {
    ($dir:ident $completer_filler:ident: $($names:ident = $feature:literal,)*) => {
        mod $dir {
            $(
                #[cfg(feature = $feature)]
                pub(super) mod $names;
            )*
        }

        $(
            #[cfg(feature = $feature)]
            #[proc_macro]
            pub fn $names(input: TokenStream) -> TokenStream {
                portrait_framework::$completer_filler(input, $dir::$names::Generator)
            }
        )*
    }
}

fillers! { derive_fillers completer_derive_filler:
    derive_delegate = "derive-delegate-filler",
}

fillers! { impl_fillers completer_impl_filler:
    default = "default-filler",
    delegate = "delegate-filler",
    log = "log-filler",
}
