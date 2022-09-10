use crate::*;
use di::*;

#[test]
fn inject_should_implement_trait_for_struct() {
    // arrange
    let provider = ServiceCollection::new()
        .add(traits::BarImpl::transient())
        .build_provider();

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
        .build_provider();

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
        .build_provider();

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
        .build_provider();

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
        .build_provider();
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
        .build_provider();
    let container = provider.get_required::<containers::ScopedContainer>();

    // act
    let svc1 = container.foo();
    let svc2 = provider.get_required::<dyn traits::Foo>();

    // assert
    assert!(!ServiceRef::ptr_eq(&svc1, &svc2));
}
