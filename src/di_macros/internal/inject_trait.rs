use super::CallSiteInjector;
use quote::ToTokens;
use syn::{TypePath, TypeTraitObject};

pub struct TraitInjector<'a> {
    trait_: &'a TypeTraitObject,
    key: Option<&'a TypePath>,
}

impl<'a> TraitInjector<'a> {
    pub fn new(trait_: &'a TypeTraitObject) -> Self {
        Self {
            trait_,
            key: None,
        }
    }

    pub fn keyed(trait_: &'a TypeTraitObject, key: &'a TypePath) -> Self {
        Self {
            trait_,
            key: Some(key),
        }
    }
}

impl<'a> CallSiteInjector<'a> for TraitInjector<'a> {
    fn service(&self) -> &'a dyn ToTokens {
        self.trait_
    }

    fn key(&self) -> Option<&'a TypePath> {
        self.key
    }
}
