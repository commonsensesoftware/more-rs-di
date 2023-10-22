use crate::{KeyedServiceRef, KeyedServiceRefMut, ServiceProvider, ServiceRef, ServiceRefMut};
use spin::Once;
use std::{any::Any, sync::Mutex};

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

fn to_vec_mut<T: Any + ?Sized>(services: &ServiceProvider) -> Vec<ServiceRefMut<T>> {
    services.get_all_mut::<T>().collect()
}

fn to_keyed_vec<TKey, TSvc: Any + ?Sized>(
    services: &ServiceProvider,
) -> Vec<KeyedServiceRef<TKey, TSvc>> {
    services.get_all_by_key::<TKey, TSvc>().collect()
}

fn to_keyed_vec_mut<TKey, TSvc: Any + ?Sized>(
    services: &ServiceProvider,
) -> Vec<KeyedServiceRefMut<TKey, TSvc>> {
    services.get_all_by_key_mut::<TKey, TSvc>().collect()
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

/// Creates and returns a holder for a lazily-initialized, required, mutable service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn exactly_one_mut<T: Any + ?Sized>(services: ServiceProvider) -> Lazy<ServiceRefMut<T>> {
    Lazy::new(services, ServiceProvider::get_required_mut::<T>)
}

/// Creates and returns a holder for a lazily-initialized, keyed, required service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn exactly_one_with_key<TKey, TSvc: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<KeyedServiceRef<TKey, TSvc>> {
    Lazy::new(services, ServiceProvider::get_required_by_key::<TKey, TSvc>)
}

/// Creates and returns a holder for a lazily-initialized, keyed, required, mutable service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn exactly_one_with_key_mut<TKey, TSvc: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<KeyedServiceRefMut<TKey, TSvc>> {
    Lazy::new(
        services,
        ServiceProvider::get_required_by_key_mut::<TKey, TSvc>,
    )
}

/// Creates and returns a holder for a lazily-initialized, optional service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn zero_or_one<T: Any + ?Sized>(services: ServiceProvider) -> Lazy<Option<ServiceRef<T>>> {
    Lazy::new(services, ServiceProvider::get::<T>)
}

/// Creates and returns a holder for a lazily-initialized, optional, mutable service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn zero_or_one_mut<T: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<Option<ServiceRefMut<T>>> {
    Lazy::new(services, ServiceProvider::get_mut::<T>)
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

