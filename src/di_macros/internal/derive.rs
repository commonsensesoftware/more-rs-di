use syn::{punctuated::Punctuated, token::Plus, Generics, ItemStruct, Path, Signature};

pub enum MacroTarget<'a> {
    Method(&'a Signature),
    Struct(&'a ItemStruct),
}

pub struct DeriveContext<'a> {
    pub generics: &'a Generics,
    pub implementation: &'a Path,
    pub service: Punctuated<Path, Plus>,
    target: MacroTarget<'a>,
}

impl<'a> DeriveContext<'a> {
    pub fn for_method(
        generics: &'a Generics,
        implementation: &'a Path,
        service: Punctuated<Path, Plus>,
        method: &'a Signature,
    ) -> Self {
        Self {
            generics,
            implementation,
            service,
            target: MacroTarget::Method(method),
        }
    }

    pub fn for_struct(
        generics: &'a Generics,
        implementation: &'a Path,
        service: Punctuated<Path, Plus>,
        struct_: &'a ItemStruct,
    ) -> Self {
        Self {
            generics,
            implementation,
            service,
            target: MacroTarget::Struct(struct_),
        }
    }

    pub fn target(&self) -> &MacroTarget<'a> {
        &self.target
    }

    pub fn is_trait(&self) -> bool {
        if self.service.len() > 1 {
            return true;
        }

        let impl_ = &self.implementation.segments.last().unwrap().ident;
        let struct_ = &self.service.first().unwrap().segments.last().unwrap().ident;
        impl_ != struct_
    }
}
