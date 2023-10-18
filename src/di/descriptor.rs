use crate::{ServiceDependency, ServiceProvider, Type};
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

/// Represents the type alias for a mutable service reference.
pub type ServiceRefMut<T> = ServiceRef<std::sync::Mutex<T>>;

/// Represents the callback function used to create a service.
pub type ServiceFactory = dyn Fn(&ServiceProvider) -> ServiceRef<dyn Any>;

/// Represents the description of a service with its service type, implementation, and lifetime.
pub struct ServiceDescriptor {
    lifetime: ServiceLifetime,
    service_type: Type,
    implementation_type: Type,
    dependencies: Vec<ServiceDependency>,
    instance: ServiceRef<Once<ServiceRef<dyn Any>>>,
    factory: ServiceRef<ServiceFactory>,
}

impl ServiceDescriptor {
    #[cfg(any(feature = "builder", feature = "inject"))]
    pub(crate) fn new(
        lifetime: ServiceLifetime,
        service_type: Type,
        implementation_type: Type,
        dependencies: Vec<ServiceDependency>,
        instance: Once<ServiceRef<dyn Any>>,
        factory: ServiceRef<ServiceFactory>,
    ) -> Self {
        Self {
            lifetime,
            service_type,
            implementation_type,
            dependencies,
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

    /// Gets the associated [service dependencies](struct.ServiceDependency.html), if any.
    pub fn dependencies(&self) -> &[ServiceDependency] {
        &self.dependencies
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

    pub(crate) fn clone_with(&self, dependencies: bool) -> Self {
        Self {
            lifetime: self.lifetime,
            service_type: self.service_type.clone(),
            implementation_type: self.implementation_type.clone(),
            dependencies: if dependencies {
                self.dependencies.clone()
            } else {
                Vec::with_capacity(0)
            },
            instance: if self.lifetime == ServiceLifetime::Singleton {
                self.instance.clone()
            } else {
                ServiceRef::new(Once::new())
            },
            factory: self.factory.clone(),
        }
    }
}

impl Clone for ServiceDescriptor {
    fn clone(&self) -> Self {
        // without context, we don't know if this is 'safe';
        // always copy dependencies here
        self.clone_with(true)
    }
}

/// Represents a builder for [service descriptors](struct.ServiceDescriptor.html).
pub struct ServiceDescriptorBuilder<TSvc: Any + ?Sized, TImpl> {
    lifetime: ServiceLifetime,
    service_type: Type,
    implementation_type: Type,
    dependencies: Vec<ServiceDependency>,
    _marker_svc: PhantomData<TSvc>,
    _marker_impl: PhantomData<TImpl>,
}

impl<TSvc: Any + ?Sized, TImpl> ServiceDescriptorBuilder<TSvc, TImpl> {
    /// Defines the factory method used to activate the service and returns the service descriptor.
    ///
    /// # Arguments
    ///
    /// * `factory` - The factory method used to create the service
    pub fn from<F>(mut self, factory: F) -> ServiceDescriptor
    where
        F: Fn(&ServiceProvider) -> ServiceRef<TSvc> + 'static,
    {
        ServiceDescriptor {
            lifetime: self.lifetime,
            service_type: self.service_type,
            implementation_type: self.implementation_type,
            dependencies: if self.dependencies.is_empty() {
                Vec::with_capacity(0)
            } else {
                self.dependencies.shrink_to_fit();
                self.dependencies
            },
            instance: ServiceRef::new(Once::new()),
            factory: ServiceRef::new(move |sp| ServiceRef::new(factory(sp))),
        }
    }

    /// Defines a dependency used by the service.
    ///
    /// # Arguments
    ///
    /// * `dependency` - The [dependency](struct.ServiceDependency.html) associated with the service
    pub fn depends_on(mut self, dependency: ServiceDependency) -> Self {
        if !self.dependencies.contains(&dependency) {
            self.dependencies.push(dependency);
        }
        self
    }

    /// Initializes a new service descriptor builder.
    ///
    /// # Arguments
    ///
    /// * `lifetime` - The [lifetime](enum.ServiceLifetime.html) of the service
    /// * `implementation_type` - The service implementation [type](struct.Type.html)
    pub fn new(lifetime: ServiceLifetime, implementation_type: Type) -> Self {
        Self {
            lifetime,
            service_type: Type::of::<TSvc>(),
            implementation_type,
            dependencies: Vec::new(),
            _marker_svc: PhantomData,
            _marker_impl: PhantomData,
        }
    }

    /// Initializes a new service descriptor builder.
    ///
    /// # Arguments
    ///
    /// * `lifetime` - The [lifetime](enum.ServiceLifetime.html) of the service
    /// * `implementation_type` - The service implementation [type](struct.Type.html)
    pub fn keyed<TKey>(lifetime: ServiceLifetime, implementation_type: Type) -> Self {
        Self {
            lifetime,
            service_type: Type::keyed::<TKey, TSvc>(),
            implementation_type,
            dependencies: Vec::new(),
            _marker_svc: PhantomData,
            _marker_impl: PhantomData,
        }
    }
}
