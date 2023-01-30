use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::Result;
pub(crate) struct Once<T>(pub(crate) Option<(Span, T)>);

impl<T> Default for Once<T> {
    fn default() -> Self { Self(None) }
}

impl<T> Once<T> {
    pub(crate) fn set(&mut self, value: T, span: Span) -> Result<()> {
        if let Some((old_span, _)) = self.0.replace((span, value)) {
            return Err(syn::Error::new(
                Span::join(&span, old_span).unwrap_or(span),
                "Argument cannot be set twice",
            ));
        }
        Ok(())
    }

    pub(crate) fn try_get(self) -> Option<T> { self.0.map(|(_, t)| t) }

    pub(crate) fn get_or(self, f: impl FnOnce() -> T) -> T { self.try_get().unwrap_or_else(f) }
}

pub(crate) trait ParseArgs: Default {
    fn parse_once(&mut self, input: ParseStream) -> Result<()>;
}

pub(crate) struct Args<T>(pub(crate) T);
impl<T: ParseArgs> Parse for Args<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = T::default();

        while !input.is_empty() {
            args.parse_once(input)?;

            if let Err(err) = input.parse::<syn::Token![,]>() {
                if !input.is_empty() {
                    return Err(err);
                }
            }
        }

        Ok(Self(args))
    }
}
