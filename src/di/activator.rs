use crate::{Mut, Ref, RefMut, ServiceFactory, ServiceProvider, Type};
use std::any::Any;

macro_rules! new {
    ($($traits:tt)+) => {
        /// Creates a new activator using the specified factory methods to instantiate the service.
        ///
        /// # Arguments
        ///
        /// * `factory` - The factory method used to create a service instance
        /// * `factory_mut` - The factory method used to create a mutable service instance
        pub fn new<TSvc: ?Sized + $($traits)+, TImpl>(
            factory: fn(&ServiceProvider) -> Ref<TSvc>,
            factory_mut: fn(&ServiceProvider) -> RefMut<TSvc>,
        ) -> Self {
            Self {
                service_type: Type::of::<TSvc>(),
                service_type_mut: Type::of::<Mut<TSvc>>(),
                implementation_type: Type::of::<TImpl>(),
                factory: Ref::new(move |sp| Ref::new(factory(sp))),
                factory_mut: Ref::new(move |sp| Ref::new(factory_mut(sp))),
                mutable: false,
            }
        }
    };
}

/// Represents an activator for a service instance.
pub struct Activator {
    service_type: Type,
    service_type_mut: Type,
    implementation_type: Type,
    factory: Ref<ServiceFactory>,
    factory_mut: Ref<ServiceFactory>,
    mutable: bool,
}

impl Activator {
    /// Gets the [service type](Type) associated with the service descriptor.
    pub fn service_type(&self) -> &Type {
        if self.mutable {
            &self.service_type_mut
        } else {
            &self.service_type
        }
    }

    /// Gets the [implementation type](Type) associated with the service descriptor.
    #[inline]
    pub fn implementation_type(&self) -> &Type {
        &self.implementation_type
    }

    /// Sets a value indicating whether the activated instance should be mutable.
    #[inline]
    pub fn as_mut(&mut self) {
        self.mutable = true;
    }

    /// Gets the [factory](ServiceFactory) method the activator represents.
    pub fn factory(&self) -> Ref<ServiceFactory> {
        if self.mutable {
            self.factory_mut.clone()
        } else {
            self.factory.clone()
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "async")] {
            new!(Any + Send + Sync);
        } else {
            new!(Any);
        }
    }
}
