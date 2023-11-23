use crate::{
    ServiceDependency, ServiceDescriptor, ServiceLifetime, ServiceProvider, Ref, Type,
};
use spin::Once;
use std::any::Any;
use std::marker::PhantomData;

/// Represents a [`ServiceDescriptor`](crate::ServiceDescriptor) builder.
pub struct ServiceDescriptorBuilder<TSvc: Any + ?Sized, TImpl> {
    lifetime: ServiceLifetime,
    service_type: Type,
    implementation_type: Type,
    dependencies: Vec<ServiceDependency>,
    _marker_svc: PhantomData<TSvc>,
    _marker_impl: PhantomData<TImpl>,
}

impl<TSvc: Any + ?Sized, TImpl> ServiceDescriptorBuilder<TSvc, TImpl> {
    /// Defines the factory method used to activate the service and returns the [`ServiceDescriptor`](crate::ServiceDescriptor).
    ///
    /// # Arguments
    ///
    /// * `factory` - The factory method used to create the service
    pub fn from(mut self, factory: impl Fn(&ServiceProvider) -> Ref<TSvc> + 'static) -> ServiceDescriptor {
        ServiceDescriptor::new(
            self.lifetime,
            self.service_type,
            self.implementation_type,
            if self.dependencies.is_empty() {
                Vec::with_capacity(0)
            } else {
                self.dependencies.shrink_to_fit();
                self.dependencies
            },
            Once::new(),
            Ref::new(move |sp| Ref::new(factory(sp))),
        )
    }

    /// Defines a dependency used by the service.
    ///
    /// # Arguments
    ///
    /// * `dependency` - The [dependency](crate::ServiceDependency) associated with the service
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
    /// * `lifetime` - The [lifetime](crate::ServiceLifetime) of the service
    /// * `implementation_type` - The service implementation [type](crate::Type)
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
    /// * `lifetime` - The [lifetime](crate::ServiceLifetime) of the service
    /// * `implementation_type` - The service implementation [type](crate::Type)
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
