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
                    if context.mutable {
                        quote! { di::lazy::zero_or_one_with_key_mut::<#key, #svc>(sp.clone()) }
                    } else {
                        quote! { di::lazy::zero_or_one_with_key::<#key, #svc>(sp.clone()) }
                    }
                } else if context.mutable {
                    quote! { sp.get_by_key_mut::<#key, #svc>() }
                } else {
                    quote! { sp.get_by_key::<#key, #svc>() }
                },
                dependency: if context.mutable {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::keyed::<#key, di::Mut<#svc>>(),
                        di::ServiceCardinality::ZeroOrOne) })
                } else {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::keyed::<#key, #svc>(),
                        di::ServiceCardinality::ZeroOrOne) })
                },
            }
        } else {
            InjectedCallSite {
                resolve: if context.lazy {
                    if context.mutable {
                        quote! { di::lazy::zero_or_one_mut::<#svc>(sp.clone()) }
                    } else {
                        quote! { di::lazy::zero_or_one::<#svc>(sp.clone()) }
                    }
                } else if context.mutable {
                    quote! { sp.get_mut::<#svc>() }
                } else {
                    quote! { sp.get::<#svc>() }
                },
                dependency: if context.mutable {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::of::<di::Mut<#svc>>(),
                        di::ServiceCardinality::ZeroOrOne) })
                } else {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::of::<#svc>(),
                        di::ServiceCardinality::ZeroOrOne) })
                },
            }
        }
    }

    fn required(&self, context: &CallSiteContext) -> InjectedCallSite {
        let svc = self.service();

        if let Some(key) = self.key() {
            InjectedCallSite {
                resolve: if context.lazy {
                    if context.mutable {
                        quote! { di::lazy::exactly_one_with_key_mut::<#key, #svc>(sp.clone()) }
                    } else {
                        quote! { di::lazy::exactly_one_with_key::<#key, #svc>(sp.clone()) }
                    }
                } else if context.mutable {
                    quote! { sp.get_required_by_key_mut::<#key, #svc>() }
                } else {
                    quote! { sp.get_required_by_key::<#key, #svc>() }
                },
                dependency: if context.mutable {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::keyed::<#key, di::Mut<#svc>>(),
                        di::ServiceCardinality::ExactlyOne) })
                } else {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::keyed::<#key, #svc>(),
                        di::ServiceCardinality::ExactlyOne) })
                },
            }
        } else {
            InjectedCallSite {
                resolve: if context.lazy {
                    if context.mutable {
                        quote! { di::lazy::exactly_one_mut::<#svc>(sp.clone()) }
                    } else {
                        quote! { di::lazy::exactly_one::<#svc>(sp.clone()) }
                    }
                } else if context.mutable {
                    quote! { sp.get_required_mut::<#svc>() }
                } else {
                    quote! { sp.get_required::<#svc>() }
                },
                dependency: if context.mutable {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::of::<di::Mut<#svc>>(),
                        di::ServiceCardinality::ExactlyOne) })
                } else {
                    Some(quote! {
                    di::ServiceDependency::new(
                       di::Type::of::<#svc>(),
                       di::ServiceCardinality::ExactlyOne) })
                },
            }
        }
    }

    fn many(&self, context: &CallSiteContext) -> InjectedCallSite {
        let svc = self.service();

        if let Some(key) = self.key() {
            InjectedCallSite {
                resolve: if context.lazy {
                    if context.mutable {
                        quote! { di::lazy::zero_or_more_with_key_mut::<#key, #svc>(sp.clone()) }
                    } else {
                        quote! { di::lazy::zero_or_more_with_key::<#key, #svc>(sp.clone()) }
                    }
                } else if context.iterator {
                    if context.mutable {
                        quote! { sp.get_all_by_key_mut::<#key, #svc>() }
                    } else {
                        quote! { sp.get_all_by_key::<#key, #svc>() }
                    }
                } else {
                    if context.mutable {
                        quote! { sp.get_all_by_key_mut::<#key, #svc>().collect() }
                    } else {
                        quote! { sp.get_all_by_key::<#key, #svc>().collect() }
                    }
                },
                dependency: if context.mutable {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::keyed::<#key, di::Mut<#svc>>(),
                        di::ServiceCardinality::ZeroOrMore) })
                } else {
                    Some(quote! {
                    di::ServiceDependency::new(
                       di::Type::keyed::<#key, #svc>(),
                       di::ServiceCardinality::ZeroOrMore) })
                },
            }
        } else {
            InjectedCallSite {
                resolve: if context.lazy {
                    if context.mutable {
                        quote! { di::lazy::zero_or_more_mut::<#svc>(sp.clone()) }
                    } else {
                        quote! { di::lazy::zero_or_more::<#svc>(sp.clone()) }
                    }
                } else if context.iterator {
                    if context.mutable {
                        quote! { sp.get_all_mut::<#svc>() }
                    } else {
                        quote! { sp.get_all::<#svc>() }
                    }
                } else {
                    if context.mutable {
                        quote! { sp.get_all_mut::<#svc>().collect() }
                    } else {
                        quote! { sp.get_all::<#svc>().collect() }
                    }
                },
                dependency: if context.mutable {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::of::<di::Mut<#svc>>(),
                        di::ServiceCardinality::ZeroOrMore) })
                } else {
                    Some(quote! {
                    di::ServiceDependency::new(
                        di::Type::of::<#svc>(),
                        di::ServiceCardinality::ZeroOrMore) })
                },
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
