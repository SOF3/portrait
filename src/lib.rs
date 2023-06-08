//! Fill impl-trait blocks with default, delegation and more.
//!
//! ## Motivation
//!
//! Rust traits support provided methods,
//! which are great for backwards compatibility and implementation coding efficiency.
//! However they come with some limitations:
//!
//! - There is no reasonable way to implement an associated function
//! if its return type is an associated type.
//! - If a trait contains many highly similar associated functions,
//! writing the defaults involves a lot of boilerplate.
//! But users can only provide one default implementation for each method
//! through procedural macros.
//!
//! With `portrait`, the default implementations are provided
//! at `impl`-level granularity instead of trait-level.
//!
//! ## Usage
//!
//! First of all, make a portrait of the trait to implement
//! with the `#[portrait::make]` attribute:
//!
//! ```rs
//! #[portrait::make]
//! trait FooBar {
//!   // ...
//! }
//! ```
//!
//! Implement the trait partially and leave the rest to the `#[portrait::fill]` attribute:
//!
//! ```rs
//! #[portrait::fill(portrait::default)]
//! impl FooBar {}
//! ```
//!
//! The `portrait::default` part is the path to the "filler macro",
//! which is the item that actually fills the `impl` block.
//! The syntax is similar to `#[derive(path::to::Macro)]`.
//!
//! If there are implementations in a different module,
//! the imports used in trait items need to be manually passed to the make attribute:
//!
//! ```rs
//! #[portrait::make(import(
//!   foo, bar::*,
//!   // same syntax as use statements
//! ))]
//! trait FooBar {...}
//! ```
//!
//! If the fill attribute fails with an error about undefined `foo_bar_portrait`,
//! import it manually together with the FooBar trait;
//! the `#[portrait::make]` attribute generates this new module
//! in the same module as the `FooBar` trait.
//!
//! ## Provided fillers
//!
//! `portrait` provides the following filler macros:
//!
//! - [`default`]:
//!   Implements each missing method and constant by delegating to [`Default::default()`]
//!   (`Default` is const-unstable and requires nightly with `#![feature(const_default_impls)]`).
//! - [`delegate`]:
//!   Proxies each missing method, constant and type
//!   to an expression (usually `self.field`) or another type implementing the same trait.
//! - [`log`]:
//!   Calls a [`format!`]-like macro with the method arguments.
//!
//! ## How this works
//!
//! Rust macros are invoked at an early stage of the compilation chain.
//! As a result, attribute macros only have access to the literal representation
//! of the item they are applied on,
//! and cross-item derivation is not directly possible.
//! Most macros evade this problem by trying to generate code
//! that works regardless of the inaccessible information,
//! e.g. the `Default` derive macro works by invoking `Default::default()`
//! without checking whether the actual field type actually implements `Default`
//! (since the compiler would do at a later stage anyway).
//!
//! Unfortunately this approach does not work in the use case of `portrait`,
//! where the attribute macro requires compile time (procedural macro runtime) access
//! to the items of the trait referenced in the `impl` block;
//! the only available information is the path to the trait
//! (which could even be renamed to a different identifier).
//!
//! `portrait` addresses this challenge by
//! asking the trait to export its information (its "portrait")
//! in the form of a token stream in advance.
//! Through the `#[portrait::make]` attribute,
//! a *declarative* macro with the same identifier is derived,
//! containing the trait items.
//! The (`#[portrait::fill]`) attribute on the `impl` block
//! then passes its inputs to the declarative macro,
//! which in turn forwards them to the actual attribute implementation
//! (e.g. `#[portrait::make]`) along with the trait items.
//!
//! Now the actual attribute has access to both the trait items and the user impl,
//! but that's not quite yet the end of story.
//! The trait items are written in the scope of the trait definition,
//! but the attribute macro output is in the scope of the impl definition.
//! The most apparent effect is that
//! imports in the trait module do not take effect on the impl output.
//! To avoid updating implementors frequently due to changes in the trait module,
//! the `#[portrait::make]` attribute also derives a module
//! that contains the imports used in the trait
//! to be automatically imported in the impl block.
//!
//! It turns out that, as of current compiler limitations,
//! private imports actually cannot be re-exported publicly
//! even though the imported type is public,
//! so it becomes impractical to scan the trait item automatically for paths to re-export
//! (prelude types also need special treatment since they are not part of the module).
//! The problem of heterogeneous scope thus becomes exposed to users inevitably:
//! either type all required re-exports manually through import,
//! or make the imports visible to a further scope.
//!
//! Another difficulty is that
//! module identifiers share the same symbol space as trait identifiers
//! (because `module::Foo` is indistinguishable from `Trait::Foo`).
//! Thus, the module containing the imports cannot be imported together with the trait,
//! and users have to manually import/export both symbols
//! unless the trait is referenced through its enclosing module.
//!
//! ## Disclaimer
//!
//! `portrait` is not the first one to use declarative macros in attributes.
//! [`macro_rules_attribute`][macro_rules_attribute] also implements a similar idea,
//! although without involving the framework of generating the `macro_rules!` part.
//!
//!   [macro_rules_attribute]: https://docs.rs/macro_rules_attribute/