/// Creates and returns a holder for a lazily-initialized, keyed, optional, mutable service.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the service
#[inline]
pub fn zero_or_one_with_key_mut<TKey, TSvc: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<Option<KeyedServiceRefMut<TKey, TSvc>>> {
    Lazy::new(services, ServiceProvider::get_by_key_mut::<TKey, TSvc>)
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

/// Creates and returns a holder for multiple, lazily-initialized, mutable services.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the services
#[inline]
pub fn zero_or_more_mut<T: Any + ?Sized>(services: ServiceProvider) -> Lazy<Vec<ServiceRefMut<T>>> {
    Lazy::new(services, to_vec_mut::<T>)
}

/// Creates and returns a holder for multiple, lazily-initialized, keyed services.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the services
#[inline]
pub fn zero_or_more_with_key<TKey, TSvc: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<Vec<KeyedServiceRef<TKey, TSvc>>> {
    Lazy::new(services, to_keyed_vec::<TKey, TSvc>)
}

/// Creates and returns a holder for multiple, lazily-initialized, keyed, mutable services.
///
/// # Arguments
///
/// * `services` - The [service provider](struct.ServiceProvider.html) used to resolve the services
#[inline]
pub fn zero_or_more_with_key_mut<TKey, TSvc: Any + ?Sized>(
    services: ServiceProvider,
) -> Lazy<Vec<KeyedServiceRefMut<TKey, TSvc>>> {
    Lazy::new(services, to_keyed_vec_mut::<TKey, TSvc>)
}

/// Creates and return a holder for a lazy-initialized, optional service that is missing.
#[inline]
pub fn missing<T: Any + ?Sized>() -> Lazy<Option<ServiceRef<T>>> {
    Lazy::new(ServiceProvider::default(), ServiceProvider::get::<T>)
}

/// Creates and return a holder for a lazy-initialized, keyed, optional service that is missing.
#[inline]
pub fn missing_with_key<TKey, TSvc: Any + ?Sized>() -> Lazy<Option<KeyedServiceRef<TKey, TSvc>>> {
    Lazy::new(
        ServiceProvider::default(),
        ServiceProvider::get_by_key::<TKey, TSvc>,
    )
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

/// Creates and returns a holder from an existing instance.
pub fn init<T: Any + ?Sized>(instance: Box<T>) -> Lazy<ServiceRef<T>> {
    Lazy {
        resolve: |_| unimplemented!(),
        services: ServiceProvider::default(),
        value: Once::initialized(ServiceRef::from(instance)),
    }
}

/// Creates and returns a holder from an existing, mutable instance.
pub fn init_mut<T: Any + ?Sized>(instance: Box<Mutex<T>>) -> Lazy<ServiceRefMut<T>> {
    Lazy {
        resolve: |_| unimplemented!(),
        services: ServiceProvider::default(),
        value: Once::initialized(ServiceRefMut::from(instance)),
    }
}

/// Creates and returns a holder from an existing instance with a key.
pub fn init_with_key<TKey, TSvc: Any + ?Sized>(
    instance: Box<TSvc>,
) -> Lazy<KeyedServiceRef<TKey, TSvc>> {
    Lazy {
        resolve: |_| unimplemented!(),
        services: ServiceProvider::default(),
        value: Once::initialized(KeyedServiceRef::<TKey, TSvc>::new(ServiceRef::from(
            instance,
        ))),
    }
}

/// Creates and returns a holder from an existing, mutable instance with a key.
pub fn init_with_key_mut<TKey, TSvc: Any + ?Sized>(
    instance: Box<Mutex<TSvc>>,
) -> Lazy<KeyedServiceRefMut<TKey, TSvc>> {
    Lazy {
        resolve: |_| unimplemented!(),
        services: ServiceProvider::default(),
        value: Once::initialized(KeyedServiceRefMut::<TKey, TSvc>::new(ServiceRef::from(
            instance,
        ))),
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Mutex;

    use crate::{lazy::*, *};

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

    trait IPityTheFoo {
        fn speak(&self) -> &str;
    }

    struct FooImpl;

    impl IPityTheFoo for FooImpl {
        fn speak(&self) -> &str {
            "I pity the foo!"
        }
    }

    #[test]
    fn lazy_should_return_required_service() {
        // arrange
        let provider = ServiceCollection::new()
            .add(transient_as_self::<Bar>().from(|_| ServiceRef::new(Bar::default())))
            .add(
                transient_as_self::<Foo>()
                    .depends_on(crate::exactly_one::<Bar>())
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
                    .depends_on(crate::zero_or_one::<Bar>())
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
                    .depends_on(crate::zero_or_one::<Bar>())
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
                    .depends_on(crate::exactly_one::<Bar>())
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

    #[test]
    fn init_should_create_lazy_from_instance() {
        // arrange
        let instance: Box<dyn IPityTheFoo> = Box::new(FooImpl);

        // act
        let lazy = lazy::init(instance);

        // assert
        assert_eq!(lazy.value().speak(), "I pity the foo!");
    }

    #[test]
    fn init_with_key_should_create_lazy_from_instance() {
        // arrange
        let instance = FooImpl;

        // act
        let lazy = lazy::init_with_key::<Bar, dyn IPityTheFoo>(Box::new(instance));

        // assert
        assert_eq!(lazy.value().speak(), "I pity the foo!");
    }

    #[test]
    fn init_mut_should_create_lazy_from_instance() {
        // arrange
        let instance: Box<Mutex<dyn IPityTheFoo>> = Box::new(Mutex::new(FooImpl));

        // act
        let lazy = lazy::init_mut(instance);

        // assert
        assert_eq!(lazy.value().lock().unwrap().speak(), "I pity the foo!");
    }

    #[test]
    fn init_with_key_mut_should_create_lazy_from_instance() {
        // arrange
        let instance: Box<Mutex<dyn IPityTheFoo>> = Box::new(Mutex::new(FooImpl));

        // act
        let lazy = lazy::init_with_key_mut::<Bar, _>(instance);

        // assert
        assert_eq!(lazy.value().lock().unwrap().speak(), "I pity the foo!");
    }
}
