use crate::{fmt, validate, Ref, ServiceDescriptor, ServiceDescriptorBuilder, ServiceProvider, Type, ValidationError};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Formatter, Result as FormatResult};
use std::iter::{DoubleEndedIterator, ExactSizeIterator};
use std::ops::Index;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;

macro_rules! decorate {
    (($($traits:tt)+), ($($bounds:tt)+)) => {
        /// Decorates an existing service descriptor with a new one that wraps the original.
        ///
        /// # Arguments
        ///
        /// * `activate` - The function that will be called to decorate the resolved service instance
        ///
        /// # Remarks
        ///
        /// This function will only decorate the last registered [ServiceDescriptor] for the specified service type. If
        /// there are multiple, the others are ignored. If you need decorate all services of a particular service type,
        /// consider using [Self::decorate_all] instead. If the service to be decorated is not registered, this function
        /// does nothing. The decorator [ServiceDescriptor] is created with the same lifetime as the original. The
        /// implementation type of the decorator is determined by the generic parameter `TImpl`. If the original and
        /// decorator implementation types are the same, the original, decorated [ServiceDescriptor] is not replaced to
        /// prevent infinite recursion.
        ///
        /// # Example
        ///
        /// ```
        /// use di::{injectable, Injectable, ServiceCollection, Ref};
        ///
        /// trait Counter {
        ///     fn count(&self) -> usize;
        /// }
        ///
        /// #[injectable(Counter)]
        /// struct SingleCount;
        ///
        /// impl Counter for SingleCount {
        ///     fn count(&self) -> usize {
        ///         1
        ///     }
        /// }
        ///
        /// struct DoubleCount(Ref<dyn Counter>);
        ///
        /// impl Counter for DoubleCount {
        ///     fn count(&self) -> usize {
        ///         self.0.count() * 2
        ///     }
        /// }
        ///
        /// let provider = ServiceCollection::new()
        ///     .add(SingleCount::transient())
        ///     .decorate::<dyn Counter, DoubleCount>(|_, decorated| Ref::new(DoubleCount(decorated)))
        ///     .build_provider()
        ///     .unwrap();
        /// let counter = provider.get_required::<dyn Counter>();
        ///
        /// assert_eq!(counter.count(), 2);
        /// ```
        pub fn decorate<TSvc: ?Sized + $($traits)+, TImpl>(
            &mut self,
            activate: impl Fn(&ServiceProvider, Ref<TSvc>) -> Ref<TSvc> + $($bounds)+,
        ) -> &mut Self {
            let service_type = Type::of::<TSvc>();

            for item in self.items.iter_mut().rev() {
                if item.service_type() != service_type {
                    continue;
                }

                let impl_type = Type::of::<TImpl>();

                if item.implementation_type() == impl_type {
                    return self;
                }

                let original = item.clone();
                let builder = ServiceDescriptorBuilder::<TSvc, TImpl>::new(original.lifetime(), impl_type);

                *item = builder.from(move |sp| {
                    let decorated = original.get(sp).downcast_ref::<Ref<TSvc>>().unwrap().clone();
                    activate(sp, decorated)
                });

                break;
            }

            self
        }

        /// Decorates all existing service descriptors with a new one that wraps the original.
        ///
        /// # Arguments
        ///
        /// * `activate` - The function that will be called to decorate the resolved service instance
        ///
        /// # Remarks
        ///
        /// This function decorates all registered [ServiceDescriptor] for the specified service type. If there are none,
        /// this function does nothing. The decorator [ServiceDescriptor] is created with the same lifetime as the original.
        /// If the original, decorated [ServiceDescriptor] is the same the decorator type, it is ignored.
        ///
        /// # Example
        ///
        /// ```
        /// use di::{injectable, Injectable, ServiceCollection, Ref};
        /// use std::sync::atomic::{AtomicUsize, Ordering};
        ///
        /// trait Feature {
        ///     fn show(&self);
        /// }
        ///
        /// #[injectable(Feature)]
        /// struct Feature1;
        ///
        /// impl Feature for Feature1 {
        ///     fn show(&self) {
        ///     }
        /// }
        ///
        /// #[injectable(Feature)]
        /// struct Feature2;
        ///
        /// impl Feature for Feature2 {
        ///     fn show(&self) {
        ///     }
        /// }
        ///
        /// #[injectable]
        /// struct Tracker(AtomicUsize);
        ///
        /// impl Tracker {
        ///     fn track(&self) {
        ///         self.0.fetch_add(1, Ordering::Relaxed);
        ///     }
        ///
        ///     fn count(&self) -> usize {
        ///         self.0.load(Ordering::Relaxed)
        ///     }
        /// }
        ///
        /// struct FeatureTracker {
        ///     feature: Ref<dyn Feature>,
        ///     tracker: Ref<Tracker>,
        /// };
        ///
        /// impl Feature for FeatureTracker {
        ///     fn show(&self) {
        ///         self.tracker.track();
        ///         self.feature.show();
        ///     }
        /// }
        ///
        /// let provider = ServiceCollection::new()
        ///     .add(Tracker::singleton())
        ///     .try_add_to_all(Feature1::transient())
        ///     .try_add_to_all(Feature2::transient())
        ///     .decorate_all::<dyn Feature, FeatureTracker>(|sp, decorated| {
        ///         Ref::new(FeatureTracker { feature: decorated, tracker: sp.get_required::<Tracker>() })
        ///     })
        ///     .build_provider()
        ///     .unwrap();
        /// let features = provider.get_all::<dyn Feature>();
        /// let tracker = provider.get_required::<Tracker>();
        ///
        /// for feature in features {
        ///     feature.show();
        /// }
        ///
        /// assert_eq!(tracker.count(), 2);
        /// ```
        pub fn decorate_all<TSvc: ?Sized + $($traits)+, TImpl>(
            &mut self,
            activate: impl Fn(&ServiceProvider, Ref<TSvc>) -> Ref<TSvc> + $($bounds)+,
        ) -> &mut Self {
            let service_type = Type::of::<TSvc>();
            let func = Ref::new(activate);

            for item in self.items.iter_mut() {
                let impl_type = Type::of::<TImpl>();

                if item.service_type() != service_type || item.implementation_type() == impl_type {
                    continue;
                }

                let original = item.clone();
                let activate = func.clone();
                let builder = ServiceDescriptorBuilder::<TSvc, TImpl>::new(original.lifetime(), impl_type);

                *item = builder.from(move |sp| {
                    let decorated = original.get(sp).downcast_ref::<Ref<TSvc>>().unwrap().clone();
                    (activate)(sp, decorated)
                });
            }

            self
        }
    };
}

