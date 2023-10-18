use super::{
    CallSiteContext, CallSiteContextBuilder, DefaultInjector, InjectedCallSite, InjectionStrategy,
    ServiceProviderInjector, StructInjector, TraitInjector,
};
use proc_macro2::{Ident, Span};
use syn::{
    spanned::Spanned, Error, GenericArgument, PathArguments, Result, Type, TypeParamBound, TypePath,
};

const SUPPORTED_TYPES: &str =
    "Expected ServiceRef, ServiceRefMut, KeyedServiceRef, KeyedServiceRefMut, Rc, or Arc.";

pub struct CallSite;

impl CallSite {
    pub fn visit(callsite: &Type, allow_default: bool) -> Result<InjectedCallSite> {
        let context = Self::new_context(callsite)?;
        let strategy = Self::get_strategy(callsite, &context, allow_default)?;
        Ok(strategy.inject(&context))
    }

    fn get_strategy<'a>(
        arg: &Type,
        context: &CallSiteContext<'a>,
        allow_default: bool,
    ) -> Result<Box<dyn InjectionStrategy + 'a>> {
        let args = Self::visit_first_of(
            context,
            &[
                "ServiceRef",
                "Rc",
                "Arc",
                "KeyedServiceRef",
                "ServiceRefMut",
                "KeyedServiceRefMut",
            ],
        );
        let count = args.len();

        if count > 0 {
            if context.optional_of_many() {
                return Err(Error::new(
                    arg.span(),
                    "Option<Vec> is not supported. Did you mean Vec?",
                ));
            }

            match count {
                1 => match args[0] {
                    Type::TraitObject(ref trait_) => Ok(Box::new(TraitInjector::new(trait_))),
                    Type::Path(ref struct_) => Ok(Box::new(StructInjector::new(struct_))),
                    _ => Err(Error::new(args[0].span(), "Expected a trait or struct.")),
                },
                2 => {
                    if let Type::Path(ref key) = args[0] {
                        match args[1] {
                            Type::TraitObject(ref trait_) => {
                                Ok(Box::new(TraitInjector::keyed(trait_, key)))
                            }
                            Type::Path(ref struct_) => {
                                Ok(Box::new(StructInjector::keyed(struct_, key)))
                            }
                            _ => Err(Error::new(args[1].span(), "Expected a trait or struct.")),
                        }
                    } else {
                        Err(Error::new(args[0].span(), "Expected a struct."))
                    }
                }
                count => Err(Error::new(
                    arg.span(),
                    format!("Expected 1-2 type arguments, but found {}.", count),
                )),
            }
        } else if context.type_.path.segments.last().unwrap().ident
            == Ident::new("ServiceProvider", Span::call_site())
        {
            Ok(Box::new(ServiceProviderInjector))
        } else if allow_default {
            Ok(Box::new(DefaultInjector))
        } else {
            Err(Error::new(context.type_.span(), SUPPORTED_TYPES))
        }
    }

    fn new_context(arg: &Type) -> Result<CallSiteContext<'_>> {
        let mut builder = CallSiteContextBuilder::default();
        let mut read_only = true;
        let input = if let Some(ty) = Self::try_visit_iterator(arg) {
            builder.is_iterator();
            ty
        } else {
            arg
        };

        if let Type::Path(outer) = input {
            if Self::is_mutable_type(outer) {
                builder.is_mutable();
                read_only = false;
            }

            let type_ = if let Some(inner) = Self::try_visit_lazy(outer) {
                match inner {
                    Type::Path(path) => {
                        builder.is_lazy();
                        path
                    }
                    _ => outer,
                }
            } else {
                outer
            };

            if let Some(inner) = Self::try_visit_option(type_) {
                if let Type::Path(path) = inner {
                    builder.is_optional();
                    builder.has_type(path);

                    if read_only && Self::is_mutable_type(path) {
                        builder.is_mutable();
                    }

                    Ok(builder.build())
                } else {
                    Err(Error::new(inner.span(), SUPPORTED_TYPES))
                }
            } else if let Some(inner) = Self::try_visit_vector(type_) {
                if let Type::Path(path) = inner {
                    builder.has_many();
                    builder.has_type(path);

                    if read_only && Self::is_mutable_type(path) {
                        builder.is_mutable();
                    }

                    Ok(builder.build())
                } else {
                    Err(Error::new(inner.span(), SUPPORTED_TYPES))
                }
            } else {
                builder.has_type(type_);

                if read_only && Self::is_mutable_type(type_) {
                    builder.is_mutable();
                }

                Ok(builder.build())
            }
        } else {
            Err(Error::new(arg.span(), "Expected type path."))
        }
    }

    fn is_mutable_type(type_: &TypePath) -> bool {
        if let Some(name) = type_.path.segments.last() {
            if name.ident == Ident::new("ServiceRefMut", Span::call_site())
                || name.ident == Ident::new("KeyedServiceRefMut", Span::call_site())
            {
                return true;
            }

            if name.ident == Ident::new("Rc", Span::call_site())
                || name.ident == Ident::new("Arc", Span::call_site())
            {
                if let PathArguments::AngleBracketed(ref generics) = name.arguments {
                    if let GenericArgument::Type(arg) = generics.args.first().unwrap() {
                        if let Type::Path(ty) = arg {
                            if let Some(name) = ty.path.segments.last() {
                                return name.ident == Ident::new("Mutex", Span::call_site());
                            }
                        }
                    }
                }
            }
        }

        return false;
    }

    #[inline]
    fn try_visit_lazy<'a>(type_: &'a TypePath) -> Option<&'a Type> {
        Self::visit_generic_type_arg(type_, "Lazy")
    }

    #[inline]
    fn try_visit_option<'a>(type_: &'a TypePath) -> Option<&'a Type> {
        Self::visit_generic_type_arg(type_, "Option")
    }

    #[inline]
    fn try_visit_vector<'a>(type_: &'a TypePath) -> Option<&'a Type> {
        Self::visit_generic_type_arg(type_, "Vec")
    }

    #[inline]
    fn visit_generic_type_arg<'a>(type_: &'a TypePath, name: &str) -> Option<&'a Type> {
        let args = Self::visit_generic_type_args(type_, name);

        if args.is_empty() {
            None
        } else {
            Some(args[0])
        }
    }

    fn visit_first_of<'a>(context: &CallSiteContext<'a>, names: &[&str]) -> Vec<&'a Type> {
        for name in names {
            let args = Self::visit_generic_type_args(context.type_, name);

            if args.len() > 0 {
                return args;
            }
        }

        Vec::with_capacity(0)
    }

    /* visit generic type arguments of:
     *
     * Rc<T>
     * Arc<T>
     * ServiceRef<T>
     * ServiceRefMut<T>
     * KeyedServiceRef<K,T>
     * KeyedServiceRefMut<K,T>
     * Lazy<T>
     * Option<T>
     * Vec<T>
     */
    fn visit_generic_type_args<'a>(type_: &'a TypePath, name: &str) -> Vec<&'a Type> {
        let path = &type_.path;
        let segment = path.segments.last().unwrap();

        if segment.ident == Ident::new(name, Span::call_site()) {
            if let PathArguments::AngleBracketed(ref type_args) = segment.arguments {
                let count = type_args.args.len();

                if count > 0 {
                    let mut args = Vec::with_capacity(count);

                    for type_arg in type_args.args.iter() {
                        if let GenericArgument::Type(ref inner_type) = type_arg {
                            args.push(inner_type);
                        }
                    }

                    if args.len() == count {
                        return args;
                    }
                }
            }
        }

        Vec::with_capacity(0)
    }

    // impl Iterator<Item = ?>
    fn try_visit_iterator<'a>(arg: &'a Type) -> Option<&'a Type> {
        if let Type::ImplTrait(impl_) = arg {
            for bound in impl_.bounds.iter() {
                if let TypeParamBound::Trait(trait_) = bound {
                    let iterator = trait_.path.segments.last().unwrap();

                    if iterator.ident == Ident::new("Iterator", Span::call_site()) {
                        if let PathArguments::AngleBracketed(ref generic) = iterator.arguments {
                            if generic.args.len() == 1 {
                                if let GenericArgument::AssocType(item) =
                                    generic.args.first().unwrap()
                                {
                                    return Some(&item.ty);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
