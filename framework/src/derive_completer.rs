extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Result;

use crate::{derive_filler, FillDerive};

/// One-line wrapper that declares a filler macro.
///
/// # Example
/// ```
/// # extern crate proc_macro;
/// #
/// portrait_framework::proc_macro_filler!(foo, Generator);
/// struct Generator(portrait_framework::NoArgs);
/// impl portrait_framework::Generate for Generator {
///     fn generate_const(
///         &mut self,
///         context: portrait_framework::Context,
///         item: &syn::TraitItemConst,
///     ) -> syn::Result<syn::ImplItemConst> {
///         todo!()
///     }
///     fn generate_fn(
///         &mut self,
///         context: portrait_framework::Context,
///         item: &syn::TraitItemFn,
///     ) -> syn::Result<syn::ImplItemFn> {
///         todo!()
///     }
///     fn generate_type(
///         &mut self,
///         context: portrait_framework::Context,
///         item: &syn::TraitItemType,
///     ) -> syn::Result<syn::ImplItemType> {
///         todo!()
///     }
/// }
/// ```
///
/// This declares a filler macro called `foo`,
/// where each missing item is generated by calling the corresponding funciton.
#[macro_export]
macro_rules! proc_macro_derive_filler {
    ($ident:ident, $generator:path) => {
        pub fn $ident(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            portrait_framework::completer_filler(input, $generator)
        }
    };
}

/// Shorthand from [`fn@derive_filler`] to [`complete_derive`] ([`proc_macro`] version).
pub fn completer_derive_filler<ArgsT: Parse, GeneratorT: GenerateDerive>(
    input: proc_macro::TokenStream,
    ctor: fn(ArgsT) -> GeneratorT,
) -> proc_macro::TokenStream {
    completer_derive_filler2(input.into(), ctor)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Shorthand from [`fn@derive_filler`] to [`complete_derive`] ([`proc_macro2`] version).
pub fn completer_derive_filler2<ArgsT: Parse, GeneratorT: GenerateDerive>(
    input: TokenStream,
    ctor: fn(ArgsT) -> GeneratorT,
) -> Result<TokenStream> {
    struct Filler<GenerateT, ArgsT>(fn(ArgsT) -> GenerateT);

    impl<GenerateT: GenerateDerive, ArgsT: Parse> FillDerive for Filler<GenerateT, ArgsT> {
        type Args = ArgsT;

        fn fill(
            self,
            trait_path: &syn::Path,
            portrait: &[syn::TraitItem],
            args: Self::Args,
            input: &syn::DeriveInput,
        ) -> Result<TokenStream> {
            let tokens = complete_derive(trait_path, portrait, input, self.0(args))?;
            Ok(quote!(#tokens))
        }
    }

    derive_filler(input, Filler(ctor))
}

/// Invokes the generator on each unimplemented item
/// and returns a clone of `impl_block` with the generated items.
pub fn complete_derive(
    trait_path: &syn::Path,
    trait_items: &[syn::TraitItem],
    input: &syn::DeriveInput,
    mut generator: impl GenerateDerive,
) -> syn::Result<syn::ItemImpl> {
    let ctx = DeriveContext { trait_path, all_trait_items: trait_items, input };

    let mut generics_params: Vec<_> = input.generics.params.iter().cloned().collect();
    let mut generics_where: Vec<_> = input
        .generics
        .where_clause
        .iter()
        .flat_map(|clause| clause.predicates.iter().cloned())
        .collect();
    // TODO generic trait support (this behavior may be filler-dependent)
    generator.extend_generics(
        DeriveContext { ..ctx },
        &mut generics_params,
        &mut generics_where,
    )?;

    let self_ty = syn::Type::Path({
        let input_ident = &input.ident;
        let self_generics = (!input.generics.params.is_empty()).then(|| {
            let type_param_names = input.generics.params.iter().map(|param| match param {
                syn::GenericParam::Lifetime(lt) => lt.lifetime.to_token_stream(),
                syn::GenericParam::Type(ty) => ty.ident.to_token_stream(),
                syn::GenericParam::Const(const_) => const_.ident.to_token_stream(),
            });

            quote!(<#(#type_param_names),*>)
        });

        syn::parse_quote!(#input_ident #self_generics)
    });

    let mut items = Vec::new();
    for trait_item in trait_items {
        let item = match trait_item {
            syn::TraitItem::Const(const_item) => {
                syn::ImplItem::Const(generator.generate_const(DeriveContext { ..ctx }, const_item)?)
            }
            syn::TraitItem::Fn(fn_item) => {
                syn::ImplItem::Fn(generator.generate_fn(DeriveContext { ..ctx }, fn_item)?)
            }
            syn::TraitItem::Type(type_item) => {
                syn::ImplItem::Type(generator.generate_type(DeriveContext { ..ctx }, type_item)?)
            }
            _ => continue, // assume other tokens do not generate an item
        };
        items.push(item);
    }

    let mut attrs: Vec<_> =
        input.attrs.iter().filter(|attr| attr.path().is_ident("cfg")).cloned().collect();
    generator.extend_attrs(DeriveContext { ..ctx }, &mut attrs)?;

    Ok(syn::ItemImpl {
        attrs,
        defaultness: None,
        unsafety: None, // TODO support explicit unsafe derive
        impl_token: syn::Token![impl](Span::call_site()),
        generics: syn::Generics {
            lt_token:     (!generics_params.is_empty())
                .then(|| syn::Token![<](input.generics.span())),
            gt_token:     (!generics_params.is_empty())
                .then(|| syn::Token![>](input.generics.span())),
            params:       generics_params.into_iter().collect(),
            where_clause: (!generics_where.is_empty()).then(|| syn::WhereClause {
                where_token: syn::Token![where](Span::call_site()),
                predicates:  generics_where.into_iter().collect(),
            }),
        },
        trait_: Some((None, trait_path.clone(), syn::Token![for](Span::call_site()))),
        self_ty: Box::new(self_ty),
        brace_token: syn::token::Brace::default(),
        items,
    })
}

/// Available context parameters passed to generators.
#[non_exhaustive]
pub struct DeriveContext<'t> {
    /// The path to reference the implemented trait.
    pub trait_path:      &'t syn::Path,
    /// All known trait items in the portrait.
    pub all_trait_items: &'t [syn::TraitItem],
    /// The input struct/enum/union.
    pub input:           &'t syn::DeriveInput,
}

/// Generates missing items.
pub trait GenerateDerive {
    /// Implements an associated constant.
    fn generate_const(
        &mut self,
        ctx: DeriveContext,
        item: &syn::TraitItemConst,
    ) -> Result<syn::ImplItemConst>;

    /// Implements an associated function.
    fn generate_fn(
        &mut self,
        ctx: DeriveContext,
        item: &syn::TraitItemFn,
    ) -> Result<syn::ImplItemFn>;

    /// Implements an associated type.
    fn generate_type(
        &mut self,
        ctx: DeriveContext,
        item: &syn::TraitItemType,
    ) -> Result<syn::ImplItemType>;

    /// Provides additional type bounds for the `impl` block.
    fn extend_generics(
        &mut self,
        _ctx: DeriveContext,
        _generics_params: &mut [syn::GenericParam],
        _generics_where: &mut [syn::WherePredicate],
    ) -> Result<()> {
        Ok(())
    }

    /// Provides additional attributes for the `impl` block.
    fn extend_attrs(
        &mut self,
        _ctx: DeriveContext,
        _attrs: &mut Vec<syn::Attribute>,
    ) -> Result<()> {
        Ok(())
    }
}
