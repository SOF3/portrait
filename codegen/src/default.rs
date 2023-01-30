use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Result};

pub(crate) struct GeneratorT(pub(crate) portrait_framework::NoArgs);
impl portrait_framework::Generator for GeneratorT {
    fn generate_const(&mut self, item: &syn::TraitItemConst) -> Result<syn::ImplItemConst> {
        Ok(syn::ImplItemConst {
            attrs:       item
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("cfg"))
                .cloned()
                .collect(),
            vis:         syn::Visibility::Inherited,
            defaultness: None,
            const_token: item.const_token,
            ident:       item.ident.clone(),
            colon_token: item.colon_token,
            ty:          item.ty.clone(),
            eq_token:    syn::Token![=](item.span()),
            expr:        syn::parse2(quote!(Default::default())).unwrap(),
            semi_token:  item.semi_token,
        })
    }

    fn generate_method(&mut self, item: &syn::TraitItemMethod) -> Result<syn::ImplItemMethod> {
        Ok(syn::ImplItemMethod {
            attrs:       item
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("cfg"))
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

    fn generate_type(&mut self, item: &syn::TraitItemType) -> Result<syn::ImplItemType> {
        Err(Error::new_spanned(item, "portrait::default cannot implement types automatically"))
    }
}

fn unuse_sig(mut sig: syn::Signature) -> syn::Signature {
    for input in &mut sig.inputs {
        if let syn::FnArg::Typed(typed) = input {
            typed.attrs.push(syn::Attribute {
                pound_token:   syn::Token![#](Span::call_site()),
                style:         syn::AttrStyle::Outer,
                bracket_token: syn::token::Bracket(Span::call_site()),
                path:          syn::parse2(quote!(allow)).unwrap(),
                tokens:        quote!((unused_variables)),
            });
        }
    }
    sig
}
