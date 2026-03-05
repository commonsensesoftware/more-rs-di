use super::ServiceLifetime::{self, *};
use crate::{Ref, ServiceDependency, ServiceFactory, ServiceProvider, Type};
use std::any::Any;
use std::sync::OnceLock;

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        macro_rules! service {
            () => { dyn Any + Send + Sync };
        }
    } else {
        macro_rules! service {
            () => { dyn Any };
        }
    }
}

/// Represents the description of a service with its service type, implementation, and lifetime.
pub struct ServiceDescriptor {
    lifetime: ServiceLifetime,
    service_type: Type,
    implementation_type: Type,
    dependencies: Vec<ServiceDependency>,
    factory: Ref<ServiceFactory>,
    instance: Ref<OnceLock<Ref<service!()>>>,
}

impl ServiceDescriptor {
    #[cfg(any(feature = "builder", feature = "inject"))]
    pub(crate) fn new(
        lifetime: ServiceLifetime,
        service_type: Type,
        implementation_type: Type,
        dependencies: Vec<ServiceDependency>,
        instance: OnceLock<Ref<service!()>>,
        factory: Ref<ServiceFactory>,
    ) -> Self {
        Self {
            lifetime,
            service_type,
            implementation_type,
            dependencies,
            instance: Ref::new(instance),
            factory,
        }
    }

    /// Gets the [lifetime](ServiceLifetime) associated with the service descriptor.
    #[inline]
    pub fn lifetime(&self) -> ServiceLifetime {
        self.lifetime
    }

    /// Gets the service [type](Type) associated with the service descriptor.
    #[inline]
    pub fn service_type(&self) -> &Type {
        &self.service_type
    }

    /// Gets the implementation [type](Type) associated with the service descriptor.
    #[inline]
    pub fn implementation_type(&self) -> &Type {
        &self.implementation_type
    }

    /// Gets the associated [service dependencies](ServiceDependency), if any.
    #[inline]
    pub fn dependencies(&self) -> &[ServiceDependency] {
        &self.dependencies
    }

    /// Gets or creates the service defined by the service descriptor.
    ///
    /// # Arguments
    ///
    /// * `services` - The current [service provider](ServiceProvider)
    pub fn get(&self, services: &ServiceProvider) -> Ref<service!()> {
        if self.lifetime == Transient {
            return (self.factory)(services);
        }

        self.instance.get_or_init(|| (self.factory)(services)).clone()
    }

    pub(crate) fn clone_with(&self, dependencies: bool) -> Self {
        Self {
            lifetime: self.lifetime,
            service_type: self.service_type.clone(),
            implementation_type: self.implementation_type.clone(),
            dependencies: if dependencies {
                self.dependencies.clone()
            } else {
                Vec::new()
            },
            instance: if self.lifetime == Singleton {
                self.instance.clone()
            } else {
                Ref::new(OnceLock::new())
            },
            factory: self.factory.clone(),
        }
    }
}

impl Clone for ServiceDescriptor {
    #[inline]
    fn clone(&self) -> Self {
        // this might not be 'safe' so always copy dependencies
        self.clone_with(true)
    }
}
