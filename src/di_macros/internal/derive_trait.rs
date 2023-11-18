use super::{Constructor, DeriveContext, Fields, MacroTarget};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

pub struct InjectableTrait;

impl InjectableTrait {
    pub fn derive<'a>(context: &'a DeriveContext<'a>) -> Result<TokenStream> {
        let callsites = match *context.target() {
            MacroTarget::Method(method) => Constructor::visit(method)?,
            MacroTarget::Struct(struct_) => Fields::visit(struct_)?,
        };
        let mut args = Vec::with_capacity(callsites.len());
        let mut deps = Vec::with_capacity(callsites.len());

        for callsite in &callsites {
            args.push(&callsite.resolve);

            if let Some(ref dep) = callsite.dependency {
                deps.push(dep);
            }
        }

        let service = if context.is_trait() {
            let svc = context.service;
            quote! { dyn #svc }
        } else {
            quote! { Self }
        };

        let implementation = &context.implementation;
        let depends_on = quote! { #(.depends_on(#deps))* };
        let (generics, _, where_) = context.generics.split_for_impl();
        let activate = match *context.target() {
            MacroTarget::Method(method) => {
                let fn_ = &method.ident;
                quote! { Self::#fn_(#(#args),*) }
            }
            MacroTarget::Struct(struct_) => match &struct_.fields {
                syn::Fields::Named(fields) => {
                    let names = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
                    quote! { Self { #(#names: #args),* } }
                }
                syn::Fields::Unnamed(_) => quote! { Self(#(#args),*) },
                syn::Fields::Unit => quote! { Self },
            },
        };
        let activate2 = activate.clone();
        let code = quote! {
            impl#generics di::Injectable for #implementation #where_ {
                fn inject(lifetime: di::ServiceLifetime) -> di::InjectBuilder {
                    di::InjectBuilder::new(
                        di::Activator::new::<#service, Self>(
                            |sp: &di::ServiceProvider| di::Ref::new(#activate),
                            |sp: &di::ServiceProvider| di::Ref::new(std::sync::Mutex::new(#activate2))
                        ),
                        lifetime
                    )#depends_on
                }
            }
        };

        Ok(code.into())
    }
}
