use crate::{ServiceProvider, ServiceRef, KeyedServiceRef};
use spin::Once;
use std::any::Any;

/// Represents a holder for lazily-initialized service resolution.
pub struct Lazy<T> {
    services: ServiceProvider,
    resolve: fn(&ServiceProvider) -> T,
    value: Once<T>,
}

impl<T> Lazy<T> {
    fn new(services: ServiceProvider, resolve: fn(&ServiceProvider) -> T) -> Self {
        Self {
            services,
            resolve,
            value: Once::new(),
        }
    }

    /// Resolves and returns a reference to the underlying, lazy-initialized service.
    pub fn value(&self) -> &T {
        self.value.call_once(|| (self.resolve)(&self.services))
    }
}

fn to_vec<T: Any + ?Sized>(services: &ServiceProvider) -> Vec<ServiceRef<T>> {
    services.get_all::<T>().collect()
}

fn to_keyed_vec<TKey, TSvc: Any + ?Sized>(services: &ServiceProvider) -> Vec<KeyedServiceRef<TKey, TSvc>> {
    services.get_all_by_key::<TKey, TSvc>().collect()
}

/// Creates and returns a holder for a lazily-initialized, required service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn exactly_one<T: Any + ?Sized>(services: ServiceProvider) -> Lazy<ServiceRef<T>> {
    Lazy::new(services, ServiceProvider::get_required::<T>)
}

/// Creates and returns a holder for a lazily-initialized, keyed, required service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn exactly_one_with_key<TKey, TSvc: Any + ?Sized>(services: ServiceProvider) -> Lazy<KeyedServiceRef<TKey, TSvc>> {
    Lazy::new(services, ServiceProvider::get_required_by_key::<TKey, TSvc>)
}

/// Creates and returns a holder for a lazily-initialized, optional service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn zero_or_one<T: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<Option<ServiceRef<T>>> {
    Lazy::new(services, ServiceProvider::get::<T>)
}

/// Creates and returns a holder for a lazily-initialized, keyed, optional service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn zero_or_one_with_key<TKey, TSvc: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<Option<KeyedServiceRef<TKey, TSvc>>> {
    Lazy::new(services, ServiceProvider::get_by_key::<TKey, TSvc>)
}

/// Creates and returns a holder for multiple, lazily-initialized services.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the services
#[inline]
pub fn zero_or_more<T: Any + ?Sized>(services: ServiceProvider) -> Lazy<Vec<ServiceRef<T>>> {
    Lazy::new(services, to_vec::<T>)
}

/// Creates and returns a holder for multiple, lazily-initialized, keyed services.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the services
#[inline]
pub fn zero_or_more_with_key<TKey, TSvc: Any + ?Sized>(services: ServiceProvider) -> Lazy<Vec<KeyedServiceRef<TKey, TSvc>>> {
    Lazy::new(services, to_keyed_vec::<TKey, TSvc>)
}

/// Creates and return a holder for a lazy-initialized, optional service that is missing.
#[inline]
pub fn missing<T: Any + ?Sized>() -> Lazy<Option<ServiceRef<T>>> {
    Lazy::new(ServiceProvider::default(), ServiceProvider::get::<T>)
}

/// Creates and return a holder for a lazy-initialized, keyed, optional service that is missing.
#[inline]
pub fn missing_with_key<TKey, TSvc: Any + ?Sized>() -> Lazy<Option<KeyedServiceRef<TKey, TSvc>>> {
    Lazy::new(ServiceProvider::default(), ServiceProvider::get_by_key::<TKey, TSvc>)
}

/// Creates and return a holder for any empty collection of lazy-initialized services.
#[inline]
pub fn empty<T: Any + ?Sized>() -> Lazy<Vec<ServiceRef<T>>> {
    Lazy::new(ServiceProvider::default(), to_vec::<T>)
}

/// Creates and return a holder for any empty collection of lazy-initialized, keyed services.
#[inline]
pub fn empty_with_key<TKey, TSvc: Any + ?Sized>() -> Lazy<Vec<KeyedServiceRef<TKey, TSvc>>> {
    Lazy::new(ServiceProvider::default(), to_keyed_vec::<TKey, TSvc>)
}

