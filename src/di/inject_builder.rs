use crate::{Activator, ServiceDependency, Type, ServiceLifetime, ServiceDescriptor};
use spin::Once;

/// Represents the builder for an injected type.
pub struct InjectBuilder {
    activator: Activator,
    lifetime: ServiceLifetime,
    key_type: Option<Type>,
    dependencies: Vec<ServiceDependency>,
}

impl InjectBuilder {
    /// Initializes a new builder.
    ///
    /// # Arguments
    ///
    /// * `activator` - The [activator](crate::Activator) used to activate the service
    /// * `lifetime` - The [lifetime](crate::ServiceLifetime) of the service
    pub fn new(activator: Activator, lifetime: ServiceLifetime) -> Self {
        Self {
            activator,
            lifetime,
            key_type: None,
            dependencies: Vec::default(),
        }
    }

    /// Defines a dependency used by the service.
    ///
    /// # Arguments
    ///
    /// * `dependency` - The [dependency](crate::ServiceDependency) associated with the services
    pub fn depends_on(mut self, dependency: ServiceDependency) -> Self {
        if !self.dependencies.contains(&dependency) {
            self.dependencies.push(dependency);
        }
        self
    }

    /// Applies a key to the injected service.
    pub fn with_key<TKey>(mut self) -> Self {
        self.key_type = Some(Type::of::<TKey>());
        self
    }

    /// Indicates the injected service is mutable.
    pub fn as_mut(mut self) -> Self {
        self.activator.as_mut();
        self
    }

    /// Builds and returns a new [`ServiceDescriptor`](crate::ServiceDescriptor).
    pub fn build(mut self) -> ServiceDescriptor {
        ServiceDescriptor::new(
            self.lifetime,
            if let Some(key) = self.key_type {
                self.activator.service_type().with_key(&key)
            } else {
                self.activator.service_type().clone()
            },
            self.activator.implementation_type().clone(),
            if self.dependencies.is_empty() {
                Vec::with_capacity(0)
            } else {
                self.dependencies.shrink_to_fit();
                self.dependencies
            },
            Once::new(),
            self.activator.factory(),
        )
    }
}

impl From<InjectBuilder> for ServiceDescriptor {
    fn from(value: InjectBuilder) -> Self {
        value.build()
    }
}
