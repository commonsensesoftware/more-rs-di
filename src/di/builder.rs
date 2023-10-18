use crate::*;
use spin::Once;
use std::any::Any;
use std::mem::MaybeUninit;

#[inline(always)]
fn no_op(_services: &ServiceProvider) -> ServiceRef<dyn Any> {
    ServiceRef::new(MaybeUninit::<Box<dyn Any>>::uninit())
}

/// Initializes a new singleton service descriptor builder.
#[inline]
pub fn singleton<TSvc: Any + ?Sized, TImpl: 'static>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
    ServiceDescriptorBuilder::new(ServiceLifetime::Singleton, Type::of::<TImpl>())
}

/// Initializes a new keyed singleton service descriptor builder.
#[inline]
pub fn keyed_singleton<TKey, TSvc, TImpl: 'static>() -> ServiceDescriptorBuilder<TSvc, TImpl>
where
    TSvc: Any + ?Sized,
{
    ServiceDescriptorBuilder::keyed::<TKey>(ServiceLifetime::Singleton, Type::of::<TImpl>())
}

/// Initializes a new singleton service descriptor.
///
/// # Arguments
///
/// * `factory` - The factory method used to create the service
#[inline]
pub fn singleton_factory<T: Any + ?Sized, F>(factory: F) -> ServiceDescriptor
where
    F: Fn(&ServiceProvider) -> ServiceRef<T> + 'static,
{
    ServiceDescriptorBuilder::<T, F>::new(ServiceLifetime::Singleton, Type::of::<F>()).from(factory)
}

/// Initializes a new keyed singleton service descriptor.
///
/// # Arguments
///
/// * `factory` - The factory method used to create the service
#[inline]
pub fn keyed_singleton_factory<TKey, TSvc, F>(factory: F) -> ServiceDescriptor
where
    TSvc: Any + ?Sized,
    F: Fn(&ServiceProvider) -> ServiceRef<TSvc> + 'static,
{
    let builder = ServiceDescriptorBuilder::<TSvc, F>::keyed::<TKey>(
        ServiceLifetime::Singleton,
        Type::of::<F>(),
    );

    builder.from(factory)
}

/// Initializes a new singleton service descriptor builder.
///
/// # Remarks
///
/// This function maps a concrete type to itself rather than a trait
#[inline]
pub fn singleton_as_self<T: Any>() -> ServiceDescriptorBuilder<T, T> {
    ServiceDescriptorBuilder::new(ServiceLifetime::Singleton, Type::of::<T>())
}

/// Initializes a new scoped service descriptor builder.
#[inline]
pub fn scoped<TSvc: Any + ?Sized, TImpl: 'static>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
    ServiceDescriptorBuilder::new(ServiceLifetime::Scoped, Type::of::<TImpl>())
}

/// Initializes a new scoped keyed service descriptor builder.
#[inline]
pub fn keyed_scoped<TKey, TSvc, TImpl: 'static>() -> ServiceDescriptorBuilder<TSvc, TImpl>
where
    TSvc: Any + ?Sized,
{
    ServiceDescriptorBuilder::keyed::<TKey>(ServiceLifetime::Scoped, Type::of::<TImpl>())
}

/// Initializes a new scoped service descriptor.
///
/// # Arguments
///
/// * `factory` - The factory method used to create the service
#[inline]
pub fn scoped_factory<T: Any + ?Sized, F>(factory: F) -> ServiceDescriptor
where
    F: Fn(&ServiceProvider) -> ServiceRef<T> + 'static,
{
    ServiceDescriptorBuilder::<T, F>::new(ServiceLifetime::Scoped, Type::of::<F>()).from(factory)
}

/// Initializes a new keyed scoped service descriptor.
///
/// # Arguments
///
/// * `factory` - The factory method used to create the service
#[inline]
pub fn keyed_scoped_factory<TKey, TSvc, F>(factory: F) -> ServiceDescriptor
where
    TSvc: Any + ?Sized,
    F: Fn(&ServiceProvider) -> ServiceRef<TSvc> + 'static,
{
    let builder = ServiceDescriptorBuilder::<TSvc, F>::keyed::<TKey>(
        ServiceLifetime::Scoped,
        Type::of::<F>(),
    );
    builder.from(factory)
}

/// Initializes a new transient service descriptor builder.
#[inline]
pub fn transient<TSvc: Any + ?Sized, TImpl: 'static>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
    ServiceDescriptorBuilder::new(ServiceLifetime::Transient, Type::of::<TImpl>())
}

/// Initializes a new keyed transient service descriptor builder.
#[inline]
pub fn keyed_transient<TKey, TSvc: Any + ?Sized, TImpl: 'static>(
) -> ServiceDescriptorBuilder<TSvc, TImpl> {
    ServiceDescriptorBuilder::keyed::<TKey>(ServiceLifetime::Transient, Type::of::<TImpl>())
}

/// Initializes a new transient service descriptor.
///
/// # Arguments
///
/// * `factory` - The factory method used to create the service
#[inline]
pub fn transient_factory<T: Any + ?Sized, F>(factory: F) -> ServiceDescriptor
where
    F: Fn(&ServiceProvider) -> ServiceRef<T> + 'static,
{
    ServiceDescriptorBuilder::<T, F>::new(ServiceLifetime::Transient, Type::of::<F>()).from(factory)
}

