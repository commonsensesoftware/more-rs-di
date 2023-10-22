use crate::{
    KeyedServiceRef, KeyedServiceRefMut, ServiceDescriptor, ServiceRef, ServiceRefMut, Type,
};
use std::any::{type_name, Any};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::empty;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Mutex;

/// Represents a service provider.
#[derive(Clone)]
pub struct ServiceProvider {
    services: ServiceRef<HashMap<Type, Vec<ServiceDescriptor>>>,
}

#[cfg(feature = "async")]
unsafe impl Send for ServiceProvider {}

#[cfg(feature = "async")]
unsafe impl Sync for ServiceProvider {}

impl ServiceProvider {
    /// Initializes a new service provider.
    ///
    /// # Arguments
    ///
    /// * `services` - The map of services descriptors encapsulated by the provider.
    pub fn new(services: HashMap<Type, Vec<ServiceDescriptor>>) -> Self {
        Self {
            services: ServiceRef::new(services),
        }
    }

    /// Gets a service of the specified type.
    pub fn get<T: Any + ?Sized>(&self) -> Option<ServiceRef<T>> {
        let key = Type::of::<T>();

        if let Some(descriptors) = self.services.get(&key) {
            if let Some(descriptor) = descriptors.last() {
                return Some(
                    descriptor
                        .get(self)
                        .downcast_ref::<ServiceRef<T>>()
                        .unwrap()
                        .clone(),
                );
            }
        }

        None
    }

    /// Gets a mutable service of the specified type.
    pub fn get_mut<T: Any + ?Sized>(&self) -> Option<ServiceRefMut<T>> {
        self.get::<Mutex<T>>()
    }

    /// Gets a keyed service of the specified type.
    pub fn get_by_key<TKey, TSvc: Any + ?Sized>(&self) -> Option<KeyedServiceRef<TKey, TSvc>> {
        let key = Type::keyed::<TKey, TSvc>();

        if let Some(descriptors) = self.services.get(&key) {
            if let Some(descriptor) = descriptors.last() {
                return Some(KeyedServiceRef::new(
                    descriptor
                        .get(self)
                        .downcast_ref::<ServiceRef<TSvc>>()
                        .unwrap()
                        .clone(),
                ));
            }
        }

        None
    }

    /// Gets a keyed, mutable service of the specified type.
    pub fn get_by_key_mut<TKey, TSvc: Any + ?Sized>(
        &self,
    ) -> Option<KeyedServiceRefMut<TKey, TSvc>> {
        self.get_by_key::<TKey, Mutex<TSvc>>()
    }

    /// Gets all of the services of the specified type.
    pub fn get_all<T: Any + ?Sized>(&self) -> impl Iterator<Item = ServiceRef<T>> + '_ {
        let key = Type::of::<T>();

