use std::iter;

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

use crate::util::set_sig_arg_span;

pub(crate) struct Generator(pub(crate) Arg);
impl portrait_framework::GenerateImpl for Generator {
    fn generate_const(
        &mut self,
        ctx: portrait_framework::ImplContext,
        item: &syn::TraitItemConst,
    ) -> syn::Result<syn::ImplItemConst> {
        let Arg { ty: delegate_ty, .. } = &self.0;
        let trait_path = &ctx.impl_block.trait_.as_ref().expect("checked in framework").1;
        let item_ident = &item.ident;
        let expr = syn::parse_quote!(<#delegate_ty as #trait_path>::#item_ident);
        Ok(syn::ImplItemConst {
            attrs: item.attrs.iter().filter(|attr| attr.path().is_ident("cfg")).cloned().collect(),
            vis: syn::Visibility::Inherited,
            defaultness: None,
            const_token: item.const_token,
            ident: item.ident.clone(),
            generics: item.generics.clone(),
            colon_token: item.colon_token,
            ty: item.ty.clone(),
            eq_token: syn::Token![=](item.span()),
            expr,
            semi_token: item.semi_token,
        })
    }

    fn generate_fn(
        &mut self,
        ctx: portrait_framework::ImplContext,
        item: &syn::TraitItemFn,
    ) -> syn::Result<syn::ImplItemFn> {
        let Arg { ty: delegate_ty, value: delegate_value } = &self.0;
        let trait_path = &ctx.impl_block.trait_.as_ref().expect("checked in framework").1;

        let mut sig = item.sig.clone();

        if let Some(delegate_expr) = delegate_value {
            set_sig_arg_span(&mut sig, delegate_expr.expr.span())?;
        }
        let sig_ident = sig.ident.clone();

        let args = sig
            .inputs
            .iter_mut()
            .map(|fn_arg| match fn_arg {
                syn::FnArg::Receiver(receiver) => {
                    let arg_attrs: Vec<_> = receiver
                        .attrs
                        .iter()
                        .filter(|attr| attr.path().is_ident("cfg"))
                        .cloned()
                        .collect();
                    let ref_ = if let Some((and, _lifetime)) = &receiver.reference {
                        Some(quote!(#and))
                    } else {
                        None
                    };
                    let mut_ = receiver.mutability;

                    let delegate_expr = &delegate_value
                        .as_ref()
                        .ok_or_else(|| {
                            syn::Error::new_spanned(
                                &receiver,
                                "Delegate value must be passed to implement traits with references",
                            )
                        })?
                        .expr;

                    Ok(quote! { #(#arg_attrs)* #ref_ #mut_ #delegate_expr })
                }
                syn::FnArg::Typed(typed) => {
                    let arg_attrs: Vec<_> = typed
                        .attrs
                        .iter()
                        .filter(|attr| attr.path().is_ident("cfg"))
                        .cloned()
                        .collect();
                    if let syn::Pat::Ident(pat) = &mut *typed.pat {
                        if pat.ident == "self" {
                            // Note: this syntax only works if delegate_expr returns exactly the receiver type.
                            let delegate_expr = &delegate_value
                                .as_ref()
                                .ok_or_else(|| {
                                    syn::Error::new_spanned(
                                        typed,
                                        "Delegate value must be passed to implement traits with \
                                         references",
                                    )
                                })?
                                .expr;
                            return Ok(quote! { #(#arg_attrs)* #delegate_expr });
                        } else {
                            // Suppress `mut` when passing arguments.
                            pat.mutability = None;
                        }
                    }

                    let pat = &typed.pat;
                    Ok(quote! { #(#arg_attrs)* #pat })
                }
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let inline_attr = syn::Attribute {
            pound_token:   syn::Token![#](Span::call_site()),
            style:         syn::AttrStyle::Outer,
            bracket_token: syn::token::Bracket(Span::call_site()),
            meta:          syn::Meta::Path(syn::parse_quote!(inline)),
        };

        Ok(syn::ImplItemFn {
            attrs: item
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("cfg"))
                .cloned()
                .chain(iter::once(inline_attr))
                .collect(),
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig,
            block: syn::parse_quote! {{
                <#delegate_ty as #trait_path>::#sig_ident(#(#args,)*)
            }},
        })
    }

    fn generate_type(
        &mut self,
        ctx: portrait_framework::ImplContext,
        item: &syn::TraitItemType,
    ) -> syn::Result<syn::ImplItemType> {
        let Arg { ty: delegate_ty, .. } = &self.0;
        let trait_path = &ctx.impl_block.trait_.as_ref().expect("checked in framework").1;
        let item_ident = &item.ident;
        let generics_unbound: Vec<_> = item
            .generics
            .params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Type(ty) => ty.ident.to_token_stream(),
                syn::GenericParam::Lifetime(lt) => lt.lifetime.to_token_stream(),
                syn::GenericParam::Const(const_) => const_.ident.to_token_stream(),
            })
            .collect();
        let generics_unbound =
            (!generics_unbound.is_empty()).then(|| quote!(< #(#generics_unbound),* >));
        let ty = syn::parse_quote!(<#delegate_ty as #trait_path>::#item_ident #generics_unbound);
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
    ty:    syn::Type,
    value: Option<ArgValue>,
}
struct ArgValue {
    _semi_token: syn::Token![;],
    expr:        syn::Expr,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        let value = if input.peek(syn::Token![;]) {
            let semi_token = input.parse().expect("peeked");
            let expr = input.parse()?;
            Some(ArgValue { _semi_token: semi_token, expr })
        } else {
            None
        };

        Ok(Self { ty, value })
    }
}