use std::fmt;

/// Placeholder values when a [cfg](attr@cfg)-disabled parameter is used in [`log`].
#[doc(hidden)]
pub struct DummyDebug {
    /// The placeholder text
    pub text: &'static str,
}

impl fmt::Debug for DummyDebug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.text)
    }
}

/// Generates a dummy implementation that returns [`Default::default()`]
/// in all associated constants and functions.
///
/// # Example
/// ```
/// // Constant defaults require the `const_default_impls` feature
/// #![cfg_attr(feature = "const-default-impls", feature(const_default_impls))]
///
/// #[portrait::make]
/// trait Foo {
///     fn qux() -> u64;
///
///     #[cfg(feature = "const-default-impls")]
///     const BAR: i32;
/// }
///
/// struct Corge;
///
/// #[portrait::fill(portrait::default)]
/// impl Foo for Corge {}
///
/// assert_eq!(Corge::qux(), 0u64);
///
/// #[cfg(feature = "const-default-impls")]
/// assert_eq!(Corge::BAR, 0i32);
/// ```
#[doc(inline)]
#[cfg(feature = "default-filler")]
pub use portrait_codegen::default;
//

//
/// Generates an implementation that delegates
/// to another implementation of the same trait.
///
/// # Syntax
/// ```
/// # /*
/// #[portrait::fill(portrait::delegate($delegate_type:ty, $self_to_delegate_value:expr))]
/// # */
/// ```
///
/// Alternatively, if the trait does not contain any associated functions with a receiver:
/// ```
/// # /*
/// #[portrait::fill(portrait::delegate($delegate_type:ty))]
/// # */
/// ```
///
/// - `$delegate_type` is the type that the implementation should delegate to.
/// - `$self_to_delegate_value` is an expression that returns the value to delegate methods with a receiver to.
///   References are automatically generated by the macro if required.
///
/// # Example
/// ```
/// #[portrait::make]
/// trait Foo {
///     const BAR: i32;
///     fn qux(&mut self, i: i64) -> u32;
///     fn corge() -> u64;
///     type Grault<U>;
/// }
///
/// // This is our delegation target type.
/// struct Real<T>(T);
/// impl<T> Foo for Real<T> {
///     const BAR: i32 = 1;
///     fn qux(&mut self, i: i64) -> u32 { i as u32 }
///     fn corge() -> u64 { 3 }
///     type Grault<U> = Option<U>;
/// }
///
/// struct Wrapper<T> {
///     real: Real<T>,
/// }
///
/// #[portrait::fill(portrait::delegate(Real<T>; self.real))]
/// impl<T> Foo for Wrapper<T> {}
/// // Note: We cannot use `U` as the generic type
/// // because it is used in `type Grault<U>` in the trait definition.
///
/// assert_eq!(Wrapper::<u8>::BAR, 1);
/// assert_eq!(Wrapper::<u8> { real: Real(0) }.qux(2), 2);
/// assert_eq!(Wrapper::<u8>::corge(), 3);
/// let _: <Wrapper<u8> as Foo>::Grault<u8> = Some(1u8);
/// ```
///
/// # Debugging tips
/// If you see error E0275 or `warn(unconditional_recursion)`,
/// it is because you try to delegate to the same type.
/// Note that it does not make sense to delegate to a value of the same type
/// since all constants and types would be recursively defined (`type X = X;`),
/// and all functions would be recursively implemented (`fn x() { x() }`).
#[doc(inline)]
#[cfg(feature = "delegate-filler")]
pub use portrait_codegen::delegate;
//