        if let Some(descriptors) = self.services.get(&key) {
            ServiceIterator::new(self, descriptors.iter())
        } else {
            ServiceIterator::new(self, empty())
        }
    }

    /// Gets all of the mutable services of the specified type.
    pub fn get_all_mut<T: Any + ?Sized>(&self) -> impl Iterator<Item = ServiceRefMut<T>> + '_ {
        self.get_all::<Mutex<T>>()
    }

    /// Gets all of the services of the specified key and type.
    pub fn get_all_by_key<'a, TKey: 'a, TSvc>(
        &'a self,
    ) -> impl Iterator<Item = KeyedServiceRef<TKey, TSvc>> + '_
    where
        TSvc: Any + ?Sized,
    {
        let key = Type::keyed::<TKey, TSvc>();

        if let Some(descriptors) = self.services.get(&key) {
            KeyedServiceIterator::new(self, descriptors.iter())
        } else {
            KeyedServiceIterator::new(self, empty())
        }
    }

    /// Gets all of the mutable services of the specified key and type.
    pub fn get_all_by_key_mut<'a, TKey: 'a, TSvc>(
        &'a self,
    ) -> impl Iterator<Item = KeyedServiceRefMut<TKey, TSvc>> + '_
    where
        TSvc: Any + ?Sized,
    {
        self.get_all_by_key::<TKey, Mutex<TSvc>>()
    }

    /// Gets a required service of the specified type.
    ///
    /// # Panics
    ///
    /// The requested service of type `T` does not exist.
    pub fn get_required<T: Any + ?Sized>(&self) -> ServiceRef<T> {
        if let Some(service) = self.get::<T>() {
            service
        } else {
            panic!(
                "No service for type '{}' has been registered.",
                type_name::<T>()
            );
        }
    }

    /// Gets a required, mutable service of the specified type.
    ///
    /// # Panics
    ///
    /// The requested service of type `T` does not exist.
    pub fn get_required_mut<T: Any + ?Sized>(&self) -> ServiceRefMut<T> {
        self.get_required::<Mutex<T>>()
    }

    /// Gets a required keyed service of the specified type.
    ///
    /// # Panics
    ///
    /// The requested service of type `TSvc` with key `TKey` does not exist.
    pub fn get_required_by_key<TKey, TSvc: Any + ?Sized>(&self) -> KeyedServiceRef<TKey, TSvc> {
        if let Some(service) = self.get_by_key::<TKey, TSvc>() {
            service
        } else {
            panic!(
                "No service for type '{}' with the key '{}' has been registered.",
                type_name::<TSvc>(),
                type_name::<TKey>()
            );
        }
    }

    /// Gets a required keyed service of the specified type.
    ///
    /// # Panics
    ///
    /// The requested service of type `TSvc` with key `TKey` does not exist.
    pub fn get_required_by_key_mut<TKey, TSvc: Any + ?Sized>(
        &self,
    ) -> KeyedServiceRefMut<TKey, TSvc> {
        self.get_required_by_key::<TKey, Mutex<TSvc>>()
    }

    /// Creates and returns a new service provider that is used to resolve
    /// services from a newly create scope.
    pub fn create_scope(&self) -> Self {
        Self::new(self.services.as_ref().clone())
    }
}

/// Represents a scoped service provider.
/// 
/// # Remarks
/// 
/// This struct has the exact same functionality as
/// [ServiceProvider](struct.ServiceProvider.html).
/// When a new instance is created, it also creates
/// a new scope from the source
/// [ServiceProvider](struct.ServiceProvider.html).
/// The primary use case for this struct is to
/// explicitly declare that a new scope should be
/// created at the injection call site.
#[derive(Clone, Default)]
pub struct ScopedServiceProvider {
    sp: ServiceProvider
}

impl From<&ServiceProvider> for ScopedServiceProvider {
    fn from(value: &ServiceProvider) -> Self {
        Self { sp: value.create_scope() }
    }
}

impl AsRef<ServiceProvider> for ScopedServiceProvider {
    fn as_ref(&self) -> &ServiceProvider {
        &self.sp
    }
}

impl Borrow<ServiceProvider> for ScopedServiceProvider {
    fn borrow(&self) -> &ServiceProvider {
        &self.sp
    }
}

impl Deref for ScopedServiceProvider {
    type Target = ServiceProvider;

    fn deref(&self) -> &Self::Target {
        &self.sp
    }
}

struct ServiceIterator<'a, T>
where
    T: Any + ?Sized,
{
    provider: &'a ServiceProvider,
    descriptors: Box<dyn Iterator<Item = &'a ServiceDescriptor> + 'a>,
    _marker: PhantomData<T>,
}

struct KeyedServiceIterator<'a, TKey, TSvc>
where
    TSvc: Any + ?Sized,
{
    provider: &'a ServiceProvider,
    descriptors: Box<dyn Iterator<Item = &'a ServiceDescriptor> + 'a>,
    _key: PhantomData<TKey>,
    _svc: PhantomData<TSvc>,
}

