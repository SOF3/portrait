//! Framework for developing portrait filler macros.
//!
//! # Example

#![cfg_attr(not(debug_assertions), deny(missing_docs))]

mod impl_filler;
pub use impl_filler::{impl_filler, FillImpl};

mod derive_filler;
pub use derive_filler::{derive_filler, FillDerive};

mod no_args;
pub use no_args::NoArgs;

mod impl_completer;
pub use impl_completer::{
    complete_impl, completer_impl_filler, completer_impl_filler2, GenerateImpl, ImplContext,
};

mod derive_completer;
pub use derive_completer::{
    complete_derive, completer_derive_filler, completer_derive_filler2, DeriveContext,
    GenerateDerive,
};

mod item_map;
pub use item_map::{subtract_items, ImplItemMap, TraitItemMap};
