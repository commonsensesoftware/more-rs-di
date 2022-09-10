use crate::{ServiceProvider, Type};
use spin::Once;
use std::any::Any;
use std::marker::PhantomData;

/// Represents the possible service lifetimes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServiceLifetime {
    /// Indicates that a single instance of the service will be created.
    Singleton,

    /// Indicates that a new instance of the service will be created for each scope.
    Scoped,

    /// Indicates that a new instance of the service will be created every time it is requested.
    Transient,
}

/// Represents the type alias for a service reference.
#[cfg(not(feature = "async"))]
pub type ServiceRef<T> = std::rc::Rc<T>;

/// Represents the type alias for a service reference.
#[cfg(feature = "async")]
pub type ServiceRef<T> = std::sync::Arc<T>;

/// Represents the callback function used to create a service.
pub type ServiceFactory = dyn Fn(&ServiceProvider) -> ServiceRef<dyn Any>;

/// Represents the description of a service with its service type, implementation, and lifetime.
pub struct ServiceDescriptor {
    lifetime: ServiceLifetime,
    service_type: Type,
    implementation_type: Type,
    instance: ServiceRef<Once<ServiceRef<dyn Any>>>,
    factory: ServiceRef<ServiceFactory>,
}

impl ServiceDescriptor {
    #[cfg(feature = "builder")]
    pub(crate) fn new(
        lifetime: ServiceLifetime,
        service_type: Type,
        implementation_type: Type,
        instance: Once<ServiceRef<dyn Any>>,
        factory: ServiceRef<ServiceFactory>,
    ) -> Self {
        Self {
            lifetime,
            service_type,
            implementation_type,
            instance: ServiceRef::new(instance),
            factory,
        }
    }

    /// Gets the [lifetime](enum.ServiceLifetime.html) associated with the service descriptor.
    pub fn lifetime(&self) -> ServiceLifetime {
        self.lifetime
    }

    /// Gets the [service type](struct.Type.html) associated with the service descriptor.
    pub fn service_type(&self) -> &Type {
        &self.service_type
    }

    /// Gets the [implementation type](struct.Type.html) associated with the service descriptor.
    pub fn implementation_type(&self) -> &Type {
        &self.implementation_type
    }

    /// Gets or creates the service defined by the service descriptor.
    ///
    /// # Arguments
    ///
    /// * `services` - The current [service provider](struct.ServiceProvider.html).
    pub fn get(&self, services: &ServiceProvider) -> ServiceRef<dyn Any> {
        if self.lifetime == ServiceLifetime::Transient {
            return (self.factory)(services);
        }

        return self.instance.call_once(|| (self.factory)(services)).clone();
    }
}

impl Clone for ServiceDescriptor {
    fn clone(&self) -> Self {
        Self {
            lifetime: self.lifetime,
            service_type: self.service_type.clone(),
            implementation_type: self.implementation_type.clone(),
            instance: if self.lifetime == ServiceLifetime::Singleton {
                self.instance.clone()
            } else {
                ServiceRef::new(Once::new())
            },
            factory: self.factory.clone(),
        }
    }
}

/// Represents a builder for [service descriptors](struct.ServiceDescriptor.html).
pub struct ServiceDescriptorBuilder<TSvc: Any + ?Sized, TImpl> {
    lifetime: ServiceLifetime,
    implementation_type: Type,
    _marker_svc: PhantomData<TSvc>,
    _marker_impl: PhantomData<TImpl>,
}

impl<TSvc: Any + ?Sized, TImpl> ServiceDescriptorBuilder<TSvc, TImpl> {
    /// Defines the factory method used to activate the service and returns the service descriptor.
    ///
    /// # Arguments
    ///
    /// * `factory` - The factory method used to create the service
    pub fn from<F>(self, factory: F) -> ServiceDescriptor
    where
        F: Fn(&ServiceProvider) -> ServiceRef<TSvc> + 'static,
    {
        ServiceDescriptor {
            lifetime: self.lifetime,
            service_type: Type::of::<TSvc>(),
            implementation_type: self.implementation_type,
            instance: ServiceRef::new(Once::new()),
            factory: ServiceRef::new(move |sp| ServiceRef::new(factory(sp))),
        }
    }

    /// Initializes a new service descriptor builder.
    ///
    /// # Arguments
    ///
    /// * `lifetime` - The [lifetime](enum.ServiceLifetime.html) of the service.
    pub fn new(lifetime: ServiceLifetime, implementation_type: Type) -> Self {
        Self {
            lifetime,
            implementation_type,
            _marker_svc: PhantomData,
            _marker_impl: PhantomData,
        }
    }
}