impl<'a, T: Any + ?Sized> ServiceIterator<'a, T> {
    fn new<I>(provider: &'a ServiceProvider, descriptors: I) -> Self
    where
        I: Iterator<Item = &'a ServiceDescriptor> + 'a,
    {
        Self {
            provider,
            descriptors: Box::new(descriptors),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: Any + ?Sized> Iterator for ServiceIterator<'a, T> {
    type Item = ServiceRef<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(descriptor) = self.descriptors.next() {
            Some(
                descriptor
                    .get(self.provider)
                    .downcast_ref::<ServiceRef<T>>()
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        }
    }
}

impl<'a, TKey, TSvc: Any + ?Sized> KeyedServiceIterator<'a, TKey, TSvc> {
    fn new<I>(provider: &'a ServiceProvider, descriptors: I) -> Self
    where
        I: Iterator<Item = &'a ServiceDescriptor> + 'a,
    {
        Self {
            provider,
            descriptors: Box::new(descriptors),
            _key: PhantomData,
            _svc: PhantomData,
        }
    }
}

impl<'a, TKey, TSvc: Any + ?Sized> Iterator for KeyedServiceIterator<'a, TKey, TSvc> {
    type Item = KeyedServiceRef<TKey, TSvc>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(descriptor) = self.descriptors.next() {
            Some(KeyedServiceRef::new(
                descriptor
                    .get(self.provider)
                    .downcast_ref::<ServiceRef<TSvc>>()
                    .unwrap()
                    .clone(),
            ))
        } else {
            None
        }
    }
}

impl Default for ServiceProvider {
    fn default() -> Self {
        Self {
            services: ServiceRef::new(HashMap::with_capacity(0)),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{test::*, *};
    use std::fs::remove_file;
    use std::path::{Path, PathBuf};

    #[cfg(feature = "async")]
    use std::sync::{Arc, Mutex};

    #[cfg(feature = "async")]
    use std::thread;

    #[test]
    fn get_should_return_none_when_service_is_unregistered() {
        // arrange
        let services = ServiceCollection::new().build_provider().unwrap();

        // act
        let result = services.get::<dyn TestService>();

        // assert
        assert!(result.is_none());
    }

    #[test]
    fn get_by_key_should_return_none_when_service_is_unregistered() {
        // arrange
        let services = ServiceCollection::new().build_provider().unwrap();

        // act
        let result = services.get_by_key::<key::Thingy, dyn TestService>();

        // assert
        assert!(result.is_none());
    }

    #[test]
    fn get_should_return_registered_service() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();

        // act
        let result = services.get::<dyn TestService>();

        // assert
        assert!(result.is_some());
    }

    #[test]
    fn get_by_key_should_return_registered_service() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                singleton_with_key::<key::Thingy, dyn Thing, Thing1>()
                    .from(|_| ServiceRef::new(Thing1::default())),
            )
            .add(singleton::<dyn Thing, Thing1>().from(|_| ServiceRef::new(Thing1::default())))
            .build_provider()
            .unwrap();

        // act
        let result = services.get_by_key::<key::Thingy, dyn Thing>();

        // assert
        assert!(result.is_some());
    }

    #[test]
    fn get_required_should_return_registered_service() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();

        // act
        let _ = services.get_required::<dyn TestService>();

        // assert
        // didn't panic
    }

    #[test]
    fn get_required_by_key_should_return_registered_service() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                singleton_with_key::<key::Thingy, dyn Thing, Thing3>()
                    .from(|_| ServiceRef::new(Thing3::default())),
            )
            .add(singleton::<dyn Thing, Thing1>().from(|_| ServiceRef::new(Thing1::default())))
            .build_provider()
            .unwrap();

        // act
        let thing = services.get_required_by_key::<key::Thingy, dyn Thing>();

