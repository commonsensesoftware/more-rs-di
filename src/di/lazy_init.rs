use crate::{ServiceProvider, ServiceRef};
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

/// Creates and returns a holder for a lazily-initialized, required service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn exactly_one<T: Any + ?Sized>(services: ServiceProvider) -> Lazy<ServiceRef<T>> {
    Lazy::new(services, ServiceProvider::get_required::<T>)
}

/// Creates and returns a holder for a lazily-initialized, optional service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn zero_or_one<TSvc: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<Option<ServiceRef<TSvc>>> {
    Lazy::new(services, ServiceProvider::get::<TSvc>)
}

/// Creates and returns a holder for multiple lazily-initialized services.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the services
#[inline]
pub fn zero_or_more<T: Any + ?Sized>(services: ServiceProvider) -> Lazy<Vec<ServiceRef<T>>> {
    Lazy::new(services, to_vec::<T>)
}

/// Creates and return a holder for a lazy-initialized, optional service that is missing.
#[inline]
pub fn missing<T: Any + ?Sized>() -> Lazy<Option<ServiceRef<T>>> {
    Lazy::new(ServiceProvider::default(), ServiceProvider::get::<T>)
}

/// Creates and return a holder for any empty collection of lazy-initialized services.
#[inline]
pub fn empty<T: Any + ?Sized>() -> Lazy<Vec<ServiceRef<T>>> {
    Lazy::new(ServiceProvider::default(), to_vec::<T>)
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
}