#[cfg(test)]
mod tests {

    use crate::lazy::{self, Lazy};
    use crate::*;

    #[derive(Default)]
    struct Bar;

    struct Foo {
        bar: Lazy<ServiceRef<Bar>>,
    }

    struct Foo2 {
        bar: Lazy<Option<ServiceRef<Bar>>>,
    }

    impl Bar {
        fn echo(&self) -> &str {
            "Delayed!"
        }
    }

    impl Foo {
        fn new(bar: Lazy<ServiceRef<Bar>>) -> Self {
            Self { bar }
        }

        fn echo(&self) -> &str {
            self.bar.value().echo()
        }
    }

    impl Foo2 {
        fn new(bar: Lazy<Option<ServiceRef<Bar>>>) -> Self {
            Self { bar }
        }

        fn echo(&self) -> Option<&str> {
            match self.bar.value() {
                Some(bar) => Some(bar.echo()),
                _ => None,
            }
        }
    }

    #[test]
    fn lazy_should_return_required_service() {
        // arrange
        let provider = ServiceCollection::new()
            .add(transient_as_self::<Bar>().from(|_| ServiceRef::new(Bar::default())))
            .add(
                transient_as_self::<Foo>()
                    .depends_on(exactly_one::<Bar>())
                    .from(|sp| ServiceRef::new(Foo::new(lazy::exactly_one::<Bar>(sp.clone())))),
            )
            .build_provider()
            .unwrap();

        // act
        let foo = provider.get_required::<Foo>();

        // assert
        assert_eq!("Delayed!", foo.echo());
    }

    #[test]
    fn lazy_should_return_optional_service() {
        // arrange
        let provider = ServiceCollection::new()
            .add(transient_as_self::<Bar>().from(|_| ServiceRef::new(Bar::default())))
            .add(
                transient_as_self::<Foo2>()
                    .depends_on(zero_or_one::<Bar>())
                    .from(|sp| ServiceRef::new(Foo2::new(lazy::zero_or_one::<Bar>(sp.clone())))),
            )
            .build_provider()
            .unwrap();

        // act
        let foo = provider.get_required::<Foo2>();

        // assert
        assert_eq!("Delayed!", foo.echo().unwrap());
    }

    #[test]
    fn lazy_should_allow_missing_optional_service() {
        // arrange
        let provider = ServiceCollection::new()
            .add(
                transient_as_self::<Foo2>()
                    .depends_on(zero_or_one::<Bar>())
                    .from(|sp| ServiceRef::new(Foo2::new(lazy::zero_or_one::<Bar>(sp.clone())))),
            )
            .build_provider()
            .unwrap();

        // act
        let foo = provider.get_required::<Foo2>();

        // assert
        assert_eq!(None, foo.echo());
    }

    #[test]
    fn missing_should_initialize_lazy() {
        // arrange
        let lazy = lazy::missing::<Bar>();

        // act
        let value = lazy.value();

        // assert
        assert!(value.is_none());
    }

    #[test]
    fn empty_should_initialize_lazy() {
        // arrange
        let lazy = lazy::empty::<Bar>();

        // act
        let value = lazy.value();

        // assert
        assert!(value.is_empty());
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn lazy_should_return_same_scoped_service() {
        // arrange
        let provider = ServiceCollection::new()
            .add(scoped_factory(|_| ServiceRef::new(Bar::default())))
            .add(
                transient_as_self::<Foo>()
                    .depends_on(exactly_one::<Bar>())
                    .from(|sp| ServiceRef::new(Foo::new(lazy::exactly_one::<Bar>(sp.clone())))),
            )
            .build_provider()
            .unwrap();
        
        // act
        let foo = provider.get_required::<Foo>();
        let bar1 = provider.get_required::<Bar>();
        let bar2 = provider.clone().get_required::<Bar>();

        // assert
        assert!(ServiceRef::ptr_eq(foo.bar.value(), &bar1));
        assert!(ServiceRef::ptr_eq(&bar1, &bar2));
    }
}
