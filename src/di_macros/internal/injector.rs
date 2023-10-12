use super::{CallSiteContext, InjectedCallSite, InjectionStrategy};
use quote::{quote, ToTokens};
use syn::TypePath;

pub trait CallSiteInjector<'a> {
    fn service(&self) -> &'a dyn ToTokens;

    fn key(&self) -> Option<&'a TypePath> {
        None
    }

    fn optional(&self, context: &CallSiteContext) -> InjectedCallSite {
        let svc = self.service();

        if let Some(key) = self.key() {
            InjectedCallSite {
                resolve: if context.lazy {
                    quote! { di::lazy::zero_or_one_with_key::<#key, #svc>(sp.clone()) }
                } else {
                    quote! { sp.get_by_key::<#key, #svc>() }
                },
                dependency: Some(
                    quote! { di::ServiceDependency::new(di::Type::keyed::<#key, #svc>(), di::ServiceCardinality::ZeroOrOne) },
                ),
            }
        } else {
            InjectedCallSite {
                resolve: if context.lazy {
                    quote! { di::lazy::zero_or_one::<#svc>(sp.clone()) }
                } else {
                    quote! { sp.get::<#svc>() }
                },
                dependency: Some(
                    quote! { di::ServiceDependency::new(di::Type::of::<#svc>(), di::ServiceCardinality::ZeroOrOne) },
                ),
            }
        }
    }

    fn required(&self, context: &CallSiteContext) -> InjectedCallSite {
        let svc = self.service();

        if let Some(key) = self.key() {
            InjectedCallSite {
                resolve: if context.lazy {
                    quote! { di::lazy::exactly_one_with_key::<#key, #svc>(sp.clone()) }
                } else {
                    quote! { sp.get_required_by_key::<#key, #svc>() }
                },
                dependency: Some(
                    quote! { di::ServiceDependency::new(di::Type::keyed::<#key, #svc>(), di::ServiceCardinality::ExactlyOne) },
                ),
            }
        } else {
            InjectedCallSite {
                resolve: if context.lazy {
                    quote! { di::lazy::exactly_one::<#svc>(sp.clone()) }
                } else {
                    quote! { sp.get_required::<#svc>() }
                },
                dependency: Some(
                    quote! { di::ServiceDependency::new(di::Type::of::<#svc>(), di::ServiceCardinality::ExactlyOne) },
                ),
            }
        }
    }

    fn many(&self, context: &CallSiteContext) -> InjectedCallSite {
        let svc = self.service();

        if let Some(key) = self.key() {
            InjectedCallSite {
                resolve: if context.lazy {
                    quote! { di::lazy::zero_or_more_with_key::<#key, #svc>(sp.clone()) }
                } else if context.iterator {
                    quote! { sp.get_all_by_key::<#key, #svc>() }
                } else {
                    quote! { sp.get_all_by_key::<#key, #svc>().collect() }
                },
                dependency: Some(
                    quote! { di::ServiceDependency::new(di::Type::keyed::<#key, #svc>(), di::ServiceCardinality::ZeroOrMore) },
                ),
            }
        } else {
            InjectedCallSite {
                resolve: if context.lazy {
                    quote! { di::lazy::zero_or_more::<#svc>(sp.clone()) }
                } else if context.iterator {
                    quote! { sp.get_all::<#svc>() }
                } else {
                    quote! { sp.get_all::<#svc>().collect() }
                },
                dependency: Some(
                    quote! { di::ServiceDependency::new(di::Type::of::<#svc>(), di::ServiceCardinality::ZeroOrMore) },
                ),
            }
        }
    }
}

impl<'a, T> InjectionStrategy for T
where
    T: CallSiteInjector<'a>,
{
    fn inject(&self, context: &CallSiteContext) -> InjectedCallSite {
        if context.optional {
            self.optional(context)
        } else if context.many {
            self.many(context)
        } else {
            self.required(context)
        }
    }
}
