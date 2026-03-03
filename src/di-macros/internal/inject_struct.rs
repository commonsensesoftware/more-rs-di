use super::CallSiteInjector;
use quote::ToTokens;
use syn::TypePath;

pub struct StructInjector<'a> {
    struct_: &'a TypePath,
    key: Option<&'a TypePath>,
}

impl<'a> StructInjector<'a> {
    pub fn new(struct_: &'a TypePath) -> Self {
        Self { struct_, key: None }
    }

    pub fn keyed(struct_: &'a TypePath, key: &'a TypePath) -> Self {
        Self {
            struct_,
            key: Some(key),
        }
    }
}

impl<'a> CallSiteInjector<'a> for StructInjector<'a> {
    fn service(&self) -> &'a dyn ToTokens {
        self.struct_
    }

    fn key(&self) -> Option<&'a TypePath> {
        self.key
    }
}
