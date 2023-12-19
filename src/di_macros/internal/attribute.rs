use syn::{
    parse::{Parse, ParseStream},
    Path, Result, punctuated::Punctuated, token::Plus,
};

pub struct InjectableAttribute {
    pub trait_: Option<Punctuated<Path, Plus>>,
}

impl Parse for InjectableAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            trait_: input.parse_terminated(Path::parse, Plus).ok(),
        })
    }
}
