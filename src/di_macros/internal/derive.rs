use syn::{Generics, ItemStruct, Path, Signature};

pub enum MacroTarget<'a> {
    Method(&'a Signature),
    Struct(&'a ItemStruct),
}

pub struct DeriveContext<'a> {
    pub generics: &'a Generics,
    pub implementation: &'a Path,
    pub service: &'a Path,
    target: MacroTarget<'a>,
}

impl<'a> DeriveContext<'a> {
    pub fn for_method(
        generics: &'a Generics,
        implementation: &'a Path,
        service: &'a Path,
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
        service: &'a Path,
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
        let impl_ = &self.implementation.segments.last().unwrap().ident;
        let struct_ = &self.service.segments.last().unwrap().ident;
        impl_ != struct_
    }
}
