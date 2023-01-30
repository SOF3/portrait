use syn::parse::{Parse, ParseStream};
use syn::Result;

/// No arguments accepted by the macro.
pub struct NoArgs;
impl Parse for NoArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            Ok(Self)
        } else {
            Err(input.error("No argument expected"))
        }
    }
}
