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

/// Initializes a new transient service descriptor builder.
#[inline]
pub fn transient<TSvc: Any + ?Sized, TImpl: 'static>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
    ServiceDescriptorBuilder::new(ServiceLifetime::Transient, Type::of::<TImpl>())
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

/// Initializes a new transient service descriptor builder.
///
/// # Remarks
///
/// This function maps a concrete type to itself rather than a trait
#[inline]
pub fn transient_as_self<T: Any>() -> ServiceDescriptorBuilder<T, T> {
    ServiceDescriptorBuilder::new(ServiceLifetime::Transient, Type::of::<T>())
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
        Once::initialized(ServiceRef::new(ServiceRef::from(instance))),
        ServiceRef::new(no_op),
    )
}
