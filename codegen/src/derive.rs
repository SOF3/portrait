use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::Result;

use crate::util;

pub(crate) fn run(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let Attr {
        debug_print,
        debug_print_filler_output,
        mod_path,
        trait_path,
        attr_path,
        args: attr_args,
    } = syn::parse2(attr)?;

    let item: syn::DeriveInput = syn::parse2(item)?;

    let mod_path = mod_path.unwrap_or_else(|| {
        // deduce the path to the portrait imports module based on the trait path
        let mut mod_path = trait_path.clone();
        let mod_name = mod_path.segments.last_mut().expect("path segments should be nonempty");
        mod_name.ident = format_ident!("{}_portrait", mod_name.ident.to_string().to_snake_case());
        mod_name.arguments = syn::PathArguments::None;
        mod_path
    });

    let item_stripped = util::strip_attr("portrait", &item, syn::visit_mut::visit_derive_input_mut);

    let output = quote! {
        #item_stripped

        const _: () = {
            use #mod_path::imports::*;

            #trait_path! {
                @TARGET {#attr_path}
                @TRAIT_PATH {#trait_path}
                @ARGS {#attr_args}
                @INPUT {#item}
                @DEBUG_PRINT_FILLER_OUTPUT {#debug_print_filler_output}
            }
        };
    };

    if debug_print {
        println!("{output}");
    }

    Ok(output)
}

mod kw {
    syn::custom_keyword!(MOD_PATH);
    syn::custom_keyword!(__DEBUG_PRINT);
    syn::custom_keyword!(DEBUG_PRINT_FILLER_OUTPUT);
    syn::custom_keyword!(with);
}

struct Attr {
    debug_print:               bool,
    debug_print_filler_output: bool,
    mod_path:                  Option<syn::Path>,
    trait_path:                syn::Path,
    attr_path:                 syn::Path,
    args:                      Option<TokenStream>,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut debug_print = false;
        let mut debug_print_filler_output = false;
        let mut mod_path = None;

        while input.peek(syn::Token![@]) {
            input.parse::<syn::Token![@]>().expect("peek result");

            let lh = input.lookahead1();
            if lh.peek(kw::__DEBUG_PRINT) {
                input.parse::<kw::__DEBUG_PRINT>().expect("peek result");

                debug_print = true;
            } else if lh.peek(kw::DEBUG_PRINT_FILLER_OUTPUT) {
                input.parse::<kw::DEBUG_PRINT_FILLER_OUTPUT>().expect("peek result");

                debug_print_filler_output = true;
            } else if lh.peek(kw::MOD_PATH) {
                input.parse::<kw::MOD_PATH>().expect("peek result");

                let inner;
                syn::parenthesized!(inner in input);
                mod_path = Some(inner.parse()?);
            } else {
                return Err(lh.error());
            }
        }

        let trait_path = input.parse()?;
        let _for_token: kw::with = input.parse()?;
        let attr_path = input.parse()?;

        let mut args = None;
        if !input.is_empty() {
            let inner;
            syn::parenthesized!(inner in input);
            args = Some(inner.parse()?);
        }

        Ok(Self { debug_print, debug_print_filler_output, mod_path, trait_path, attr_path, args })
    }
}
