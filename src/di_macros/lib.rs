mod alias;
mod internal;

extern crate proc_macro;

use crate::internal::*;
use internal::{Constructor, DeriveContext, InjectableTrait};
use proc_macro2::TokenStream;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{PathSep, Plus},
    *,
};

#[proc_macro_attribute]
pub fn inject(_metadata: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // this attribute is intentionally inert
    input
}

#[proc_macro_attribute]
pub fn injectable(metadata: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(_injectable(TokenStream::from(metadata), TokenStream::from(input)))
}

fn _injectable(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let original = input.clone();
    let result = match parse2::<InjectableAttribute>(metadata) {
        Ok(attribute) => {
            if let Ok(impl_) = parse2::<ItemImpl>(input.clone()) {
                derive_from_struct_impl(impl_, attribute, original)
            } else if let Ok(struct_) = parse2::<ItemStruct>(input) {
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
        Err(error) => error.to_compile_error(),
    }
}

fn derive_from_struct_impl(
    impl_: ItemImpl,
    attribute: InjectableAttribute,
    original: TokenStream,
) -> Result<TokenStream> {
    if let Type::Path(type_) = &*impl_.self_ty {
        let imp = &type_.path;
        let svc = service_from_attribute(imp, attribute);
        match Constructor::select(&impl_, imp) {
            Ok(method) => {
                let context = DeriveContext::for_method(&impl_.generics, imp, svc, method);
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
    let svc = service_from_attribute(imp, attribute);
    let context = DeriveContext::for_struct(&struct_.generics, imp, svc, &struct_);

    derive(context, original)
}

fn service_from_attribute(impl_: &Path, mut attribute: InjectableAttribute) -> Punctuated<Path, Plus> {
    let mut punctuated = attribute.trait_.take().unwrap_or_default();

    if punctuated.is_empty() {
        punctuated.push(impl_.clone());
    }

    punctuated
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
                    GenericParam::Lifetime(param) => GenericArgument::Lifetime(param.lifetime.clone()),
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
            original.extend(injectable);
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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new ()) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new () . into ())) , ",
            "lifetime) ",
            "} ",
            "}"
        );

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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: create ()) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: create () . into ())) , ",
            "lifetime) ",
            "} ",
            "}"
        );

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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new (sp . get_required :: < dyn Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new (sp . get_required :: < dyn Bar > ()) . into ())) , ",
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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new (sp . get :: < dyn Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new (sp . get :: < dyn Bar > ()) . into ())) , ",
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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new (sp . get_all :: < dyn Bar > () . collect ())) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new (sp . get_all :: < dyn Bar > () . collect ()) . into ())) , ",
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
                fn create_new(_foo: Ref<dyn Foo>, _bar: Option<Ref<dyn Bar>>) -> Self {
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
            "fn create_new (_foo : Ref < dyn Foo >, _bar : Option < Ref < dyn Bar >>) -> Self { ",
            "Self { } ",
            "} ",
            "} ",
            "impl di :: Injectable for ThingImpl { ",
            "fn inject (lifetime : di :: ServiceLifetime) -> di :: InjectBuilder { ",
            "di :: InjectBuilder :: new (",
            "di :: Activator :: new :: < dyn Thing , Self > (",
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: create_new (sp . get_required :: < dyn Foo > () , sp . get :: < dyn Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: create_new (sp . get_required :: < dyn Foo > () , sp . get :: < dyn Bar > ()) . into ())) , ",
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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new ()) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new () . into ())) , ",
            "lifetime) ",
            "} ",
            "}"
        );

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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new (sp . get_required :: < Bar > ())) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new (sp . get_required :: < Bar > ()) . into ())) , ",
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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new ()) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new () . into ())) , ",
            "lifetime) ",
            "} ",
            "}"
        );

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
                fn new(key: Ref<TKey>, value: Ref<TValue>) -> Self {
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
            "fn new (key : Ref < TKey >, value : Ref < TValue >) -> Self { ",
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
            "| sp : & di :: ServiceProvider | di :: Ref :: new (Self :: new (sp . get_required :: < TKey > () , sp . get_required :: < TValue > ())) , ",
            "| sp : & di :: ServiceProvider | di :: RefMut :: new (Self :: new (sp . get_required :: < TKey > () , sp . get_required :: < TValue > ()) . into ())) , ",
            "lifetime) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < TKey > () , di :: ServiceCardinality :: ExactlyOne)) ",
            ". depends_on (di :: ServiceDependency :: new (di :: Type :: of :: < TValue > () , di :: ServiceCardinality :: ExactlyOne)) ",
            "} ",
            "}");

        assert_eq!(expected, result.to_string());
    }
}
