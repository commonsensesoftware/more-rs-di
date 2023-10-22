use crate::{validate, ServiceDescriptor, ServiceProvider, Type, ValidationError};
use std::any::Any;
use std::collections::HashMap;
use std::iter::{DoubleEndedIterator, ExactSizeIterator};
use std::ops::Index;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;

/// Represents a service collection.
#[derive(Default)]
pub struct ServiceCollection {
    items: Vec<ServiceDescriptor>,
}

impl ServiceCollection {
    /// Creates and returns a new instance of the service collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of elements in the collection.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Removes all elements from the collection.
    pub fn clear(&mut self) {
        self.items.clear()
    }

    /// Removes and returns the element at position index within the collection.
    ///
    /// # Argument
    ///
    /// * `index` - The index of the element to remove.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> ServiceDescriptor {
        self.items.remove(index)
    }

    /// Adds a service using the specified service descriptor.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [service descriptor](struct.ServiceDescriptor.html) to register.
    pub fn add<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        self.items.push(descriptor.into());
        self
    }

    /// Adds a service using the specified service descriptor if the service has not already been registered.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [service descriptor](struct.ServiceDescriptor.html) to register.
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
    /// * `descriptor` - The [service descriptor](struct.ServiceDescriptor.html) to register.
    pub fn try_add_to_all<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        let new_item = descriptor.into();
        let service_type = new_item.service_type();
        let implementation_type = new_item.implementation_type();

        if service_type == implementation_type {
            return self;
        }

        for item in &self.items {
            if item.service_type() == service_type
                && item.implementation_type() == implementation_type
            {
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
    /// * `descriptors` - The [service descriptors](struct.ServiceDescriptor.html) to register.
    pub fn try_add_all(
        &mut self,
        descriptors: impl IntoIterator<Item = ServiceDescriptor>,
    ) -> &mut Self {
        for descriptor in descriptors {
            self.try_add_to_all(descriptor);
        }
        self
    }

    /// Removes the first service descriptor with the same service type and adds the replacement.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The replacement [service descriptor](struct.ServiceDescriptor.html).
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
    /// * `descriptor` - The replacement [service descriptor](struct.ServiceDescriptor.html).
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

    /// Builds and returns a new [service provider](struct.ServiceProvider.html).
    pub fn build_provider(&self) -> Result<ServiceProvider, ValidationError> {
        if let Err(error) = validate(self) {
            return Err(error);
        }

        let mut services = HashMap::with_capacity(self.items.len());

        for item in &self.items {
            let key = item.service_type().clone();
            let descriptors = services.entry(key).or_insert_with(Vec::new);

            // note: dependencies are only interesting for validation. after a ServiceProvider
            // is created, no further validation occurs. prevent copying unnecessary memory
            // and allow it to potentially be freed if the ServiceCollection is dropped.
            descriptors.push(item.clone_with(false));
        }

        for values in services.values_mut() {
            values.shrink_to_fit();
        }

        services.shrink_to_fit();
        Ok(ServiceProvider::new(services))
    }

    /// Gets a read-only iterator for the collection
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &ServiceDescriptor> + ExactSizeIterator + DoubleEndedIterator {
        self.items.iter()
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{test::*, *};
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
        let descriptor =
            existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
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
        let descriptor =
            existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
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
        let descriptor =
            existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
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

        collection.add(
            singleton::<dyn TestService, TestServiceImpl>()
                .from(|_| ServiceRef::new(TestServiceImpl::default())),
        );

        // act
        collection.try_add(
            singleton::<dyn TestService, TestServiceImpl>()
                .from(|_| ServiceRef::new(TestServiceImpl::default())),
        );

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
            singleton::<dyn OtherTestService, OtherTestServiceImpl>().from(|sp| {
                ServiceRef::new(OtherTestServiceImpl::new(
                    sp.get_required::<dyn TestService>(),
                ))
            }),
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
            transient::<dyn TestService, TestServiceImpl>()
                .from(|_| ServiceRef::new(TestServiceImpl::default())),
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
            transient::<dyn TestService, TestServiceImpl>()
                .from(|_| ServiceRef::new(TestServiceImpl::default())),
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
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            );

        // act
        collection.replace(
            singleton::<dyn TestService, TestServiceImpl>()
                .from(|_| ServiceRef::new(TestServiceImpl::default())),
        );

        // assert
        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn remove_all_should_remove_registered_services() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .add(
                singleton::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            );

        // act
        collection.remove_all::<dyn TestService>();

        // assert
        assert!(collection.is_empty());
    }

    #[test]
    fn try_replace_should_do_nothing_when_service_is_registered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection.add(
            singleton::<dyn TestService, TestServiceImpl>()
                .from(|_| ServiceRef::new(TestServiceImpl { value: 1 })),
        );

        // act
        collection.try_replace(
            singleton::<dyn TestService, TestServiceImpl>()
                .from(|_| ServiceRef::new(TestServiceImpl { value: 2 })),
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
        let descriptor =
            existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
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
                .add(singleton_as_self().from(|sp| {
                    ServiceRef::new(Droppable::new(sp.get_required::<Path>().to_path_buf()))
                }));
        }

        // assert
        let not_dropped = file.exists();
        remove_file(&file).ok();
        assert!(not_dropped);
    }
}