/// Represents a service collection.
#[derive(Default)]
pub struct ServiceCollection {
    items: Vec<ServiceDescriptor>,
}

impl ServiceCollection {
    /// Creates and returns a new instance of the service collection.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the collection contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of elements in the collection.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Removes all elements from the collection.
    #[inline]
    pub fn clear(&mut self) {
        self.items.clear()
    }

    /// Removes and returns the element at position index within the collection.
    ///
    /// # Argument
    ///
    /// * `index` - The index of the element to remove
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> ServiceDescriptor {
        self.items.remove(index)
    }

    /// Adds a service using the specified service descriptor.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [ServiceDescriptor] to register
    pub fn add<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        self.items.push(descriptor.into());
        self
    }

    /// Adds a service using the specified service descriptor if the service has not already been registered.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [ServiceDescriptor] to register
    pub fn try_add<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        let new_item = descriptor.into();
        let service_type = new_item.service_type();

        for item in &self.items {
            if item.service_type() == service_type {
                return self;
            }
        }

        self.items.push(new_item);
        self
    }

    /// Adds a service using the specified service descriptor if the service with same service and
    /// implementation type has not already been registered.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [ServiceDescriptor] to register
    pub fn try_add_to_all<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        let new_item = descriptor.into();
        let service_type = new_item.service_type();
        let implementation_type = new_item.implementation_type();

        if service_type == implementation_type {
            return self;
        }

        for item in &self.items {
            if item.service_type() == service_type && item.implementation_type() == implementation_type {
                return self;
            }
        }

        self.items.push(new_item);
        self
    }

    /// Adds the specified service descriptors if each of the services are not already registered
    /// with the same service and implementation type.
    ///
    /// # Arguments
    ///
    /// * `descriptors` - The [ServiceDescriptor] sequence to register
    pub fn try_add_all(&mut self, descriptors: impl IntoIterator<Item = ServiceDescriptor>) -> &mut Self {
        for descriptor in descriptors {
            self.try_add_to_all(descriptor);
        }
        self
    }

    /// Removes the first service descriptor with the same service type and adds the replacement.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The replacement [ServiceDescriptor]
    pub fn replace<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        let new_item = descriptor.into();
        let service_type = new_item.service_type();

        for i in 0..self.items.len() {
            if self.items[i].service_type() == service_type {
                self.items.remove(i);
                break;
            }
        }

        self.items.push(new_item);
        self
    }

    /// Adds or replaces a service with the specified descriptor if the service has not already been registered.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The replacement [ServiceDescriptor]
    #[inline]
    pub fn try_replace<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        self.try_add(descriptor)
    }

    /// Removes all specified descriptors of the specified type.
    pub fn remove_all<T: Any + ?Sized>(&mut self) -> &mut Self {
        let service_type = Type::of::<T>();

        for i in (0..self.items.len()).rev() {
            if self.items[i].service_type() == service_type {
                self.items.remove(i);
            }
        }

        self
    }

    /// Builds and returns a new [ServiceProvider].
    pub fn build_provider(&self) -> Result<ServiceProvider, ValidationError> {
        validate(self)?;

        let mut services = HashMap::with_capacity(self.items.len());

        for item in &self.items {
            let key = item.service_type().clone();
            let descriptors = services.entry(key).or_insert_with(Vec::new);

            // dependencies are only interesting for validation. after a ServiceProvider is created, no further
            // validation occurs. prevent copying unnecessary memory and allow it to potentially be freed if the
            // ServiceCollection is dropped.
            descriptors.push(item.clone_with(false));
        }

        for values in services.values_mut() {
            values.shrink_to_fit();
        }

        services.shrink_to_fit();
        Ok(ServiceProvider::new(services))
    }

    /// Gets a read-only iterator for the collection
    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ServiceDescriptor> + DoubleEndedIterator {
        self.items.iter()
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "async")] {
            decorate!((Any + Send + Sync), (Send + Sync + 'static));
        } else {
            decorate!((Any), ('static));
        }
    }
}

impl<'a> IntoIterator for &'a ServiceCollection {
    type Item = &'a ServiceDescriptor;
    type IntoIter = Iter<'a, ServiceDescriptor>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a mut ServiceCollection {
    type Item = &'a mut ServiceDescriptor;
    type IntoIter = IterMut<'a, ServiceDescriptor>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut()
    }
}

impl IntoIterator for ServiceCollection {
    type Item = ServiceDescriptor;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl Index<usize> for ServiceCollection {
    type Output = ServiceDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl std::fmt::Debug for ServiceCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        fmt::write(self, fmt::text::Renderer, f)
    }
}

impl std::fmt::Display for ServiceCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        cfg_if::cfg_if! {
            if #[cfg(feature = "fmt")] {
                if f.alternate() {
                    return fmt::write(self, fmt::terminal::Renderer, f);
                }
            }
        }

        fmt::write(self, fmt::text::Renderer, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{existing, existing_as_self, singleton, singleton_as_self, test::*, transient};
    use std::fs::remove_file;
    use std::path::{Path, PathBuf};

    #[test]
    fn is_empty_should_return_true_when_empty() {
        // arrange
        let collection = ServiceCollection::default();

        // act
        let empty = collection.is_empty();

        // assert
        assert!(empty);
    }

    #[test]
    fn length_should_return_zero_when_empty() {
        // arrange
        let collection = ServiceCollection::default();

        // act
        let length = collection.len();

        // assert
        assert_eq!(length, 0);
    }

    #[test]
    fn is_empty_should_return_false_when_not_empty() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        let not_empty = !collection.is_empty();

        // assert
        assert!(not_empty);
    }

    #[test]
    fn length_should_return_count_when_not_empty() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        let length = collection.len();

        // assert
        assert_eq!(length, 1);
    }

    #[test]
    fn clear_should_remove_all_elements() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        collection.clear();

        // assert
        assert!(collection.is_empty());
    }

    #[test]
    fn try_add_should_do_nothing_when_service_is_registered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection.add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // act
        collection
            .try_add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // assert
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn try_add_to_all_should_add_descriptor_when_implementation_is_unregistered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection.add(existing::<dyn TestService, TestServiceImpl>(Box::new(
            TestServiceImpl::default(),
        )));

        collection.try_add_to_all(
            singleton::<dyn OtherTestService, OtherTestServiceImpl>()
                .from(|sp| Ref::new(OtherTestServiceImpl::new(sp.get_required::<dyn TestService>()))),
        );

        // act
        let count = collection.len();

        // assert
        assert_eq!(count, 2);
    }

    #[test]
    fn try_add_to_all_should_not_add_descriptor_when_implementation_is_registered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection.add(existing::<dyn TestService, TestServiceImpl>(Box::new(
            TestServiceImpl::default(),
        )));

        collection.try_add_to_all(
            transient::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())),
        );

        // act
        let count = collection.len();

        // assert
        assert_eq!(count, 1);
    }

    #[test]
    fn try_add_all_should_only_add_descriptors_for_unregistered_implementations() {
        // arrange
        let descriptors = vec![
            existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default())),
            transient::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())),
        ];
        let mut collection = ServiceCollection::new();

        collection.try_add_all(descriptors.into_iter());

        // act
        let count = collection.len();

        // assert
        assert_eq!(count, 1);
    }

    #[test]
    fn replace_should_replace_first_registered_service() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())))
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // act
        collection
            .replace(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // assert
        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn remove_all_should_remove_registered_services() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())))
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // act
        collection.remove_all::<dyn TestService>();

        // assert
        assert!(collection.is_empty());
    }

    #[test]
    fn try_replace_should_do_nothing_when_service_is_registered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl { value: 1 })));

        // act
        collection.try_replace(
            singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl { value: 2 })),
        );

        // assert
        let value = collection
            .build_provider()
            .unwrap()
            .get_required::<dyn TestService>()
            .value();
        assert_eq!(value, 1);
    }

    #[test]
    fn remove_should_remove_element_at_index() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        let _ = collection.remove(0);

        // assert
        assert!(collection.is_empty());
    }

    #[test]
    fn service_collection_should_drop_existing_as_service() {
        // arrange
        let file = new_temp_file("drop1");

        // act
        {
            let mut services = ServiceCollection::new();
            services.add(existing_as_self(Droppable::new(file.clone())));
        }

        // assert
        let dropped = !file.exists();
        remove_file(&file).ok();
        assert!(dropped);
    }

    #[test]
    fn service_collection_should_not_drop_service_if_never_instantiated() {
        // arrange
        let file = new_temp_file("drop4");
        let mut services = ServiceCollection::new();

        // act
        {
            services
                .add(existing::<Path, PathBuf>(file.clone().into_boxed_path()))
                .add(singleton_as_self().from(|sp| Ref::new(Droppable::new(sp.get_required::<Path>().to_path_buf()))));
        }

        // assert
        let not_dropped = file.exists();
        remove_file(&file).ok();
        assert!(not_dropped);
    }
}
