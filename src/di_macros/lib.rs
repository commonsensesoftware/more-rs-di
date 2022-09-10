extern crate proc_macro;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    *,
};

struct InjectableAttribute {
    trait_: Option<Ident>,
}

impl Parse for InjectableAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            trait_: input.parse().ok(),
        })
    }
}

/// Represents the metadata used to identify an injected function.
///
/// # Remarks
///
/// The default behavior looks for an associated function with the
/// name `new`. To change this behavior, decorate the function to
/// be used with `#[inject]`. This attribute may only be applied
/// to a single function.
#[proc_macro_attribute]
pub fn inject(
    _metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // this attribute is intentionally inert
    input
}

/// Represents the metadata used to implement the `Injectable` trait.
///
/// # Arguments
///
/// * `trait` - the optional name of the trait the implementation satisfies.
///
/// # Remarks
///
/// This attribute must be applied to the `impl` of a struct. The
/// defining struct implementation must either have an associated
/// function named `new` or decorate the injected function with
/// `#[inject]`. The injected function does not have to be public.
/// 
/// If `trait` is not specified, then the implementation will
/// injectable as the defining struct itself.
///
/// The injected call site arguments are restricted to the same return
/// values supported by `ServiceProvider`, which can only be:
///
/// * `ServiceRef<T>`
/// * `Option<ServiceRef<T>>`
/// * `Vec<ServiceRef<T>>`
/// * `ServiceProvider`
/// 
/// `ServiceRef<T>` is a type alias for `Rc<T>` or `Arc<T>` depending
/// on whether the **async** feature is activated; therefore, `Rc<T>`
/// and `Arc<T>` are also allowed any place `ServiceRef<T>` is allowed.
/// 
/// # Examples
/// 
/// Injecting a struct as a trait.
/// 
/// ```
/// pub trait Foo {
///    fn do_work(&self);
/// }
/// 
/// pub struct FooImpl;
/// 
/// impl Foo for FooImpl {
///     fn do_work(&self) {
///         println!("Did something!");
///     }
/// }
/// 
/// #[injectable(Foo)]
/// impl FooImpl {
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
/// ```
/// 
/// Injecting a struct as itself.
/// 
/// ```
/// pub struct Foo;
/// 
/// #[injectable]
/// impl Foo {
///     pub fn new() -> Self {
///         Self {}
///     }
/// 
///     fn do_work(&self) {
///         println!("Did something!");
///     }
/// }
/// ```
/// 
/// Define a custom injection function.
/// 
/// ```
/// pub struct Bar;
/// pub struct Foo {
///     bar: ServiceRef<Bar>
/// };
/// 
/// #[injectable]
/// impl Foo {
///     #[inject]
///     pub fn create(bar: ServiceRef<Bar>) -> Self {
///         Self { bar }
///     }
/// }
#[proc_macro_attribute]
pub fn injectable(
    metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(_injectable(
        TokenStream::from(metadata),
        TokenStream::from(input),
    ))
}

fn _injectable(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let mut original = TokenStream::from(input.clone());
    let result = match parse2::<InjectableAttribute>(metadata) {
        Ok(attribute) => {
            if let Ok(impl_) = parse2::<ItemImpl>(TokenStream::from(input)) {
                if let Type::Path(type_) = &*impl_.self_ty {
                    let implementation = &type_.path.segments.first().unwrap().ident;
                    let service = attribute.trait_.as_ref().unwrap_or(implementation);

                    match get_injected_method(&impl_, implementation) {
                        Ok(method) => match implement_injectable(implementation, service, method) {
                            Ok(trait_impl) => {
                                original.extend(trait_impl.into_iter());
                                Ok(original)
                            }
                            Err(error) => Err(error),
                        },
                        Err(error) => Err(error),
                    }
                } else {
                    Err(Error::new(impl_.span(), "Expected implementation type."))
                }
            } else {
                Err(Error::new(
                    original.span(),
                    "Attribute can only be applied to a structure implementation block.",
                ))
            }
        }
        Err(error) => Err(error),
    };

    match result {
        Ok(output) => output,
        Err(error) => error.to_compile_error().into(),
    }
}

