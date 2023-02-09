# portrait

[![GitHub actions](https://github.com/SOF3/portrait/workflows/CI/badge.svg)](https://github.com/SOF3/portrait/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/portrait.svg)](https://crates.io/crates/portrait)
[![crates.io](https://img.shields.io/crates/d/portrait.svg)](https://crates.io/crates/portrait)
[![docs.rs](https://docs.rs/portrait/badge.svg)](https://docs.rs/portrait)
[![GitHub](https://img.shields.io/github/last-commit/SOF3/portrait)](https://github.com/SOF3/portrait)
[![GitHub](https://img.shields.io/github/stars/SOF3/portrait?style=social)](https://github.com/SOF3/portrait)

Fill impl-trait blocks with default, delegation and more.

## Motivation

Rust traits support provided methods,
which are great for backwards compatibility and implementation coding efficiency.
However they come with some limitations:

- There is no reasonable way to implement an associated function
  if its return type is an associated type.
- If a trait contains many highly similar associated functions,
  writing the defaults involves a lot of boilerplate.
  But users can only provide one default implementation for each method
  through procedural macros.

With `portrait`, the default implementations are provided
at `impl`-level granularity instead of trait-level.

## Usage

First of all, make a portrait of the trait to implement
with the `#[portrait::make]` attribute:

```rs
#[portrait::make]
trait FooBar {
  // ...
}
```

Implement the trait partially and leave the rest to the `#[portrait::fill]` attribute:

```rs
#[portrait::fill(portrait::default)]
impl FooBar {}
```

The `portrait::default` part is the path to the "filler macro",
which is the item that actually fills the `impl` block.
The syntax is similar to `#[derive(path::to::Macro)]`.

If there are implementations in a different module,
the imports used in trait items need to be manually passed to the make attribute:

```rs
#[portrait::make(import(
  foo, bar::*,
  // same syntax as use statements
))]
trait FooBar {...}
```

If the fill attribute fails with an error about undefined `foo_bar_portrait`,
import it manually together with the FooBar trait;
the `#[portrait::make]` module generates it in the same module as the `FooBar` trait.

## Provided fillers

`portrait` provides the following filler macros:

- `default`:
  Implements each missing method and constant by delegating to `Default::default()`
  (`Default` is const-unstable and requires nightly with `#![feature(const_default_impls)]`).
- `delegate`:
  Proxies each missing method, constant and type
  to an expression (usually `self.field`) or another type implementing the same trait.

## How this works

Rust macros are invoked at an early stage of the compilation chain.
As a result, attribute macros only have access to the literal representation
of the item they are applied on,
and cross-item derivation is not directly possible.
Most macros evade this problem by trying to generate code
that works regardless of the inaccessible information,
e.g. the `Default` derive macro works by invoking `Default::default()`
without checking whether the actual field type actually implements `Default`
(since the compiler would do at a later stage anyway).

Unfortunately this approach does not work in the use case of `portrait`,
where the attribute macro requires compile time (procedural macro runtime) access
to the items of the trait referenced in the `impl` block;
the only available information is the path to the trait
(which could even be renamed to a different identifier).

`portrait` addresses this challenge by
asking the trait to export its information (its "portrait")
in the form of a token stream in advance.
Through the `#[portrait::make]` attribute,
a *declarative* macro with the same identifier is derived,
containing the trait items.
The (`#[portrait::fill]`) attribute on the `impl` block
then passes its inputs to the declarative macro,
which in turn forwards them to the actual attribute implementation
(e.g. `#[portrait::make]`) along with the trait items.

Now the actual attribute has access to both the trait items and the user impl,
but that's not quite yet the end of story.
The trait items are written in the scope of the trait definition,
but the attribute macro output is in the scope of the impl definition.
The most apparent effect is that
imports in the trait module do not take effect on the impl output.
To avoid updating implementors frequently due to changes in the trait module,
the `#[portrait::make]` attribute also derives a module
that contains the imports used in the trait
to be automatically imported in the impl block.

It turns out that, as of current compiler limitations,
private imports actually cannot be re-exported publicly
even though the imported type is public,
so it becomes impractical to scan the trait item automatically for paths to re-export
(prelude types also need special treatment since they are not part of the module).
The problem of heterogeneous scope thus becomes exposed to users inevitably:
either type all required re-exports manually through import,
or make the imports visible to a further scope.

Another difficulty is that
module identifiers share the same symbol space as trait identifiers
(because `module::Foo` is indistinguishable from `Trait::Foo`).
Thus, the module containing the imports cannot be imported together with the trait,
and users have to manually import/export both symbols
unless the trait is referenced through its enclosing module.

## Disclaimer

`portrait` is not the first one to use declarative macros in attributes.
[`macro_rules_attribute`][macro_rules_attribute] also implements a similar idea,
although without involving the framework of generating the `macro_rules!` part.

  [macro_rules_attribute]: https://docs.rs/macro_rules_attribute/
