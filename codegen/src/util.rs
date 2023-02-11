use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
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

pub(crate) fn set_sig_arg_span(sig: &mut syn::Signature, span: Span) -> Result<()> {
    for input in &mut sig.inputs {
        match input {
            syn::FnArg::Receiver(receiver) => {
                receiver.self_token.span = span;
            }
            syn::FnArg::Typed(pat_ty) => {
                let pat = &mut *pat_ty.pat;
                *pat = copy_with_span(pat, span)?;
            }
        }
    }

    Ok(())
}

fn copy_with_span<T: ToTokens + Parse>(t: &T, span: Span) -> Result<T> {
    let mut ts = t.to_token_stream();
    ts = copy_ts_with_span(ts, span);
    syn::parse2(ts)
}

fn copy_ts_with_span(ts: TokenStream, span: Span) -> TokenStream {
    ts.into_iter()
        .map(|mut tt| {
            if let TokenTree::Group(group) = tt {
                let group_ts = copy_ts_with_span(group.stream(), span);
                tt = TokenTree::Group(proc_macro2::Group::new(group.delimiter(), group_ts));
            }
            tt.set_span(span);
            tt
        })
        .collect()
}
