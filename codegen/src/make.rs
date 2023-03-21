use std::collections::HashSet;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::visit::{visit_path, Visit};
use syn::{parenthesized, Result};

use crate::util;
use crate::util::{Once, ParseArgs};

pub(crate) fn run(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = syn::parse2::<syn::ItemTrait>(item)?;
    let vis = &item.vis;
    let unstripped_trait_items = item.items.clone();

    let item_ident = &item.ident;

    let util::Args(ItemArgs { debug_print, name: mod_name, imports, auto_imports }) =
        syn::parse2::<util::Args<ItemArgs>>(attr)?;
    let mod_name =
        mod_name.get_or(|| format_ident!("{}_portrait", item.ident.to_string().to_snake_case()));

    let mut imports: Vec<_> = imports.into_iter().map(ToTokens::into_token_stream).collect();
    if auto_imports.get_or(|| false) {
        let mut import_collector = ImportCollector::default();
        for trait_item in &item.items {
            import_collector.visit_trait_item(trait_item)
        }

        imports.extend(import_collector.idents.iter().map(|ident| quote!(super::super::#ident)));
    }

    let pub_export = match vis {
        syn::Visibility::Public(_) => quote! {
            #[doc(hidden)]
            #[macro_export]
        },
        _ => quote!(),
    };
    // random name required because macro may be exported despite unused
    let macro_random_name = format_ident!("portrait_items_{:x}", rand::random::<u128>());

    let import_vis = match vis {
        syn::Visibility::Inherited => quote_spanned!(vis.span() => pub(in super::super)),
        syn::Visibility::Public(_) => quote!(#vis),
        syn::Visibility::Restricted(restricted) => {
            match restricted.in_token {
                None if restricted.path.is_ident("self") => {
                    quote_spanned!(vis.span() => pub(in super::super))
                }
                None if restricted.path.is_ident("super") => {
                    quote_spanned!(vis.span() => pub(in super::super::super))
                }
                None if restricted.path.is_ident("crate") => quote!(#vis),
                None => return Err(syn::Error::new_spanned(vis, "invalid visibility scope")),
                Some(_) => quote!(#vis), // absolute path
            }
        }
    };

    let output = quote! {
        #item

        #pub_export
        macro_rules! #macro_random_name {
            (@TARGET {$target_macro:path} @ARGS {$($args:tt)*} @IMPL {$($impl:tt)*} @DEBUG_PRINT_FILLER_OUTPUT {$debug_print:literal}) => {
                $target_macro! {
                    TRAIT_PORTRAIT { #({#unstripped_trait_items})* }
                    ARGS { $($args)* }
                    IMPL { $($impl)* }
                    DEBUG_PRINT_FILLER_OUTPUT { $debug_print }
                }
            }
        }

        #[allow(non_snake_case)]
        #vis use #macro_random_name as #item_ident;

        #[allow(non_snake_case)]
        #vis mod #mod_name {
            pub mod imports {
                #(#import_vis use #imports;)*
            }
        }
    };
    if debug_print.get_or(|| false) {
        println!("{output}");
    }
    Ok(output)
}

#[derive(Default)]
struct ItemArgs {
    debug_print:  Once<bool>,
    name:         Once<syn::Ident>,
    imports:      Vec<syn::UseTree>,
    auto_imports: Once<bool>,
}

mod kw {
    syn::custom_keyword!(__debug_print);
    syn::custom_keyword!(name);
    syn::custom_keyword!(import);
    syn::custom_keyword!(auto_imports);
}

impl ParseArgs for ItemArgs {
    fn parse_once(&mut self, input: ParseStream) -> Result<()> {
        let lh = input.lookahead1();
        if lh.peek(kw::__debug_print) {
            let key = input.parse::<kw::__debug_print>()?;
            self.debug_print.set(true, key.span())?;
        } else if lh.peek(kw::name) {
            let key = input.parse::<kw::name>()?;
            _ = input.parse::<syn::Token![=]>()?;
            self.name.set(input.parse()?, key.span())?;
        } else if lh.peek(kw::import) {
            _ = input.parse::<kw::import>()?;
            let inner;
            _ = parenthesized!(inner in input);
            let imports = inner.parse_terminated(syn::UseTree::parse, syn::Token![,])?;
            self.imports.extend(imports);
        } else if lh.peek(kw::auto_imports) {
            let key = input.parse::<kw::auto_imports>()?;
            self.debug_print.set(true, key.span())?;
        } else {
            return Err(lh.error());
        }
        Ok(())
    }
}

#[derive(Default)]
struct ImportCollector {
    idents: HashSet<syn::Ident>,
}

impl<'ast> Visit<'ast> for ImportCollector {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none() {
            let segment = path.segments.first().expect("path segments should be nonempty");
            self.idents.insert(segment.ident.clone());
        }

        visit_path(self, path)
    }
}