/// Initializes a new keyed transient service descriptor.
///
/// # Arguments
///
/// * `factory` - The factory method used to create the service
#[inline]
pub fn keyed_transient_factory<TKey, TSvc, F>(factory: F) -> ServiceDescriptor
where
    TSvc: Any + ?Sized,
    F: Fn(&ServiceProvider) -> ServiceRef<TSvc> + 'static,
{
    let builder = ServiceDescriptorBuilder::<TSvc, F>::keyed::<TKey>(
        ServiceLifetime::Transient,
        Type::of::<F>(),
    );
    builder.from(factory)
}

/// Initializes a new transient service descriptor builder.
///
/// # Remarks
///
/// This function maps a concrete type to itself rather than a trait
#[inline]
pub fn transient_as_self<T: Any>() -> ServiceDescriptorBuilder<T, T> {
    ServiceDescriptorBuilder::new(ServiceLifetime::Transient, Type::of::<T>())
}

/// Initializes a new transient keyed service descriptor builder.
///
/// # Remarks
///
/// This function maps a concrete type to itself rather than a trait
#[inline]
pub fn keyed_transient_as_self<TKey, TSvc: Any>() -> ServiceDescriptorBuilder<TSvc, TSvc> {
    ServiceDescriptorBuilder::keyed::<TKey>(ServiceLifetime::Transient, Type::of::<TSvc>())
}

/// Creates a new singleton service descriptor for an existing service instance.
///
/// # Arguments
///
/// * `instance` - The existing service instance
///
/// # Remarks
///
/// This function maps an existing instance to a trait
#[inline]
pub fn existing<TSvc: Any + ?Sized, TImpl: 'static>(instance: Box<TSvc>) -> ServiceDescriptor {
    ServiceDescriptor::new(
        ServiceLifetime::Singleton,
        Type::of::<TSvc>(),
        Type::of::<TImpl>(),
        Vec::with_capacity(0),
        Once::initialized(ServiceRef::new(ServiceRef::<TSvc>::from(instance))),
        ServiceRef::new(no_op),
    )
}

/// Creates a new singleton service descriptor for an existing service instance.
///
/// # Arguments
///
/// * `instance` - The existing service instance
///
/// # Remarks
///
/// This function maps an existing instance to itself rather than a trait
#[inline]
pub fn existing_as_self<T: Any>(instance: T) -> ServiceDescriptor {
    ServiceDescriptor::new(
        ServiceLifetime::Singleton,
        Type::of::<T>(),
        Type::of::<T>(),
        Vec::with_capacity(0),
        Once::initialized(ServiceRef::new(ServiceRef::from(instance))),
        ServiceRef::new(no_op),
    )
}

/// Creates a new singleton service descriptor for an existing service instance with a key.
///
/// # Arguments
///
/// * `instance` - The existing service instance
///
/// # Remarks
///
/// This function maps an existing instance to a trait
#[inline]
pub fn existing_with_key<TKey, TSvc: Any + ?Sized, TImpl: 'static>(
    instance: Box<TSvc>,
) -> ServiceDescriptor {
    ServiceDescriptor::new(
        ServiceLifetime::Singleton,
        Type::keyed::<TKey, TSvc>(),
        Type::of::<TImpl>(),
        Vec::with_capacity(0),
        Once::initialized(ServiceRef::new(ServiceRef::<TSvc>::from(instance))),
        ServiceRef::new(no_op),
    )
}

/// Creates a new singleton service descriptor for an existing service instance with a key.
///
/// # Arguments
///
/// * `instance` - The existing service instance
///
/// # Remarks
///
/// This function maps an existing instance to itself rather than a trait
#[inline]
pub fn existing_with_key_as_self<TKey, TSvc: Any>(instance: TSvc) -> ServiceDescriptor {
    ServiceDescriptor::new(
        ServiceLifetime::Singleton,
        Type::keyed::<TKey, TSvc>(),
        Type::of::<TSvc>(),
        Vec::with_capacity(0),
        Once::initialized(ServiceRef::new(ServiceRef::from(instance))),
        ServiceRef::new(no_op),
    )
}

/// Creates a new service dependency with a cardinality of exactly one (1:1).
#[inline]
pub fn exactly_one<T: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::of::<T>(), ServiceCardinality::ExactlyOne)
}

/// Creates a new keyed service dependency with a cardinality of exactly one (1:1).
#[inline]
pub fn exactly_one_with_key<TKey, TSvc: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::keyed::<TKey, TSvc>(), ServiceCardinality::ExactlyOne)
}

/// Creates a new service dependency with a cardinality of zero or one (0:1).
#[inline]
pub fn zero_or_one<T: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::of::<T>(), ServiceCardinality::ZeroOrOne)
}

/// Creates a new keyed service dependency with a cardinality of zero or one (0:1).
#[inline]
pub fn zero_or_one_with_key<TKey, TSvc: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::keyed::<TKey, TSvc>(), ServiceCardinality::ZeroOrOne)
}

/// Creates a new service dependency with a cardinality of zero or more (0:*).
#[inline]
pub fn zero_or_more<T: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::of::<T>(), ServiceCardinality::ZeroOrMore)
}

/// Creates a new keyed service dependency with a cardinality of zero or more (0:*).
#[inline]
pub fn zero_or_more_with_key<TKey, TSvc: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::keyed::<TKey, TSvc>(), ServiceCardinality::ZeroOrMore)
}
