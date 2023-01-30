use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::Result;

use crate::Completer;

/// Parses the macro input directly and passes them to the completer.
///
/// Use this function if information of all implemented/unimplemented trait/impl items
/// are required at the same time.
/// If the completer just maps each unimplemented trait item to an impl item statelessly,
/// use [`completer_filler2`](crate::completer_filler2)/[`proc_macro_filler`](crate::proc_macro_filler) instead.
pub fn filler<CompleterT: Completer>(
    input: TokenStream,
    completer: CompleterT,
) -> Result<TokenStream> {
    let Input::<CompleterT::Args> { portrait, args, item_impl, debug_print } = syn::parse2(input)?;

    let output = completer.complete(&portrait, args, &item_impl)?;

    if debug_print {
        println!("{output}");
    }

    Ok(output)
}

mod kw {
    syn::custom_keyword!(TRAIT_PORTRAIT);
    syn::custom_keyword!(ARGS);
    syn::custom_keyword!(IMPL);
    syn::custom_keyword!(DEBUG_PRINT_FILLER_OUTPUT);
}

pub(crate) struct Input<ArgsT> {
    pub(crate) portrait:    Vec<syn::TraitItem>,
    pub(crate) args:        ArgsT,
    pub(crate) item_impl:   syn::ItemImpl,
    pub(crate) debug_print: bool,
}

impl<ArgsT: Parse> Parse for Input<ArgsT> {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::TRAIT_PORTRAIT>()?;

        let portrait_braced;
        syn::braced!(portrait_braced in input);
        let mut portrait = Vec::new();
        while !portrait_braced.is_empty() {
            let item_braced;
            syn::braced!(item_braced in portrait_braced);
            let item: syn::TraitItem = item_braced.parse()?;
            if !item_braced.is_empty() {
                return Err(item_braced.error("braces should only contain one trait item"));
            }
            portrait.push(item);
        }

        input.parse::<kw::ARGS>()?;
        let args_braced;
        syn::braced!(args_braced in input);
        let args: ArgsT = args_braced.parse()?;
        if !args_braced.is_empty() {
            return Err(args_braced.error("args not fully parsed"));
        }

        input.parse::<kw::IMPL>()?;
        let impl_braced;
        syn::braced!(impl_braced in input);
        let item_impl = impl_braced.parse()?;
        if !impl_braced.is_empty() {
            return Err(impl_braced.error("trailing tokens after impl block"));
        }

        input.parse::<kw::DEBUG_PRINT_FILLER_OUTPUT>()?;
        let dpfo_braced;
        syn::braced!(dpfo_braced in input);
        let dpfo: syn::LitBool = dpfo_braced.parse()?;
        if !dpfo_braced.is_empty() {
            return Err(impl_braced.error("trailing tokens after impl block"));
        }

        if !input.is_empty() {
            return Err(input.error("trailing tokens in macro input"));
        }

        Ok(Self { portrait, args, item_impl, debug_print: dpfo.value })
    }
}