//
/// Invokes a portrait macro on the applied impl block.
///
/// # Usage
/// ```
/// # /*
/// #[portrait::fill(path::to::filler)]
/// impl Trait for Type {
///     // override filled items here
/// }
/// # */
/// ```
///
/// Pass the path to the filler macro in the attribute.
/// Extra parameters can be specified in front of the path with the `@` syntax, e.g.:
///
/// ```
/// # /*
/// #[portrait::fill(@OPTION1(...) @OPTION2 path::to::filler)]
/// # */
/// ```
///
/// ### `DEBUG_PRINT_FILLER_OUTPUT`
/// > Syntax: `@DEBUG_PRINT_FILLER_OUTPUT`
///
/// Prints the output of the filler macro, used for filler macro developers to debug their code.
///
/// ### `MOD_PATH`
/// > Syntax: `@MOD_PATH(path::to::name)`
///
/// Specifies the derived module path if it is imported differently
/// or overridden with `name` in [`#[make]`].
#[doc(inline)]
pub use portrait_codegen::fill;
//

//
/// Generates an implementation that simply logs the parameters and returns `()`.
///
/// # Syntax
/// ```
/// # /*
/// #[portrait::fill(portrait::log($logger:path))]
/// # */
/// ```
///
/// You can also specify leading parameters to the macro call:
/// ```
/// # /*
/// #[portrait::fill(portrait::log($logger:path, $($args:expr),*))]
/// # */
/// ```
///
/// - `$logger` is the path to the macro for logging, e.g. `log::info` or [`println!`].
/// - `$args` are the arguments passed to the macro before the format template,
///   e.g. the log level in `log::log` or the writer in [`writeln!`].
///
/// Associated constants are not supported.
/// Associated types are always `()`
/// (we assume to be the return likely type of `$logger`).
///
/// Currently, this macro does not properly support `#[cfg]` on arguments yet.
///
/// # Example
/// ```
/// // Imports required for calling the `write!` macro
/// use std::fmt::{self, Write};
///
/// #[portrait::make]
/// trait Foo {
///     type Bar;
///     fn qux(&mut self, i: i64) -> Self::Bar;
/// }
///
/// #[derive(Default)]
/// struct Receiver {
///     buffer: String,
/// }
///
/// #[portrait::fill(portrait::log(
///     write -> fmt::Result,
///     &mut self.buffer,
/// ))]
/// impl Foo for Receiver {}
///
/// let mut recv = Receiver::default();
/// recv.qux(3);
/// assert_eq!(recv.buffer.as_str(), "qux(3)");
/// ```
#[doc(inline)]
#[cfg(feature = "log-filler")]
pub use portrait_codegen::log;
//

//
/// Creates a portrait of a trait, allowing it to be used in [`fill`].
///
/// # Parameters
/// Parameters are comma-delimited.
///
/// ## `name`
/// > Syntax: `name = $name:ident`
///
/// Sets the derived module name to `$name`.
/// Users for the [`#[fill]`] part have to import this name.
///
/// ## `import`
/// > Syntax: `import($($imports:UseTree)*)`
///
/// Import the [use trees](https://docs.rs/syn/1/syn/enum.UseTree.html) in `$imports`
/// in the scope of the `impl` block.
///
/// ## `auto_imports`
/// > Syntax: `auto_imports`
///
/// An experimental feature for detecting imports automatically.
/// Requires the imports to be `pub use` (or `pub(crate) use` if the trait is also `pub(crate)`)
/// in order to re-export from the derived module.
#[doc(inline)]
pub use portrait_codegen::make;
