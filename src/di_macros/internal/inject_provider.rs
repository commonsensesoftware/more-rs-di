use super::{CallSiteContext, InjectedCallSite, InjectionStrategy};
use quote::quote;

pub struct ServiceProviderInjector;

impl InjectionStrategy for ServiceProviderInjector {
    fn inject(&self, context: &CallSiteContext) -> InjectedCallSite {
        InjectedCallSite {
            resolve: if context.scoped {
                quote! { sp.into() } // ServiceProvider.into() -> ScopedServiceProvider
            } else {
                quote! { sp.clone() } // ServiceProvider.clone() -> ServiceProvider
            },
            dependency: None,
        }
    }
}
