use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::Result;

/// Determines how to derive an impl.
pub trait FillDerive {
    /// The arguments passed to the filler through macros.
    type Args: Parse;

    /// Derives the impl given a portrait of the trait items and the derived item.
    fn fill(
        self,
        trait_path: &syn::Path,
        portrait: &[syn::TraitItem],
        args: Self::Args,
        input: &syn::DeriveInput,
    ) -> Result<TokenStream>;
}

/// Parses the macro input directly and passes them to the filler.
///
/// Use this function if information of all implemented/unimplemented trait/impl items
/// are required at the same time.
/// If the filler just maps each unimplemented trait item to an impl item statelessly,
/// use [`completer_derive_filler2`](crate::completer_derive_filler2)/[`proc_macro_derive_filler`](crate::proc_macro_derive_filler) for shorthand.
pub fn derive_filler<FillerT: FillDerive>(
    input: TokenStream,
    filler: FillerT,
) -> Result<TokenStream> {
    let Input::<FillerT::Args> { trait_path, portrait, args, input, debug_print } =
        syn::parse2(input)?;

    let output = filler.fill(&trait_path, &portrait, args, &input)?;

    if debug_print {
        println!("{output}");
    }

    Ok(output)
}

mod kw {
    syn::custom_keyword!(TRAIT_PORTRAIT);
    syn::custom_keyword!(TRAIT_PATH);
    syn::custom_keyword!(ARGS);
    syn::custom_keyword!(INPUT);
    syn::custom_keyword!(DEBUG_PRINT_FILLER_OUTPUT);
}

pub(crate) struct Input<ArgsT> {
    pub(crate) trait_path:  syn::Path,
    pub(crate) portrait:    Vec<syn::TraitItem>,
    pub(crate) args:        ArgsT,
    pub(crate) input:       syn::DeriveInput,
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

        input.parse::<kw::TRAIT_PATH>()?;
        let trait_path_braced;
        syn::braced!(trait_path_braced in input);
        let trait_path: syn::Path = trait_path_braced.parse()?;
        if !trait_path_braced.is_empty() {
            return Err(trait_path_braced.error("trait path not fully parsed"));
        }

        input.parse::<kw::ARGS>()?;
        let args_braced;
        syn::braced!(args_braced in input);
        let args: ArgsT = args_braced.parse()?;
        if !args_braced.is_empty() {
            return Err(args_braced.error("args not fully parsed"));
        }

        input.parse::<kw::INPUT>()?;
        let input_braced;
        syn::braced!(input_braced in input);
        let derive_input = input_braced.parse()?;
        if !input_braced.is_empty() {
            return Err(input_braced.error("trailing tokens after input block"));
        }

        input.parse::<kw::DEBUG_PRINT_FILLER_OUTPUT>()?;
        let dpfo_braced;
        syn::braced!(dpfo_braced in input);
        let dpfo: syn::LitBool = dpfo_braced.parse()?;
        if !dpfo_braced.is_empty() {
            return Err(input_braced.error("trailing tokens after input block"));
        }

        if !input.is_empty() {
            return Err(input.error("trailing tokens in macro input"));
        }

        Ok(Self { trait_path, portrait, args, input: derive_input, debug_print: dpfo.value })
    }
}
