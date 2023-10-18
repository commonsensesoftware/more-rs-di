use syn::{
    parse::{Parse, ParseStream},
    Path, Result,
};

pub struct InjectableAttribute {
    pub trait_: Option<Path>,
}

impl Parse for InjectableAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            trait_: input.parse().ok(),
        })
    }
}
