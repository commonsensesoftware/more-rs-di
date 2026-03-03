use super::{CallSiteContext, InjectedCallSite, InjectionStrategy};
use quote::quote;

pub struct DefaultInjector;

impl InjectionStrategy for DefaultInjector {
    fn inject(&self, _context: &CallSiteContext) -> InjectedCallSite {
        InjectedCallSite {
            resolve: quote! { Default::default() },
            dependency: None,
        }
    }
}