        // assert
        assert_eq!(&thing.to_string(), "di::test::Thing3");
    }

    #[test]
    #[should_panic(
        expected = "No service for type 'dyn di::test::TestService' has been registered."
    )]
    fn get_required_should_panic_when_service_is_unregistered() {
        // arrange
        let services = ServiceCollection::new().build_provider().unwrap();

        // act
        let _ = services.get_required::<dyn TestService>();

        // assert
        // panics
    }

    #[test]
    #[should_panic(
        expected = "No service for type 'dyn di::test::Thing' with the key 'di::test::key::Thing1' has been registered."
    )]
    fn get_required_by_key_should_panic_when_service_is_unregistered() {
        // arrange
        let services = ServiceCollection::new().build_provider().unwrap();

        // act
        let _ = services.get_required_by_key::<key::Thing1, dyn Thing>();

        // assert
        // panics
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn get_should_return_same_instance_for_singleton_service() {
        // arrange
        let services = ServiceCollection::new()
            .add(existing::<dyn TestService, TestServiceImpl>(Box::new(
                TestServiceImpl::default(),
            )))
            .add(
                singleton::<dyn OtherTestService, OtherTestServiceImpl>().from(|sp| {
                    ServiceRef::new(OtherTestServiceImpl::new(
                        sp.get_required::<dyn TestService>(),
                    ))
                }),
            )
            .build_provider()
            .unwrap();

        // act
        let svc2 = services.get_required::<dyn OtherTestService>();
        let svc1 = services.get_required::<dyn OtherTestService>();

        // assert
        assert!(ServiceRef::ptr_eq(&svc1, &svc2));
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn get_should_return_different_instances_for_transient_service() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                transient::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();

        // act
        let svc1 = services.get_required::<dyn TestService>();
        let svc2 = services.get_required::<dyn TestService>();

        // assert
        assert!(!ServiceRef::ptr_eq(&svc1, &svc2));
    }

    #[test]
    fn get_all_should_return_all_services() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl { value: 1 })),
            )
            .add(
                singleton::<dyn TestService, TestService2Impl>()
                    .from(|_| ServiceRef::new(TestService2Impl { value: 2 })),
            );

        let provider = collection.build_provider().unwrap();

        // act
        let services = provider.get_all::<dyn TestService>();
        let values: Vec<_> = services.map(|s| s.value()).collect();

        // assert
        assert_eq!(&values, &[1, 2]);
    }

    #[test]
    fn get_all_by_key_should_return_all_services() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(
                singleton_with_key::<key::Thingies, dyn Thing, Thing1>()
                    .from(|_| ServiceRef::new(Thing1::default())),
            )
            .add(
                singleton_with_key::<key::Thingies, dyn Thing, Thing2>()
                    .from(|_| ServiceRef::new(Thing2::default())),
            )
            .add(
                singleton_with_key::<key::Thingies, dyn Thing, Thing3>()
                    .from(|_| ServiceRef::new(Thing3::default())),
            );

        let provider = collection.build_provider().unwrap();

        // act
        let services = provider.get_all_by_key::<key::Thingies, dyn Thing>();
        let values: Vec<_> = services.map(|s| s.to_string()).collect();

        // assert
        assert_eq!(
            &values,
            &[
                "di::test::Thing1".to_owned(),
                "di::test::Thing2".to_owned(),
                "di::test::Thing3".to_owned()
            ]
        );
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn two_scoped_service_providers_should_create_different_instances() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                scoped::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();
        let scope1 = services.create_scope();
        let scope2 = services.create_scope();

        // act
        let svc1 = scope1.get_required::<dyn TestService>();
        let svc2 = scope2.get_required::<dyn TestService>();

        // assert
        assert!(!ServiceRef::ptr_eq(&svc1, &svc2));
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn parent_child_scoped_service_providers_should_create_different_instances() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                scoped::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();
        let scope1 = services.create_scope();
        let scope2 = scope1.create_scope();

        // act
        let svc1 = scope1.get_required::<dyn TestService>();
        let svc2 = scope2.get_required::<dyn TestService>();

        // assert
        assert!(!ServiceRef::ptr_eq(&svc1, &svc2));
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn scoped_service_provider_should_have_same_singleton_when_eager_created_in_parent() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();
        let svc1 = services.get_required::<dyn TestService>();
        let scope1 = services.create_scope();
        let scope2 = scope1.create_scope();

        // act
        let svc2 = scope1.get_required::<dyn TestService>();
        let svc3 = scope2.get_required::<dyn TestService>();

        // assert
        assert!(ServiceRef::ptr_eq(&svc1, &svc2));
        assert!(ServiceRef::ptr_eq(&svc1, &svc3));
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn scoped_service_provider_should_have_same_singleton_when_lazy_created_in_parent() {
        // arrange
        let services = ServiceCollection::new()
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();
        let scope1 = services.create_scope();
        let scope2 = scope1.create_scope();
        let svc1 = services.get_required::<dyn TestService>();

        // act
        let svc2 = scope1.get_required::<dyn TestService>();
        let svc3 = scope2.get_required::<dyn TestService>();

        // assert
        assert!(ServiceRef::ptr_eq(&svc1, &svc2));
        assert!(ServiceRef::ptr_eq(&svc1, &svc3));
    }

    #[test]
    fn service_provider_should_drop_existing_as_service() {
        // arrange
        let file = new_temp_file("drop2");

        // act
        {
            let mut services = ServiceCollection::new();
            services.add(existing_as_self(Droppable::new(file.clone())));
            let _ = services.build_provider().unwrap();
        }

        // assert
        let dropped = !file.exists();
        remove_file(&file).ok();
        assert!(dropped);
    }

    #[test]
    fn service_provider_should_drop_lazy_initialized_service() {
        // arrange
        let file = new_temp_file("drop3");

        // act
        {
            let provider = ServiceCollection::new()
                .add(existing::<Path, PathBuf>(file.clone().into_boxed_path()))
                .add(singleton_as_self().from(|sp| {
                    ServiceRef::new(Droppable::new(sp.get_required::<Path>().to_path_buf()))
                }))
                .build_provider()
                .unwrap();
            let _ = provider.get_required::<Droppable>();
        }

        // assert
        let dropped = !file.exists();
        remove_file(&file).ok();
        assert!(dropped);
    }

    #[test]
    fn service_provider_should_not_drop_service_if_never_instantiated() {
        // arrange
        let file = new_temp_file("drop5");

        // act
        {
            let _ = ServiceCollection::new()
                .add(existing::<Path, PathBuf>(file.clone().into_boxed_path()))
                .add(singleton_as_self().from(|sp| {
                    ServiceRef::new(Droppable::new(sp.get_required::<Path>().to_path_buf()))
                }))
                .build_provider()
                .unwrap();
        }

        // assert
        let not_dropped = file.exists();
        remove_file(&file).ok();
        assert!(not_dropped);
    }

    #[test]
    #[allow(clippy::vtable_address_comparisons)]
    fn clone_should_be_shallow() {
        // arrange
        let provider1 = ServiceCollection::new()
            .add(
                transient::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .build_provider()
            .unwrap();

        // act
        let provider2 = provider1.clone();

        // assert
        assert!(ServiceRef::ptr_eq(&provider1.services, &provider2.services));
        assert!(std::ptr::eq(
            provider1.services.as_ref(),
            provider2.services.as_ref()
        ));
    }

    #[cfg(feature = "async")]
    #[derive(Clone)]
    struct Holder<T: Send + Sync + Clone>(T);

    #[cfg(feature = "async")]
    fn inject<V: Send + Sync + Clone>(value: V) -> Holder<V> {
        Holder(value)
    }

    #[test]
    #[cfg(feature = "async")]
    fn service_provider_should_be_async_safe() {
        // arrange
        let provider = ServiceCollection::new()
            .add(
                singleton::<dyn TestService, TestAsyncServiceImpl>()
                    .from(|_| ServiceRef::new(TestAsyncServiceImpl::default())),
            )
            .build_provider()
            .unwrap();
        let holder = inject(provider);
        let h1 = holder.clone();
        let h2 = holder.clone();
        let value = Arc::new(Mutex::new(0));
        let v1 = value.clone();
        let v2 = value.clone();

        // act
        let t1 = thread::spawn(move || {
            let service = h1.0.get_required::<dyn TestService>();
            let mut result = v1.lock().unwrap();
            *result += service.value();
        });

        let t2 = thread::spawn(move || {
            let service = h2.0.get_required::<dyn TestService>();
            let mut result = v2.lock().unwrap();
            *result += service.value();
        });

        t1.join().ok();
        t2.join().ok();

        // assert
        assert_eq!(*value.lock().unwrap(), 3);
    }
}
