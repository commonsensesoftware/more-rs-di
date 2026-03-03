use super::{CallSiteContext, InjectedCallSite, InjectionStrategy};
use quote::quote;

pub struct ServiceProviderInjector;

impl InjectionStrategy for ServiceProviderInjector {
    fn inject(&self, context: &CallSiteContext) -> InjectedCallSite {
        InjectedCallSite {
            resolve: if context.scoped {
                quote! { ScopedServiceProvider::from(sp) }
            } else {
                quote! { sp.clone() }
            },
            dependency: None,
        }
    }
}
