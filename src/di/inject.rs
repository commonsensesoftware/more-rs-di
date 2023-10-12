use crate::{ServiceDescriptor, ServiceLifetime};

/// Defines the behavior of an injectable type.
pub trait Injectable: Sized {
    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for an injectable type.
    ///
    /// # Arguments
    ///
    /// * `lifetime` - The [lifetime](enum.ServiceLifetime.html) of the injected type.
    fn inject(lifetime: ServiceLifetime) -> ServiceDescriptor;

    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for a singleton injected type.
    fn singleton() -> ServiceDescriptor {
        Self::inject(ServiceLifetime::Singleton)
    }

    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for a scoped injected type.
    fn scoped() -> ServiceDescriptor {
        Self::inject(ServiceLifetime::Scoped)
    }

    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for a transient injected injected.
    fn transient() -> ServiceDescriptor {
        Self::inject(ServiceLifetime::Transient)
    }
}

/// Defines the behavior of an injectable type with a key.
pub trait KeyedInjectable: Sized {
    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for an injectable type with a key.
    ///
    /// # Arguments
    ///
    /// * `lifetime` - The [lifetime](enum.ServiceLifetime.html) of the injected type.
    fn inject_with_key<TKey>(lifetime: ServiceLifetime) -> ServiceDescriptor;

    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for a singleton injected type with a key.
    fn keyed_singleton<TKey>() -> ServiceDescriptor {
        Self::inject_with_key::<TKey>(ServiceLifetime::Singleton)
    }

    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for a scoped injected type with a key.
    fn keyed_scoped<TKey>() -> ServiceDescriptor {
        Self::inject_with_key::<TKey>(ServiceLifetime::Scoped)
    }

    /// Creates and returns a [service descriptor](struct.ServiceDescriptor.html) for a transient injected injected with a key.
    fn keyed_transient<TKey>() -> ServiceDescriptor {
        Self::inject_with_key::<TKey>(ServiceLifetime::Transient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    trait TestService {}
    trait OtherTestService {}

    #[derive(Default)]
    struct TestServiceImpl {}
    struct OtherTestServiceImpl {
        _service: ServiceRef<dyn TestService>,
    }

    impl TestService for TestServiceImpl {}

    impl Injectable for TestServiceImpl {
        fn inject(lifetime: ServiceLifetime) -> ServiceDescriptor {
            ServiceDescriptorBuilder::<dyn TestService, Self>::new(lifetime, Type::of::<Self>())
                .from(|_| ServiceRef::new(Self::default()))
        }
    }

    impl OtherTestServiceImpl {
        fn new(service: ServiceRef<dyn TestService>) -> Self {
            Self { _service: service }
        }
    }

    impl Injectable for OtherTestServiceImpl {
        fn inject(lifetime: ServiceLifetime) -> ServiceDescriptor {
            ServiceDescriptorBuilder::<dyn OtherTestService, Self>::new(
                lifetime,
                Type::of::<Self>(),
            )
            .from(|sp| ServiceRef::new(Self::new(sp.get_required::<dyn TestService>())))
        }
    }

    impl OtherTestService for OtherTestServiceImpl {}

    #[test]
    fn inject_should_invoke_constructor_injection() {
        // arrange
        let services = ServiceCollection::new()
            .add(TestServiceImpl::singleton())
            .add(OtherTestServiceImpl::transient())
            .build_provider()
            .unwrap();

        // act
        let service = services.get::<dyn OtherTestService>();

        // assert
        assert!(service.is_some());
    }
}
