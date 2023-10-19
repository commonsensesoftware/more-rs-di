mod internal;

extern crate proc_macro;

use crate::internal::*;
use internal::{Constructor, DeriveContext, InjectableTrait};
use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, spanned::Spanned, token::PathSep, *};

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
/// This attribute may be applied a struct definition or a struct `impl`
/// block. The defining struct implementation block must either have an
/// associated function named `new` or decorate the injected function with
/// `#[inject]`. The injected function does not have to be public.
///
/// If `trait` is not specified, then the implementation will
/// injectable as the defining struct itself.
///
/// The injected call site arguments are restricted to the same return
/// values supported by `ServiceProvider`, which can only be:
///
/// * `ServiceRef<T>`
/// * `ServiceRefMut<T>`
/// * `Option<ServiceRef<T>>`
/// * `Option<ServiceRefMut<T>>`
/// * `Vec<ServiceRef<T>>`
/// * `Vec<ServiceRefMut<T>>`
/// * `impl Iterator<Item = ServiceRef<T>>`
/// * `impl Iterator<Item = ServiceRefMut<T>>`
/// * `Lazy<ServiceRef<T>>`
/// * `Lazy<ServiceRefMut<T>>`
/// * `Lazy<Option<ServiceRef<T>>>`
/// * `Lazy<Option<ServiceRefMut<T>>>`
/// * `Lazy<Vec<ServiceRef<T>>>`
/// * `Lazy<Vec<ServiceRefMut<T>>>`
/// * `KeyedServiceRef<TKey, TSvc>`
/// * `KeyedServiceRefMut<TKey, TSvc>`
/// * `ServiceProvider`
///
/// `ServiceRef<T>` is a type alias for `Rc<T>` or `Arc<T>` and
/// `ServiceRefMut<T>` is a type alias for `Rc<Mutex<T>>` or `Arc<Mutex<T>>`
/// depending on whether the **async** feature is activated; therefore,
/// `Rc<T>` and `Arc<T>` are allowed any place `ServiceRef<T>` is allowed
/// and `Rc<Mutex<T>>` and `Arc<Mutex<T>>` are allowed any place
/// `ServiceRefMut<T>` is allowed.
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
/// #[injectable]
/// pub struct Foo;
///
/// impl Foo {
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
                derive(context, original)
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
    let imp = &build_path_from_struct(&struct_);
    let svc = attribute.trait_.as_ref().unwrap_or(imp);
    let context = DeriveContext::for_struct(&struct_.generics, imp, svc, &struct_);

    derive(context, original)
}

fn build_path_from_struct(struct_: &ItemStruct) -> Path {
    let generics = &struct_.generics;
    let mut segments = Punctuated::<PathSegment, PathSep>::new();
    let segment = PathSegment {
        ident: struct_.ident.clone(),
        arguments: if generics.params.is_empty() {
            PathArguments::None
        } else {
            let mut args = Punctuated::<GenericArgument, Token![,]>::new();

            for param in &generics.params {
                args.push(match param {
                    GenericParam::Const(_) => continue,
                    GenericParam::Type(type_) => GenericArgument::Type(Type::Path(TypePath {
                        qself: None,
                        path: Path::from(type_.ident.clone()),
                    })),
                    GenericParam::Lifetime(param) => {
                        GenericArgument::Lifetime(param.lifetime.clone())
                    }
                });
            }

            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                gt_token: Default::default(),
                args,
                lt_token: Default::default(),
            })
        },
    };

    segments.push(segment);

    Path {
        leading_colon: None,
        segments,
    }
}

#[inline]
fn derive<'a>(context: DeriveContext<'a>, mut original: TokenStream) -> Result<TokenStream> {
    match InjectableTrait::derive(&context) {
        Ok(injectable) => {
            original.extend(injectable.into_iter());
            Ok(original)
        }
        Err(error) => Err(error),
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Foo , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new ()) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new ()))) , ",
            "lifetime) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Foo , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: create ()) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: create ()))) , ",
            "lifetime) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Foo , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get_required :: < dyn Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new (sp . get_required :: < dyn Bar > ())))) , ",
            "lifetime) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ExactlyOne)) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Foo , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get :: < dyn Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new (sp . get :: < dyn Bar > ())))) , ",
            "lifetime) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ZeroOrOne)) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Foo , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get_all :: < dyn Bar > () . collect ())) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new (sp . get_all :: < dyn Bar > () . collect ())))) , ",
            "lifetime) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ZeroOrMore)) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Thing , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: create_new (sp . get_required :: < dyn Foo > () , sp . get :: < dyn Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: create_new (sp . get_required :: < dyn Foo > () , sp . get :: < dyn Bar > ())))) , ",
            "lifetime) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Foo > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < dyn Bar > () , di :: ServiceCardinality :: ZeroOrOne)) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < Self , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new ()) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new ()))) , ",
            "lifetime) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Foo , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get_required :: < Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new (sp . get_required :: < Bar > ())))) , ",
            "lifetime) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < Bar > () , di :: ServiceCardinality :: ExactlyOne)) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < Self , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new ()) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new ()))) , ",
            "lifetime) ",
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
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Pair < TKey , TValue > , Self > (",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (Self :: new (sp . get_required :: < TKey > () , sp . get_required :: < TValue > ())) , ",
            "| sp : & di :: ServiceProvider | di :: ServiceRef :: new (std :: sync :: Mutex :: new (Self :: new (sp . get_required :: < TKey > () , sp . get_required :: < TValue > ())))) , ",
            "lifetime) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < TKey > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < TValue > () , di :: ServiceCardinality :: ExactlyOne)) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }
}
