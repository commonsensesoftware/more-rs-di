mod internal;

extern crate proc_macro;

use crate::internal::*;
use internal::{DeriveContext, DeriveStrategy, InjectableTrait, KeyedInjectableTrait, Constructor};
use proc_macro2::TokenStream;
use syn::{spanned::Spanned, *};

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
    let original = TokenStream::from(input.clone());
    let result = match parse2::<InjectableAttribute>(metadata) {
        Ok(attribute) => {
            if let Ok(impl_) = parse2::<ItemImpl>(TokenStream::from(input.clone())) {
                derive_from_struct_impl(impl_, attribute, original)
            } else if let Ok(struct_) = parse2::<ItemStruct>(TokenStream::from(input)) {
                derive_from_struct(struct_, attribute, original)
            } else {
                Err(Error::new(
                    original.span(),
                    "Attribute can only be applied to a structure or structure implementation block.",
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

fn derive_from_struct_impl(
    impl_: ItemImpl,
    attribute: InjectableAttribute,
    original: TokenStream,
) -> Result<TokenStream> {
    if let Type::Path(type_) = &*impl_.self_ty {
        let imp = &type_.path;
        let svc = attribute.trait_.as_ref().unwrap_or(imp);

        match Constructor::select(&impl_, imp) {
            Ok(method) => {
                let context = DeriveContext::for_method(&impl_.generics, imp, &svc, method);
                derive(context, &attribute, original)
            }
            Err(error) => Err(error),
        }
    } else {
        Err(Error::new(impl_.span(), "Expected implementation type."))
    }
}

fn derive_from_struct(
    struct_: ItemStruct,
    attribute: InjectableAttribute,
    original: TokenStream,
) -> Result<TokenStream> {
    let path = syn::Path::from(struct_.ident.clone());
    let imp = &path;
    let svc = attribute.trait_.as_ref().unwrap_or(imp);
    let context = DeriveContext::for_struct(&struct_.generics, imp, svc, &struct_);
    
    derive(context, &attribute, original)
}

fn derive<'a>(
    context: DeriveContext<'a>,
    attribute: &'a InjectableAttribute,
    mut original: TokenStream,
) -> Result<TokenStream> {
    let injectable = InjectableTrait::derive(&context);

    if injectable.is_ok() {
        if attribute.keyed {
            match KeyedInjectableTrait::derive(&context) {
                Ok(keyed_injectable) => {
                    original.extend(injectable.unwrap().into_iter());
                    original.extend(keyed_injectable.into_iter());
                    Ok(original)
                }
                Err(error) => Err(error),
            }
        } else {
            original.extend(injectable.unwrap().into_iter());
            Ok(original)
        }
    } else {
        Err(injectable.unwrap_err())
    }
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
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new ())) ",
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
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: create ())) ",
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
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get_required :: < dyn Bar > ()))) ",
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
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ZeroOrOne)) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get :: < dyn Bar > ()))) ",
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
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ZeroOrMore)) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get_all :: < dyn Bar > () . collect ()))) ",
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
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Foo > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ZeroOrOne)) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: create_new (sp . get_required :: < dyn Foo > () , sp . get :: < dyn Bar > ()))) ",
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
            "di :: ServiceDescriptorBuilder :: < Self , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new ())) ",
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
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < Bar > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get_required :: < Bar > ()))) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_implement_injectable_for_generic_struct() {
        // arrange
        let metadata = TokenStream::new();
        let input = TokenStream::from_str(
            r#"
            impl<T: Default> GenericBar<T> {
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
            "impl < T : Default > GenericBar < T > { ",
            "fn new () -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl < T : Default > di :: Injectable for GenericBar < T > { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < Self , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new ())) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }

    #[test]
    fn attribute_should_implement_injectable_for_generic_trait() {
        // arrange
        let metadata = TokenStream::from_str(r#"Pair<TKey, TValue>"#).unwrap();
        let input = TokenStream::from_str(
            r#"
            impl<TKey, TValue> PairImpl<TKey, TValue>
            where
                TKey: Debug,
                TValue: Debug
            {
                fn new(key: ServiceRef<TKey>, value: ServiceRef<TValue>) -> Self {
                    Self { key, value }
                }
            }
        "#,
        )
        .unwrap();

        // act
        let result = _injectable(metadata, input);

        // assert
        let expected = concat!(
            "impl < TKey , TValue > PairImpl < TKey , TValue > ",
            "where ",
            "TKey : Debug , ",
            "TValue : Debug ",
            "{ ",
            "fn new (key : ServiceRef < TKey >, value : ServiceRef < TValue >) -> Self { ",
            "Self { key , value } ",
            "} ",
            "} ",
            "impl < TKey , TValue > di :: Injectable for PairImpl < TKey , TValue > ",
            "where ",
            "TKey : Debug , ",
            "TValue : Debug ",
            "{ ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: ServiceDescriptor { ",
            "di :: ServiceDescriptorBuilder :: < dyn Pair < TKey , TValue > , Self > :: new (lifetime , di :: Type :: of :: < Self > ()) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < TKey > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < TValue > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". from (| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (\
                sp . get_required :: < TKey > () , \
                sp . get_required :: < TValue > ()))) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }
}
