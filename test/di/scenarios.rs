use crate::{traits::*, *};
use di::*;

#[test]
fn inject_should_implement_trait_for_struct() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::BarImpl::transient())
        .build_provider()
        .unwrap();

    // act
    let bar = provider.get_required::<dyn traits::Bar>();

    // assert
    assert_eq!("Success!", bar.echo());
}

#[test]
fn inject_should_implement_trait_for_struct_with_dependency() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::FooImpl::singleton())
        .add(traits::BarImpl::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<dyn traits::Foo>();

    // assert
    assert_eq!("Success!", foo.echo());
}

#[test]
fn inject_should_implement_struct_for_self() {
    // arrange
    let provider = ServiceCollection::new()
        .add(structs::Bar::transient())
        .build_provider()
        .unwrap();

    // act
    let bar = provider.get_required::<structs::Bar>();

    // assert
    assert_eq!("Success!", bar.echo());
}

#[test]
fn inject_should_implement_struct_for_self_with_dependency() {
    // arrange
    let provider = ServiceCollection::new()
        .add(structs::Foo::singleton())
        .add(structs::Bar::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<structs::Foo>();

    // assert
    assert_eq!("Success!", foo.echo());
}

#[test]
#[allow(clippy::vtable_address_comparisons)]
fn inject_should_clone_service_provider_and_return_same_singleton() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::FooImpl::singleton())
        .add(traits::BarImpl::transient())
        .add(containers::Container::transient())
        .build_provider()
        .unwrap();
    let container = provider.get_required::<containers::Container>();

    // act
    let svc1 = container.foo();
    let svc2 = provider.get_required::<dyn traits::Foo>();

    // assert
    assert!(ServiceRef::ptr_eq(&svc1, &svc2));
}

#[test]
#[allow(clippy::vtable_address_comparisons)]
fn inject_should_clone_service_provider_and_return_different_scoped_instance() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::FooImpl::scoped())
        .add(traits::BarImpl::transient())
        .add(containers::ScopedContainer::transient())
        .build_provider()
        .unwrap();
    let container = provider.get_required::<containers::ScopedContainer>();

    // act
    let svc1 = container.foo();
    let svc2 = provider.get_required::<dyn traits::Foo>();

    // assert
    assert!(!ServiceRef::ptr_eq(&svc1, &svc2));
}

#[test]
fn inject_should_add_dependencies_for_validation() {
    // arrange
    let mut services = ServiceCollection::new();

    services.add(traits::FooImpl::transient());

    // act
    let result = services.build_provider();

    // assert
    assert!(result.is_err());
}

#[test]
fn inject_should_implement_generic_struct_with_dependency() {
    // arrange
    let provider = ServiceCollection::new()
        .add(structs::GenericFoo::<u8>::singleton())
        .add(structs::GenericBar::<u8>::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<structs::GenericFoo<u8>>();

    // assert
    assert_eq!(u8::default(), foo.echo());
}

#[test]
fn inject_should_implement_generic_trait_for_generic_struct() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::PairImpl::<u8, u8>::transient())
        .build_provider()
        .unwrap();

    // act
    let pair = provider.get_required::<dyn traits::Pair<u8, u8>>();

    // assert
    assert_eq!(&u8::default(), pair.key());
    assert_eq!(&u8::default(), pair.value());
}

#[test]
fn inject_should_implement_lazy_struct() {
    // arrange
    let provider = ServiceCollection::new()
        .add(structs::Bar::transient())
        .add(structs::LazyFoo::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<structs::LazyFoo>();

    // assert
    assert_eq!("Success!", foo.echo())
}

#[test]
fn inject_should_implement_required_lazy_trait() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::BarImpl::transient())
        .add(traits::OneLazyFoo::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<dyn traits::Foo>();

    // assert
    assert_eq!("Success!", foo.echo())
}

#[test]
fn inject_should_implement_optional_lazy_trait() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::BarImpl::transient())
        .add(traits::MaybeLazyFoo::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<dyn traits::Foo>();

    // assert
    assert_eq!("Success!", foo.echo())
}

#[test]
fn inject_should_handle_implement_optional_lazy_trait() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::MaybeLazyFoo::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<dyn traits::Foo>();

    // assert
    assert_eq!("", foo.echo())
}

#[test]
fn inject_should_implement_many_lazy_trait() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::BarImpl::transient())
        .add(traits::ManyLazyFoo::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<dyn traits::Foo>();

    // assert
    assert_eq!("Success!", foo.echo())
}

