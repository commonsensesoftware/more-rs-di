use crate::internal::CallSiteContext;
use proc_macro2::TokenStream;

pub struct InjectedCallSite {
    pub resolve: TokenStream,
    pub dependency: Option<TokenStream>,
}

pub trait InjectionStrategy {
    fn inject(&self, context: &CallSiteContext) -> InjectedCallSite;
}