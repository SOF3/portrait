use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, Result};

use crate::util::set_sig_arg_span;

pub(crate) struct Generator(pub(crate) Arg);
impl portrait_framework::GenerateImpl for Generator {
    fn generate_const(
        &mut self,
        _ctx: portrait_framework::ImplContext,
        item: &syn::TraitItemConst,
    ) -> syn::Result<syn::ImplItemConst> {
        Err(Error::new_spanned(
            item,
            "portrait::log cannot implement associated constants automatically",
        ))
    }

    fn generate_fn(
        &mut self,
        _ctx: portrait_framework::ImplContext,
        item: &syn::TraitItemFn,
    ) -> syn::Result<syn::ImplItemFn> {
        let Arg { logger, args: prefix_args, .. } = &mut self.0;

        if !prefix_args.empty_or_trailing() {
            prefix_args.push_punct(syn::Token![,](Span::call_site()));
        }

        let mut sig = item.sig.clone();
        set_sig_arg_span(&mut sig, prefix_args.span())?;

        let mut fmt_args = Vec::new();

        for param in sig.inputs.iter() {
            let (attrs, pat) = match param {
                syn::FnArg::Receiver(_) => continue,
                syn::FnArg::Typed(pat_ty) => (&pat_ty.attrs, &pat_ty.pat),
            };

            let mut arg_expr = quote!(#pat);

            let cfg_attrs: Vec<_> =
                attrs.iter().filter(|attr| attr.path().is_ident("cfg")).collect();
            if !cfg_attrs.is_empty() {
                let cfg_args = cfg_attrs
                    .iter()
                    .map(|attr| attr.parse_args::<TokenStream>())
                    .collect::<Result<Vec<_>>>()?;

                arg_expr = quote! {{
                    #[cfg(all(#(#cfg_args),*))] { #arg_expr }
                    #[cfg(not(all(#(#cfg_args),*)))] { ::portrait::DummyDebug("(cfg disabled)") }
                }};
            }

            fmt_args.push(arg_expr);
        }

        let fmt_string = format!("{}({})", &sig.ident, fmt_args.iter().map(|_| "{:?}").join(", "));

        Ok(syn::ImplItemFn {
            attrs: item.attrs.iter().filter(|attr| attr.path().is_ident("cfg")).cloned().collect(),
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig,
            block: syn::parse_quote! {{
                #logger!(#prefix_args #fmt_string, #(#fmt_args),*)
            }},
        })
    }

    fn generate_type(
        &mut self,
        _ctx: portrait_framework::ImplContext,
        item: &syn::TraitItemType,
    ) -> syn::Result<syn::ImplItemType> {
        let Arg { ret_ty, .. } = &self.0;

        let ty = match ret_ty {
            Some((_, ty)) => (**ty).clone(),
            None => syn::Type::Tuple(syn::TypeTuple {
                paren_token: syn::token::Paren(item.span()),
                elems:       Punctuated::new(),
            }),
        };

        Ok(syn::ImplItemType {
            attrs: item.attrs.iter().filter(|attr| attr.path().is_ident("cfg")).cloned().collect(),
            vis: syn::Visibility::Inherited,
            defaultness: None,
            type_token: item.type_token,
            ident: item.ident.clone(),
            generics: item.generics.clone(),
            eq_token: syn::Token![=](item.span()),
            ty,
            semi_token: item.semi_token,
        })
    }
}

pub(crate) struct Arg {
    logger:       syn::Path,
    ret_ty:       Option<(syn::Token![->], Box<syn::Type>)>,
    _comma_token: Option<syn::Token![,]>,
    args:         Punctuated<syn::Expr, syn::Token![,]>,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let logger = input.parse()?;

        let ret_ty = if input.peek(syn::Token![->]) {
            let r_arrow = input.parse().expect("peeked");
            let ty = input.parse()?;
            Some((r_arrow, Box::new(ty)))
        } else {
            None
        };

        let mut comma_token: Option<syn::Token![,]> = None;
        let mut args = Punctuated::new();
        if input.peek(syn::Token![,]) {
            comma_token = Some(input.parse().expect("peeked"));
            args = Punctuated::parse_terminated(input)?;
        }

        Ok(Self { logger, ret_ty, _comma_token: comma_token, args })
    }
}
