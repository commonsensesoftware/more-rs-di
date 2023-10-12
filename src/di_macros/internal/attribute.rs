use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Path, Result, Token,
};

pub struct InjectableAttribute {
    pub trait_: Option<Path>,
    pub keyed: bool,
}

impl Parse for InjectableAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<Path, Token![,]>::parse_terminated(input)?;
        let count = args.len();

        if count > 2 {
            return Err(Error::new(
                args.span(),
                "Found {} arguments, but expected 0-2 arguments.",
            ));
        }

        let mut trait_ = None;
        let mut keyed = false;
        let name =  Ident::new("keyed", Span::call_site());

        for arg in args {
            if arg.segments.last().unwrap().ident == name {
                keyed = true
            } else {
                trait_ = Some(arg)
            }
        }

        Ok(Self { trait_, keyed })
    }
}
