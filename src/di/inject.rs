use crate::{InjectBuilder, ServiceLifetime};

/// Defines the behavior of an injectable type.
pub trait Injectable: Sized {
    /// Creates and returns a [builder](crate::InjectBuilder) for an injected type.
    ///
    /// # Arguments
    ///
    /// * `lifetime` - The [lifetime](crate::ServiceLifetime) of the injected type
    fn inject(lifetime: ServiceLifetime) -> InjectBuilder;

    /// Creates and returns a [builder](crate::InjectBuilder) for a singleton injected type.
    fn singleton() -> InjectBuilder {
        Self::inject(ServiceLifetime::Singleton)
    }

    /// Creates and returns a [builder](crate::InjectBuilder) for a scoped injected type.
    fn scoped() -> InjectBuilder {
        Self::inject(ServiceLifetime::Scoped)
    }

    /// Creates and returns a [builder](crate::InjectBuilder) for a transient injected type.
    fn transient() -> InjectBuilder {
        Self::inject(ServiceLifetime::Transient)
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
        _service: Ref<dyn TestService>,
    }

    impl TestService for TestServiceImpl {}

    impl Injectable for TestServiceImpl {
        fn inject(lifetime: ServiceLifetime) -> InjectBuilder {
            InjectBuilder::new(
                Activator::new::<dyn TestService, Self>(
                    |_| Ref::new(Self::default()),
                    |_| Ref::new(Mut::new(Self::default())),
                ),
                lifetime,
            )
        }
    }

    impl OtherTestServiceImpl {
        fn new(service: Ref<dyn TestService>) -> Self {
            Self { _service: service }
        }
    }

    impl Injectable for OtherTestServiceImpl {
        fn inject(lifetime: ServiceLifetime) -> InjectBuilder {
            InjectBuilder::new(
                Activator::new::<dyn OtherTestService, Self>(
                    |sp| Ref::new(Self::new(sp.get_required::<dyn TestService>())),
                    |sp| {
                        Ref::new(Mut::new(Self::new(sp.get_required::<dyn TestService>())))
                    },
                ),
                lifetime,
            )
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
