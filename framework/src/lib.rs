//! Framework for developing portrait filler macros.
//!
//! # Example

#![cfg_attr(not(debug_assertions), deny(missing_docs))]

mod filler;
pub use filler::{filler, Fill};

mod no_args;
pub use no_args::NoArgs;

mod trait_item_map;
pub use trait_item_map::{
    complete, completer_filler, completer_filler2, subtract_items, Context, Generate, ImplItemMap,
    TraitItemMap,
};
