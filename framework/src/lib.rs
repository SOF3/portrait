//! Framework for developing portrait filler macros.
//!
//! # Example

#![cfg_attr(not(debug_assertions), deny(missing_docs))]

use proc_macro2::TokenStream;
use syn::parse::Parse;
use syn::Result;

mod filler;
pub use filler::filler;

mod no_args;
pub use no_args::NoArgs;

mod trait_item_map;
pub use trait_item_map::{
    complete, completer_filler, completer_filler2, subtract_items, Generator, ImplItemMap,
    TraitItemMap,
};

/// A completer determines how to fill the implementation of a trait.
pub trait Completer {
    /// The arguments passed to the completer through macros.
    type Args: Parse;

    /// Completes the impl given a portrait of the trait items.
    fn complete(
        self,
        portrait: &[syn::TraitItem],
        args: Self::Args,
        item_impl: &syn::ItemImpl,
    ) -> Result<TokenStream>;
}
