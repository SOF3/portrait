use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Result};

pub(crate) struct Generator(pub(crate) portrait_framework::NoArgs);
impl portrait_framework::Generate for Generator {
    fn generate_const(
        &mut self,
        _: portrait_framework::Context,
        item: &syn::TraitItemConst,
    ) -> Result<syn::ImplItemConst> {
        Ok(syn::ImplItemConst {
            attrs:       item
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("cfg"))
                .cloned()
                .collect(),
            vis:         syn::Visibility::Inherited,
            defaultness: None,
            const_token: item.const_token,
            ident:       item.ident.clone(),
            generics:    item.generics.clone(),
            colon_token: item.colon_token,
            ty:          item.ty.clone(),
            eq_token:    syn::Token![=](item.span()),
            expr:        syn::parse2(quote!(Default::default())).unwrap(),
            semi_token:  item.semi_token,
        })
    }

    fn generate_fn(
        &mut self,
        _: portrait_framework::Context,
        item: &syn::TraitItemFn,
    ) -> Result<syn::ImplItemFn> {
        Ok(syn::ImplItemFn {
            attrs:       item
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("cfg"))
                .cloned()
                .collect(),
            vis:         syn::Visibility::Inherited,
            defaultness: None,
            sig:         unuse_sig(item.sig.clone()),
            block:       syn::parse2(quote! {
                { Default::default() }
            })
            .unwrap(),
        })
    }

    fn generate_type(
        &mut self,
        _: portrait_framework::Context,
        item: &syn::TraitItemType,
    ) -> Result<syn::ImplItemType> {
        Err(Error::new_spanned(
            item,
            "portrait::default cannot implement associated types automatically",
        ))
    }
}

fn unuse_sig(mut sig: syn::Signature) -> syn::Signature {
    for input in &mut sig.inputs {
        if let syn::FnArg::Typed(typed) = input {
            typed.attrs.push(syn::Attribute {
                pound_token:   syn::Token![#](Span::call_site()),
                style:         syn::AttrStyle::Outer,
                bracket_token: syn::token::Bracket(Span::call_site()),
                meta:          syn::Meta::List(syn::MetaList {
                    path:      syn::parse2(quote!(allow)).unwrap(),
                    delimiter: syn::MacroDelimiter::Paren(syn::token::Paren(typed.span())),
                    tokens:    quote!(unused_variables),
                }),
            });
        }
    }
    sig
}
