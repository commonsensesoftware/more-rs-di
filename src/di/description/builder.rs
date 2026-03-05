use super::{ServiceDescriptor, ServiceLifetime};
use crate::{Ref, ServiceDependency, ServiceProvider, Type};
use std::any::Any;
use std::marker::PhantomData;
use std::sync::OnceLock;

/// Represents a [ServiceDescriptor] builder.
pub struct ServiceDescriptorBuilder<TSvc: ?Sized, TImpl> {
    lifetime: ServiceLifetime,
    service_type: Type,
    implementation_type: Type,
    dependencies: Vec<ServiceDependency>,
    _marker_svc: PhantomData<TSvc>,
    _marker_impl: PhantomData<TImpl>,
}

impl<TSvc: ?Sized, TImpl> ServiceDescriptorBuilder<TSvc, TImpl> {
    /// Defines a dependency used by the service.
    ///
    /// # Arguments
    ///
    /// * `dependency` - The [dependency](ServiceDependency) associated with the service
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
    /// * `lifetime` - The [lifetime](ServiceLifetime) of the service
    /// * `implementation_type` - The service implementation [type](Type)
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
    /// * `lifetime` - The [lifetime](ServiceLifetime) of the service
    /// * `implementation_type` - The service implementation [type](Type)
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

macro_rules! from {
    (($($traits:tt)+), ($($bounds:tt)+)) => {
        impl<TSvc: ?Sized + $($traits)+, TImpl> ServiceDescriptorBuilder<TSvc, TImpl> {
            /// Defines the factory function used to activate the service and returns the corresponding [ServiceDescriptor].
            ///
            /// # Arguments
            ///
            /// * `factory` - The factory function used to activate the service
            pub fn from(mut self, factory: impl (Fn(&ServiceProvider) -> Ref<TSvc>) + $($bounds)+) -> ServiceDescriptor {
                ServiceDescriptor::new(
                    self.lifetime,
                    self.service_type,
                    self.implementation_type,
                    if self.dependencies.is_empty() {
                        Vec::new()
                    } else {
                        self.dependencies.shrink_to_fit();
                        self.dependencies
                    },
                    OnceLock::new(),
                    Ref::new(move |sp| Ref::new(factory(sp))),
                )
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        from!((Any + Send + Sync), (Send + Sync + 'static));
    } else {
        from!((Any), ('static));
    }
}
