use super::{CallSiteContext, InjectedCallSite, InjectionStrategy};
use quote::quote;

pub struct ServiceProviderInjector;

impl InjectionStrategy for ServiceProviderInjector {
    fn inject(&self, _context: &CallSiteContext) -> InjectedCallSite {
        InjectedCallSite {
            resolve: quote! { sp.clone() },
            dependency: None,
        }
    }
}
