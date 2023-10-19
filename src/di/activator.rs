use crate::{ServiceFactory, ServiceProvider, ServiceRef, ServiceRefMut, Type};
use std::{any::Any, sync::Mutex};

/// Represents an activator for a service instance.
pub struct Activator {
    service_type: Type,
    service_type_mut: Type,
    implementation_type: Type,
    factory: ServiceRef<ServiceFactory>,
    factory_mut: ServiceRef<ServiceFactory>,
    mutable: bool,
}

impl Activator {
    /// Gets the [service type](struct.Type.html) associated with the service descriptor.
    pub fn service_type(&self) -> &Type {
        if self.mutable {
            &self.service_type_mut
        } else {
            &self.service_type
        }
    }

    /// Gets the [implementation type](struct.Type.html) associated with the service descriptor.
    pub fn implementation_type(&self) -> &Type {
        &self.implementation_type
    }

    /// Sets a value indicating whether the activated instance should be mutable.
    pub fn as_mut(&mut self) {
        self.mutable = true;
    }

    /// Gets the factory method the activator represents.
    pub fn factory(&self) -> ServiceRef<ServiceFactory> {
        if self.mutable {
            self.factory_mut.clone()
        } else {
            self.factory.clone()
        }
    }

    /// Creates a new activator using the specified factory methods to instantiate the service.
    ///
    /// # Arguments
    ///
    /// * `factory` - The factory method used to create a service instance
    /// * `factory_mut` - The factory method used to create a mutable service instance
    pub fn new<TSvc: Any + ?Sized, TImpl>(
        factory: fn(&ServiceProvider) -> ServiceRef<TSvc>,
        factory_mut: fn(&ServiceProvider) -> ServiceRefMut<TSvc>) -> Self
    {
        Self {
            service_type: Type::of::<TSvc>(),
            service_type_mut: Type::of::<Mutex<TSvc>>(),
            implementation_type: Type::of::<TImpl>(),
            factory: ServiceRef::new(move |sp| ServiceRef::new(factory(sp))),
            factory_mut: ServiceRef::new(move |sp| ServiceRef::new(factory_mut(sp))),
            mutable: false,
        }
    }
}