fn implement_injectable(
    implementation: &Ident,
    service: &Ident,
    method: &Signature,
) -> Result<TokenStream> {
    let args = inject_argument_call_sites(method)?;
    let fn_ = &method.ident;
    let is_trait = implementation != service;
    let new = if is_trait {
        quote! { di::ServiceDescriptorBuilder::<dyn #service, Self>::new(lifetime, di::Type::of::<Self>()) }
    } else {
        quote! { di::ServiceDescriptorBuilder::<#service, Self>::new(lifetime, di::Type::of::<Self>()) }
    };
    let code = quote! {
        impl di::Injectable for #implementation {
            fn inject(lifetime: di::ServiceLifetime) -> di::ServiceDescriptor {
                #new.from(|sp: &di::ServiceProvider| di::ServiceRef::new(#implementation::#fn_(#(#args),*)))
            }
        }
    };
    Ok(code.into())
}

fn get_injected_method<'a>(impl_: &'a ItemImpl, type_: &Ident) -> Result<&'a Signature> {
    let new = Ident::new("new", Span::call_site());
    let mut convention = Option::None;
    let mut methods = Vec::new();

    for item in &impl_.items {
        if let ImplItem::Method(method) = item {
            let signature = &method.sig;

            if method.attrs.iter().any(|a| a.path.is_ident("inject")) {
                methods.push(signature);
            }

            if signature.ident == new {
                convention = Some(signature);
            }
        }
    }

    match methods.len() {
        0 => {
            if let Some(method) = convention {
                Ok(method)
            } else {
                Err(Error::new(
                    impl_.span(),
                    format!(
                        "Neither {}::new or an associated method decorated with #[inject] was found.",
                        type_
                    ),
                ))
            }
        }
        1 => Ok(methods[0]),
        _ => Err(Error::new(
            impl_.span(),
            format!(
                "{} has more than one associated method decorated with #[inject].",
                type_
            ),
        )),
    }
}

fn inject_argument_call_sites(method: &Signature) -> Result<Vec<TokenStream>> {
    let mut args = Vec::with_capacity(method.inputs.len());

    if args.capacity() == 0 {
        return Ok(args);
    }

    for input in method.inputs.iter() {
        args.push(match input {
            FnArg::Typed(type_) => resolve_type(&*type_.ty)?,
            _ => return Err(Error::new(
                input.span(),
                "The argument must be ServiceRef, Rc, or Arc and optionally wrapped with Option or Vec.")),
        });
    }

    Ok(args)
}

