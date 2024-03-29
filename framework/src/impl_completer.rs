extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::Result;

use crate::{impl_filler, subtract_items, FillImpl};

/// One-line wrapper that declares a filler macro.
///
/// # Example
/// ```
/// # extern crate proc_macro;
/// #
/// portrait_framework::proc_macro_impl_filler!(foo, Generator);
/// struct Generator(portrait_framework::NoArgs);
/// impl portrait_framework::GenerateImpl for Generator {
///     fn generate_const(
///         &mut self,
///         context: portrait_framework::ImplContext,
///         item: &syn::TraitItemConst,
///     ) -> syn::Result<syn::ImplItemConst> {
///         todo!()
///     }
///     fn generate_fn(
///         &mut self,
///         context: portrait_framework::ImplContext,
///         item: &syn::TraitItemFn,
///     ) -> syn::Result<syn::ImplItemFn> {
///         todo!()
///     }
///     fn generate_type(
///         &mut self,
///         context: portrait_framework::ImplContext,
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
macro_rules! proc_macro_impl_filler {
    ($ident:ident, $generator:path) => {
        pub fn $ident(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
            portrait_framework::completer_impl_filler(input, $generator)
        }
    };
}

/// Shorthand from [`fn@impl_filler`] to [`complete_impl`] ([`proc_macro`] version).
pub fn completer_impl_filler<ArgsT: Parse, GeneratorT: GenerateImpl>(
    input: proc_macro::TokenStream,
    ctor: fn(ArgsT) -> GeneratorT,
) -> proc_macro::TokenStream {
    completer_impl_filler2(input.into(), ctor).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Shorthand from [`fn@impl_filler`] to [`complete_impl`] ([`proc_macro2`] version).
pub fn completer_impl_filler2<ArgsT: Parse, GeneratorT: GenerateImpl>(
    input: TokenStream,
    ctor: fn(ArgsT) -> GeneratorT,
) -> Result<TokenStream> {
    struct Filler<GenerateT, ArgsT>(fn(ArgsT) -> GenerateT);

    impl<GenerateT: GenerateImpl, ArgsT: Parse> FillImpl for Filler<GenerateT, ArgsT> {
        type Args = ArgsT;

        fn fill(
            self,
            portrait: &[syn::TraitItem],
            args: Self::Args,
            item_impl: &syn::ItemImpl,
        ) -> Result<TokenStream> {
            let tokens = complete_impl(portrait, item_impl, self.0(args))?;
            Ok(quote!(#tokens))
        }
    }

    impl_filler(input, Filler(ctor))
}

/// Invokes the generator on each unimplemented item
/// and returns a clone of `impl_block` with the generated items.
pub fn complete_impl(
    trait_items: &[syn::TraitItem],
    impl_block: &syn::ItemImpl,
    mut generator: impl GenerateImpl,
) -> syn::Result<syn::ItemImpl> {
    let mut output = impl_block.clone();

    let ctx = ImplContext { all_trait_items: trait_items, impl_block };

    let items = subtract_items(trait_items, impl_block)?;
    for trait_item in items.consts.values() {
        let impl_item = generator.generate_const(ImplContext { ..ctx }, trait_item)?;
        output.items.push(syn::ImplItem::Const(impl_item));
    }
    for trait_item in items.fns.values() {
        let impl_item = generator.generate_fn(ImplContext { ..ctx }, trait_item)?;
        output.items.push(syn::ImplItem::Fn(impl_item));
    }
    for trait_item in items.types.values() {
        let impl_item = generator.generate_type(ImplContext { ..ctx }, trait_item)?;
        output.items.push(syn::ImplItem::Type(impl_item));
    }

    Ok(output)
}

/// Available context parameters passed to generators.
#[non_exhaustive]
pub struct ImplContext<'t> {
    /// All known trait items in the portrait.
    pub all_trait_items: &'t [syn::TraitItem],
    /// The input impl block.
    pub impl_block:      &'t syn::ItemImpl,
}

/// Generates missing items.
pub trait GenerateImpl {
    /// Implements an associated constant.
    fn generate_const(
        &mut self,
        ctx: ImplContext,
        item: &syn::TraitItemConst,
    ) -> Result<syn::ImplItemConst>;

    /// Implements an associated function.
    fn generate_fn(&mut self, ctx: ImplContext, item: &syn::TraitItemFn)
        -> Result<syn::ImplItemFn>;

    /// Implements an associated type.
    fn generate_type(
        &mut self,
        ctx: ImplContext,
        item: &syn::TraitItemType,
    ) -> Result<syn::ImplItemType>;
}
