use super::{
    CallSiteContext, CallSiteContextBuilder, DefaultInjector, InjectedCallSite, InjectionStrategy,
    ServiceProviderInjector, StructInjector, TraitInjector,
};
use proc_macro2::{Ident, Span};
use syn::{
    spanned::Spanned, Error, GenericArgument, PathArguments, Result, Type, TypeParamBound, TypePath,
};

use crate::alias::{try_get_aliases, Aliases};

const UNSUPPORTED_TYPE: &str = "Injection is unsupported for the specified type.";

pub struct CallSite;

struct KnownType {
    name: String,
    mutable: bool,
}

impl KnownType {
    fn new<S: AsRef<str>>(name: S, mutable: bool) -> Self {
        Self {
            name: name.as_ref().into(),
            mutable,
        }
    }
}

impl CallSite {
    pub fn visit(callsite: &Type, allow_default: bool) -> Result<InjectedCallSite> {
        let types = Self::get_known_types();
        let context = Self::new_context(callsite, &types)?;
        let strategy = Self::get_strategy(callsite, &types, &context, allow_default)?;
        Ok(strategy.inject(&context))
    }

    fn get_known_types() -> Vec<KnownType> {
        let mut types = vec![
            KnownType::new("Ref", true),
            KnownType::new("Rc", true),
            KnownType::new("Arc", true),
            KnownType::new("KeyedRef", true),
            KnownType::new("RefMut", false),
            KnownType::new("KeyedRefMut", false),
        ];

        Self::merge_type_aliases(&mut types);

        types
    }

    fn merge_type_aliases(types: &mut Vec<KnownType>) {
        let aliases = try_get_aliases().unwrap_or_else(Aliases::legacy);

        if let Some(name) = &aliases.keyed_ref_mut {
            if !types.iter().any(|t| &t.name == name) {
                types.insert(0, KnownType::new(name, false))
            }
        }

        if let Some(name) = &aliases.ref_mut {
            if !types.iter().any(|t| &t.name == name) {
                types.insert(0, KnownType::new(name, false))
            }
        }

        if let Some(name) = &aliases.keyed_ref {
            if !types.iter().any(|t| &t.name == name) {
                types.insert(0, KnownType::new(name, true))
            }
        }

        if let Some(name) = &aliases.r#ref {
            if !types.iter().any(|t| &t.name == name) {
                types.insert(0, KnownType::new(name, true))
            }
        }
    }

    fn get_strategy<'a>(
        arg: &Type,
        known_types: &Vec<KnownType>,
        context: &CallSiteContext<'a>,
        allow_default: bool,
    ) -> Result<Box<dyn InjectionStrategy + 'a>> {
        let args = Self::visit_first_of(context, known_types);
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
        } else if context.scoped
            || context.type_.path.segments.last().unwrap().ident
                == Ident::new("ServiceProvider", Span::call_site())
        {
            Ok(Box::new(ServiceProviderInjector))
        } else if allow_default {
            Ok(Box::new(DefaultInjector))
        } else {
            Err(Error::new(context.type_.span(), UNSUPPORTED_TYPE))
        }
    }

    fn new_context<'a>(
        arg: &'a Type,
        known_types: &'a Vec<KnownType>,
    ) -> Result<CallSiteContext<'a>> {
        let mut builder = CallSiteContextBuilder::default();
        let mut read_only = true;
        let input = if let Some(ty) = Self::try_visit_iterator(arg) {
            builder.is_iterator();
            ty
        } else {
            arg
        };

        if let Type::Path(outer) = input {
            if Self::is_mutable_type(outer, known_types) {
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
                    if read_only && Self::is_mutable_type(path, known_types) {
                        builder.is_mutable();
                    }

                    builder.is_optional();
                    builder.has_type(path);
                    Ok(builder.build())
                } else {
                    Err(Error::new(inner.span(), UNSUPPORTED_TYPE))
                }
            } else if let Some(inner) = Self::try_visit_vector(type_) {
                if let Type::Path(path) = inner {
                    if read_only && Self::is_mutable_type(path, known_types) {
                        builder.is_mutable();
                    }

                    builder.has_many();
                    builder.has_type(path);
                    Ok(builder.build())
                } else {
                    Err(Error::new(inner.span(), UNSUPPORTED_TYPE))
                }
            } else {
                if read_only && Self::is_mutable_type(type_, known_types) {
                    builder.is_mutable();
                }

                if type_.path.segments.last().unwrap().ident
                    == Ident::new("ScopedServiceProvider", Span::call_site())
                {
                    builder.is_scoped();
                }

                builder.has_type(type_);
                Ok(builder.build())
            }
        } else {
            Err(Error::new(arg.span(), "Expected type path."))
        }
    }

    fn is_mutable_type(type_: &TypePath, known_types: &Vec<KnownType>) -> bool {
        if let Some(name) = type_.path.segments.last() {
            if let Some(known_type) = known_types
                .iter()
                .find(|t| name.ident == Ident::new(&t.name, Span::call_site()))
            {
                // Mut<T> = RefCell<T> | Mutex<T>
                // Rc<Mut<T>>
                // Arc<Mut<T>>
                // Ref<Mut<T>>
                // KeyedRef<K,Mut<T>>
                if known_type.mutable {
                    if let PathArguments::AngleBracketed(ref generics) = name.arguments {
                        if let GenericArgument::Type(arg) = generics.args.last().unwrap() {
                            if let Type::Path(ty) = arg {
                                if let Some(name) = ty.path.segments.last() {
                                    return name.ident == Ident::new("RefCell", Span::call_site())
                                        || name.ident == Ident::new("Mutex", Span::call_site());
                                }
                            }
                        }
                    }
                } else {
                    // RefMut<T> = Ref<Mut<T>>
                    // KeyedRefMut<K,T> = KeyedRef<K,Mut<T>>
                    return true;
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

    fn visit_first_of<'a>(
        context: &CallSiteContext<'a>,
        known_types: &Vec<KnownType>,
    ) -> Vec<&'a Type> {
        for KnownType { name, mutable } in known_types {
            let mut args = Self::visit_generic_type_args(context.type_, name);

            if let Some(mut arg) = args.pop() {
                if *mutable {
                    if let Type::Path(path) = arg {
                        if let Some(ty) = Self::visit_generic_type_arg(path, "RefCell") {
                            arg = ty;
                        } else if let Some(ty) = Self::visit_generic_type_arg(path, "Mutex") {
                            arg = ty;
                        }
                    }
                }

                args.push(arg);
                return args;
            }
        }

        Vec::with_capacity(0)
    }

    /* visit generic type arguments of:
     *
     * Rc<T>
     * Arc<T>
     * Ref<T>
     * RefMut<T>
     * KeyedRef<K,T>
     * KeyedRefMut<K,T>
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