fn resolve_type(arg: &Type) -> Result<TokenStream> {
    if let Type::Path(type_) = arg {
        let optional;
        let many;
        let inner_type = if let Some(inner) = get_generic_type_arg(type_, "Option") {
            optional = true;
            many = false;

            if let Type::Path(path) = inner {
                path
            } else {
                return Err(Error::new(inner.span(), "Expected ServiceRef, Rc, or Arc."));
            }
        } else if let Some(inner) = get_generic_type_arg(type_, "Vec") {
            optional = false;
            many = true;

            if let Type::Path(path) = inner {
                path
            } else {
                return Err(Error::new(inner.span(), "Expected ServiceRef, Rc, or Arc."));
            }
        } else {
            optional = false;
            many = false;
            type_
        };

        if let Some(inner_type) = get_generic_type_arg(inner_type, "ServiceRef")
            .or(get_generic_type_arg(inner_type, "Rc"))
            .or(get_generic_type_arg(inner_type, "Arc"))
        {
            if optional && many {
                return Err(Error::new(
                    arg.span(),
                    "Option<Vec> is not supported. Did you mean Vec?",
                ));
            }

            return match inner_type {
                Type::TraitObject(trait_) => {
                    if optional {
                        Ok(quote! { sp.get::<#trait_>() })
                    } else if many {
                        Ok(quote! { sp.get_all::<#trait_>().collect() })
                    } else {
                        Ok(quote! { sp.get_required::<#trait_>() })
                    }
                }
                Type::Path(struct_) => {
                    if optional {
                        Ok(quote! { sp.get::<#struct_>() })
                    } else if many {
                        Ok(quote! { sp.get_all::<#struct_>().collect() })
                    } else {
                        Ok(quote! { sp.get_required::<#struct_>() })
                    }
                }
                _ => Err(Error::new(inner_type.span(), "Expected a trait or struct.")),
            };
        } else if inner_type.path.segments.first().unwrap().ident
            == Ident::new("ServiceProvider", Span::call_site())
        {
            return Ok(quote! { sp.clone() });
        } else {
            return Err(Error::new(
                inner_type.span(),
                "Expected ServiceRef, Rc, or Arc.",
            ));
        }
    }

    Err(Error::new(arg.span(), "Expected type path."))
}

fn get_generic_type_arg<'a>(type_: &'a TypePath, name: &str) -> Option<&'a Type> {
    let path = &type_.path;
    let segment = path.segments.first().unwrap();

    if segment.ident == Ident::new(name, Span::call_site()) {
        if let PathArguments::AngleBracketed(ref type_args) = segment.arguments {
            for type_arg in type_args.args.iter() {
                if let GenericArgument::Type(ref inner_type) = type_arg {
                    return Some(inner_type);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn attribute_should_implement_injectable_by_convention() {
        // arrange
        let metadata = TokenStream::from_str(r#"Foo"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl FooImpl {
                fn new() -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl FooImpl { ",
            "fn new () -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for FooImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Foo , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (FooImpl :: new ())) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_implement_injectable_using_decorated_method() {
        // arrange
        let metadata = TokenStream::from_str(r#"Foo"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl FooImpl {
                #[inject]
                fn create() -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl FooImpl { ",
            "# [inject] ",
            "fn create () -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for FooImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Foo , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (FooImpl :: create ())) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_inject_required_dependency() {
        // arrange
        let metadata = TokenStream::from_str(r#"Foo"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl FooImpl {
                fn new(_bar: Rc<dyn Bar>) -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl FooImpl { ",
            "fn new (_bar : Rc < dyn Bar >) -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for FooImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Foo , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (FooImpl :: new (sp . get_required :: < dyn Bar > ()))) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_inject_optional_dependency() {
        // arrange
        let metadata = TokenStream::from_str(r#"Foo"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl FooImpl {
                fn new(_bar: Option<Rc<dyn Bar>>) -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl FooImpl { ",
            "fn new (_bar : Option < Rc < dyn Bar >>) -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for FooImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Foo , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (FooImpl :: new (sp . get :: < dyn Bar > ()))) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_inject_dependency_collection() {
        // arrange
        let metadata = TokenStream::from_str(r#"Foo"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl FooImpl {
                fn new(_bars: Vec<Rc<dyn Bar>>) -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl FooImpl { ",
            "fn new (_bars : Vec < Rc < dyn Bar >>) -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for FooImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Foo , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (FooImpl :: new (sp . get_all :: < dyn Bar > () . collect ()))) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_inject_multiple_dependencies() {
        // arrange
        let metadata = TokenStream::from_str(r#"Thing"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl ThingImpl {
                #[inject]
                fn create_new(_foo: ServiceRef<dyn Foo>, _bar: Option<ServiceRef<dyn Bar>>) -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl ThingImpl { ",
            "# [inject] ",
            "fn create_new (_foo : ServiceRef < dyn Foo >, _bar : Option < ServiceRef < dyn Bar >>) -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for ThingImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Thing , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (ThingImpl :: create_new (sp . get_required :: < dyn Foo > () , sp . get :: < dyn Bar > ()))) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_implement_injectable_for_self() {
        // arrange
        let metadata = TokenStream::new();
        let input = TokenStream::from_str(
            r#"
            impl FooImpl {
                fn new() -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl FooImpl { ",
            "fn new () -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for FooImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < FooImpl , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (FooImpl :: new ())) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_implement_injectable_for_struct() {
        // arrange
        let metadata = TokenStream::from_str(r#"Foo"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl FooImpl {
                fn new(_bar: Rc<Bar>) -> Self {
                    Self { }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl FooImpl { ",
            "fn new (_bar : Rc < Bar >) -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for FooImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Foo , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (FooImpl :: new (sp . get_required :: < Bar > ()))) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }
}
