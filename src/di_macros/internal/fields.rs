use super::{CallSite, InjectedCallSite};
use syn::{ItemStruct, Result};

pub struct Fields;

impl Fields {
    pub fn visit(struct_: &ItemStruct) -> Result<Vec<InjectedCallSite>> {
        let mut callsites: Vec<InjectedCallSite>;

        match &struct_.fields {
            syn::Fields::Named(fields) => {
                callsites = Vec::with_capacity(fields.named.len());

                for field in &fields.named {
                    callsites.push(CallSite::visit(&field.ty, true)?);
                }
            }
            syn::Fields::Unnamed(fields) => {
                callsites = Vec::with_capacity(fields.unnamed.len());

                for field in &fields.unnamed {
                    callsites.push(CallSite::visit(&field.ty, true)?);
                }
            }
            syn::Fields::Unit => {
                callsites = Vec::with_capacity(0);
            }
        }

        Ok(callsites)
    }
}
