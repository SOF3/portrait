use std::collections::HashMap;

use syn::{Error, Result};

/// Shorthand for `TraitItemMap::new().minus(ImplItemMap::new())`.
pub fn subtract_items<'t>(
    trait_items: &'t [syn::TraitItem],
    impl_block: &'t syn::ItemImpl,
) -> syn::Result<TraitItemMap<'t>> {
    let mut items = TraitItemMap::new(trait_items);
    items.minus(&ImplItemMap::new(impl_block))?;
    Ok(items)
}

/// Indexes items in a trait by namespaced identifier.
#[derive(Default)]
pub struct TraitItemMap<'t> {
    /// Associated constants in the trait.
    pub consts: HashMap<syn::Ident, &'t syn::TraitItemConst>,
    /// Associated functions in the trait.
    pub fns:    HashMap<syn::Ident, &'t syn::TraitItemFn>,
    /// Associated types in the trait.
    pub types:  HashMap<syn::Ident, &'t syn::TraitItemType>,
}

impl<'t> TraitItemMap<'t> {
    /// Constructs the trait item index from a slice of trait items.
    pub fn new(trait_items: &'t [syn::TraitItem]) -> Self {
        let mut map = Self::default();
        for item in trait_items {
            match item {
                syn::TraitItem::Const(item) => {
                    map.consts.insert(item.ident.clone(), item);
                }
                syn::TraitItem::Fn(item) => {
                    map.fns.insert(item.sig.ident.clone(), item);
                }
                syn::TraitItem::Type(item) => {
                    map.types.insert(item.ident.clone(), item);
                }
                _ => {}
            }
        }
        map
    }

    /// Removes the items found in the impl, leaving only unimplemented items.
    pub fn minus(&mut self, impl_items: &ImplItemMap) -> Result<()> {
        for (ident, impl_item) in &impl_items.consts {
            if self.consts.remove(ident).is_none() {
                return Err(Error::new_spanned(
                    impl_item,
                    "no associated constant called {ident} in trait",
                ));
            }
        }

        for (ident, impl_item) in &impl_items.fns {
            if self.fns.remove(ident).is_none() {
                return Err(Error::new_spanned(
                    impl_item,
                    "no associated function called {ident} in trait",
                ));
            }
        }

        for (ident, impl_item) in &impl_items.types {
            if self.types.remove(ident).is_none() {
                return Err(Error::new_spanned(
                    impl_item,
                    "no associated type called {ident} in trait",
                ));
            }
        }

        Ok(())
    }
}

/// Indexes items in an impl block by namespaced identifier.
#[derive(Default)]
pub struct ImplItemMap<'t> {
    /// Associated constants in the implementation.
    pub consts: HashMap<syn::Ident, &'t syn::ImplItemConst>,
    /// Associated functions in the implementation.
    pub fns:    HashMap<syn::Ident, &'t syn::ImplItemFn>,
    /// Associated types in the implementation.
    pub types:  HashMap<syn::Ident, &'t syn::ImplItemType>,
}

impl<'t> ImplItemMap<'t> {
    /// Constructs the impl item index from an impl block.
    pub fn new(impl_block: &'t syn::ItemImpl) -> Self {
        let mut map = Self::default();
        for item in &impl_block.items {
            match item {
                syn::ImplItem::Const(item) => {
                    map.consts.insert(item.ident.clone(), item);
                }
                syn::ImplItem::Fn(item) => {
                    map.fns.insert(item.sig.ident.clone(), item);
                }
                syn::ImplItem::Type(item) => {
                    map.types.insert(item.ident.clone(), item);
                }
                _ => {}
            }
        }
        map
    }
}