#[test]
fn inject_should_implemented_keyed_dependencies() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::Thing1::transient().with_key::<key::Thing1>())
        .add(traits::Thing2::transient().with_key::<key::Thing2>())
        .add(traits::CatInTheHat::transient())
        .build_provider()
        .unwrap();

    // act
    let cat_in_the_hat = provider.get_required::<CatInTheHat>();

    // assert
    assert_eq!(
        &cat_in_the_hat.thing1.to_string(),
        "more_di_tests::traits::Thing1"
    );
    assert_eq!(
        &cat_in_the_hat.thing2.to_string(),
        "more_di_tests::traits::Thing2"
    );
}

#[test]
fn inject_should_implement_iterator_argument() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::Thing1::transient())
        .add(traits::Thing2::transient())
        .add(traits::Thingies::transient())
        .build_provider()
        .unwrap();

    // act
    let thingies = provider.get_required::<Thingies>();

    // assert
    assert_eq!(thingies.count(), 2);
}

#[test]
fn inject_should_implement_trait_for_unit_struct() {
    // arrange
    let provider = ServiceCollection::new()
        .add(structs::UnitStruct::transient())
        .build_provider()
        .unwrap();

    // act
    let unit = provider.get_required::<structs::UnitStruct>();

    // assert
    assert_eq!(unit.echo(), "Hello world!");
}

#[test]
fn service_descriptor_should_exclude_duplicate_dependencies() {
    // arrange

    // act
    let descriptor = structs::NormalStruct::transient().build();

    // assert
    assert_eq!(descriptor.dependencies().len(), 1);
}

#[test]
fn inject_should_implement_struct_definition() {
    // arrange
    let provider = ServiceCollection::new()
        .add(structs::UnitStruct::transient())
        .add(structs::NormalStruct::singleton())
        .build_provider()
        .unwrap();

    // act
    let struct_ = provider.get_required::<structs::NormalStruct>();

    // assert
    assert_eq!(struct_.count, 0);
    assert_eq!(struct_.lazy.value().echo(), "Hello world!");
}

#[test]
fn inject_should_implement_tuple_struct() {
    // arrange
    let provider = ServiceCollection::new()
        .add(structs::UnitStruct::transient())
        .add(structs::Record::singleton())
        .build_provider()
        .unwrap();

    // act
    let record = provider.get_required::<structs::Record>();

    // assert
    assert_eq!(record.2, 0);
    assert_eq!(record.1.value().echo(), "Hello world!");
}

#[test]
fn inject_should_implement_trait_for_struct_definition() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::BarImpl::transient())
        .add(traits::FooToo::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<dyn Foo>();

    // assert
    assert_eq!(foo.echo(), "Success!");
}

#[test]
fn inject_should_implement_trait_for_tuple_struct() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::BarImpl::transient())
        .add(traits::FooTwo::transient())
        .build_provider()
        .unwrap();

    // act
    let foo = provider.get_required::<dyn Foo>();

    // assert
    assert_eq!(foo.echo(), "Success!");
}

#[test]
fn inject_should_implement_many_for_struct_field() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::Thing1::transient())
        .add(traits::Thing2::transient())
        .add(traits::MoreThingies::transient())
        .build_provider()
        .unwrap();

    // act
    let thingies = provider.get_required::<MoreThingies>();

    // assert
    assert_eq!(thingies.count(), 2);
}

#[test]
fn inject_should_resolve_keyed() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::Thing1::transient().with_key::<key::Thing1>())
        .build_provider()
        .unwrap();

    // act
    let _ = provider.get_required_by_key::<key::Thing1, dyn Thing>();

    // assert
    // no panic!
}

#[test]
fn inject_should_resolve_mut() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::Thing1::transient().as_mut())
        .build_provider()
        .unwrap();

    // act
    let _ = provider.get_required_mut::<dyn Thing>();

    // assert
    // no panic!
}

#[test]
fn inject_should_resolve_keyed_mut() {
    // arrange
    let provider = ServiceCollection::new()
        .add(
            traits::Thing1::transient()
                .with_key::<key::Thing2>()
                .as_mut(),
        )
        .build_provider()
        .unwrap();

    // act
    let _ = provider.get_required_by_key_mut::<key::Thing2, dyn Thing>();

    // assert
    // no panic!
}

#[test]
fn inject_should_support_multiple_traits() {
    // arrange
    let provider = ServiceCollection::new()
        .add(MultiService::singleton())
        .add(transient_factory::<dyn Service1>(|sp| {
            sp.get_required::<MultiService>()
        }))
        .add(transient_factory::<dyn Service2>(|sp| {
            sp.get_required::<MultiService>()
        }))
        .build_provider()
        .unwrap();

    // act
    let _svc1 = provider.get_required::<dyn Service1>();
    let _svc2 = provider.get_required::<dyn Service2>();

    // assert
    // no panic!
}
