use crate::{
    ServiceCardinality, ServiceCollection, ServiceDependency, ServiceDescriptor, ServiceLifetime,
    Type,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
struct ValidationResult {
    message: String,
}

impl ValidationResult {
    fn fail<T: AsRef<str>>(message: T) -> Self {
        Self {
            message: String::from(message.as_ref()),
        }
    }
}

/// Represents an validation error.
#[derive(Clone, Debug)]
pub struct ValidationError {
    message: String,
    results: Vec<ValidationResult>,
}

impl ValidationError {
    fn fail(results: Vec<ValidationResult>) -> Self {
        Self {
            message: if results.is_empty() {
                String::from("Validation failed.")
            } else if results.len() == 1 {
                results[0].message.clone()
            } else {
                String::from("One or more validation errors occurred.")
            },
            results,
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.message)?;

        if self.results.len() > 1 {
            for (i, result) in self.results.iter().enumerate() {
                write!(formatter, "\n  [{}] {}", i + 1, result.message)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        "validation error"
    }
}

trait ValidationRule<'a> {
    fn evaluate(&self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>);
}

struct MissingRequiredType<'a> {
    lookup: &'a HashMap<&'a Type, &'a ServiceDescriptor>,
}

impl<'a> MissingRequiredType<'a> {
    fn new(lookup: &'a HashMap<&'a Type, &'a ServiceDescriptor>) -> Self {
        Self { lookup }
    }
}

impl<'a> ValidationRule<'a> for MissingRequiredType<'a> {
    fn evaluate(&self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>) {
        for dependency in descriptor.dependencies() {
            if dependency.cardinality() == ServiceCardinality::ExactlyOne
                && !self.lookup.contains_key(dependency.injected_type())
            {
                results.push(ValidationResult::fail(format!(
                    "Service '{}' requires dependent service '{}', which has not be registered",
                    descriptor.implementation_type().name(),
                    dependency.injected_type().name()
                )));
            }
        }
    }
}

struct CircularDependency<'a> {
    lookup: &'a HashMap<&'a Type, &'a ServiceDescriptor>,
    queue: RefCell<Vec<&'a ServiceDependency>>,
}

impl<'a> CircularDependency<'a> {
    fn new(lookup: &'a HashMap<&'a Type, &'a ServiceDescriptor>) -> Self {
        Self {
            lookup,
            queue: RefCell::new(Vec::new()),
        }
    }

    fn check_dependency_graph(
        &self,
        root: &'a ServiceDescriptor,
        dependency: &'a ServiceDependency,
        results: &mut Vec<ValidationResult>,
    ) {
        let mut queue = self.queue.borrow_mut();

        queue.clear();
        queue.push(dependency);

        while let Some(current) = queue.pop() {
            if let Some(descriptor) = self.lookup.get(current.injected_type()) {
                if descriptor.service_type() != root.service_type() {
                    queue.extend(descriptor.dependencies());
                } else {
                    results.push(ValidationResult::fail(format!(
                        "A circular dependency was detected for service '{}' on service '{}'",
                        descriptor.service_type().name(),
                        root.implementation_type().name()
                    )));
                }
            }
        }
    }
}

impl<'a> ValidationRule<'a> for CircularDependency<'a> {
    fn evaluate(&self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>) {
        for dependency in descriptor.dependencies() {
            self.check_dependency_graph(descriptor, dependency, results);
        }
    }
}

struct SingletonDependsOnScoped<'a> {
    lookup: &'a HashMap<&'a Type, &'a ServiceDescriptor>,
    visited: RefCell<HashSet<&'a Type>>,
    queue: RefCell<Vec<&'a ServiceDescriptor>>,
}

impl<'a> SingletonDependsOnScoped<'a> {
    fn new(lookup: &'a HashMap<&'a Type, &'a ServiceDescriptor>) -> Self {
        Self {
            lookup,
            visited: RefCell::new(HashSet::new()),
            queue: RefCell::new(Vec::new()),
        }
    }
}

impl<'a> ValidationRule<'a> for SingletonDependsOnScoped<'a> {
    fn evaluate(&self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>) {
        if descriptor.lifetime() != ServiceLifetime::Singleton {
            return;
        }

        let mut level = "";
        let mut visited = self.visited.borrow_mut();
        let mut queue = self.queue.borrow_mut();

        visited.clear();
        queue.clear();
        queue.push(descriptor);

        while let Some(current) = queue.pop() {
            if !visited.insert(current.service_type()) {
                continue;
            }

            for dependency in current.dependencies() {
                if let Some(next) = self.lookup.get(dependency.injected_type()) {
                    queue.push(next);

                    if next.lifetime() == ServiceLifetime::Scoped {
                        results.push(ValidationResult::fail(format!(
                            "The service '{}' has a singleton lifetime, \
                             but its {}dependency '{}' has a scoped lifetime",
                            descriptor.implementation_type().name(),
                            level,
                            next.service_type().name()
                        )));
                    }
                }
            }

            level = "transitive ";
        }
    }
}

/// Validates the specified [service collection](struct.ServiceCollection.html).
///
/// # Arguments
///
/// * `services` - The [service collection](struct.ServiceCollection.html) to validate
pub fn validate(services: &ServiceCollection) -> Result<(), ValidationError> {
    let mut lookup = HashMap::with_capacity(services.len());

    for i in 0..services.len() {
        lookup.insert(services[i].service_type(), &services[i]);
    }

    let mut results = Vec::new();
    let missing_type = MissingRequiredType::new(&lookup);
    let circular_dep = CircularDependency::new(&lookup);
    let scoped_in_singleton = SingletonDependsOnScoped::new(&lookup);
    let rules: Vec<&dyn ValidationRule> = vec![&missing_type, &circular_dep, &scoped_in_singleton];

    for descriptor in services {
        for rule in &rules {
            rule.evaluate(descriptor, &mut results);
        }
    }

    if results.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::fail(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::*, *};

    #[test]
    fn validate_should_report_missing_required_type() {
        // arrange
        let mut services = ServiceCollection::new();

        services.add(
            singleton::<dyn OtherTestService, OtherTestServiceImpl>()
                .depends_on(exactly_one::<dyn TestService>())
                .from(|sp| {
                    ServiceRef::new(OtherTestServiceImpl::new(
                        sp.get_required::<dyn TestService>(),
                    ))
                }),
        );

        // act
        let result = validate(&services);

        // assert
        assert_eq!(
            &result.err().unwrap().to_string(),
            "Service 'di::test::OtherTestServiceImpl' requires dependent service \
             'dyn di::test::TestService', which has not be registered"
        );
    }

    #[test]
    fn validate_should_ignore_missing_optional_type() {
        // arrange
        let mut services = ServiceCollection::new();

        services.add(
            singleton::<dyn OtherTestService, TestOptionalDepImpl>()
                .depends_on(zero_or_one::<dyn TestService>())
                .from(|sp| ServiceRef::new(TestOptionalDepImpl::new(sp.get::<dyn TestService>()))),
        );

        // act
        let result = validate(&services);

        // assert
        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_report_circular_dependency() {
        // arrange
        let mut services = ServiceCollection::new();

        services.add(
            singleton::<dyn TestService, TestCircularDepImpl>()
                .depends_on(exactly_one::<dyn TestService>())
                .from(|sp| {
                    ServiceRef::new(TestCircularDepImpl::new(
                        sp.get_required::<dyn TestService>(),
                    ))
                }),
        );

        // act
        let result = validate(&services);

        // assert
        assert_eq!(
            &result.err().unwrap().to_string(),
            "A circular dependency was detected for service \
             'dyn di::test::TestService' on service 'di::test::TestCircularDepImpl'"
        );
    }

    #[test]
    fn validate_should_report_multiple_issues() {
        // arrange
        let mut services = ServiceCollection::new();

        services
            .add(
                singleton::<dyn TestService, TestAllKindOfProblems>()
                    .depends_on(exactly_one::<dyn OtherTestService>())
                    .depends_on(exactly_one::<dyn AnotherTestService>())
                    .from(|sp| {
                        ServiceRef::new(TestAllKindOfProblems::new(
                            sp.get_required::<dyn OtherTestService>(),
                            sp.get_required::<dyn AnotherTestService>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn OtherTestService, OtherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn TestService>())
                    .from(|sp| {
                        ServiceRef::new(OtherTestServiceImpl::new(
                            sp.get_required::<dyn TestService>(),
                        ))
                    }),
            );

        // act
        let result = validate(&services);

        // assert
        assert_eq!(
            &result.err().unwrap().to_string(),
            "One or more validation errors occurred.\n  \
              [1] Service 'di::test::TestAllKindOfProblems' requires dependent service 'dyn di::test::AnotherTestService', which has not be registered\n  \
              [2] A circular dependency was detected for service 'dyn di::test::TestService' on service 'di::test::TestAllKindOfProblems'\n  \
              [3] A circular dependency was detected for service 'dyn di::test::OtherTestService' on service 'di::test::OtherTestServiceImpl'");
    }

    #[test]
    fn validate_should_report_scoped_service_in_singleton() {
        // arrange
        let mut services = ServiceCollection::new();

        services
            .add(
                scoped::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .add(
                singleton::<dyn OtherTestService, OtherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn TestService>())
                    .from(|sp| {
                        ServiceRef::new(OtherTestServiceImpl::new(
                            sp.get_required::<dyn TestService>(),
                        ))
                    }),
            );

        // act
        let result = validate(&services);

        // assert
        assert_eq!(
            &result.err().unwrap().to_string(),
            "The service 'di::test::OtherTestServiceImpl' has a singleton lifetime, \
             but its dependency 'dyn di::test::TestService' has a scoped lifetime"
        );
    }

    #[test]
    fn validate_should_report_transitive_scoped_service_in_singleton() {
        // arrange
        let mut services = ServiceCollection::new();

        services
            .add(
                scoped::<dyn TestService, TestServiceImpl>()
                    .from(|_| ServiceRef::new(TestServiceImpl::default())),
            )
            .add(
                transient::<dyn OtherTestService, OtherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn TestService>())
                    .from(|sp| {
                        ServiceRef::new(OtherTestServiceImpl::new(
                            sp.get_required::<dyn TestService>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn AnotherTestService, AnotherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn OtherTestService>())
                    .from(|sp| {
                        ServiceRef::new(AnotherTestServiceImpl::new(
                            sp.get_required::<dyn OtherTestService>(),
                        ))
                    }),
            );

        // act
        let result = validate(&services);

        // assert
        assert_eq!(
            &result.err().unwrap().to_string(),
            "The service 'di::test::AnotherTestServiceImpl' has a singleton lifetime, \
             but its transitive dependency 'dyn di::test::TestService' has a scoped lifetime"
        );
    }

    #[test]
    fn validate_should_allow_unidirectional_dependency_multiple_times() {
        // arrange
        struct A;
        struct B;
        struct C;
        struct M;
        struct Z;

        let mut services = ServiceCollection::new();

        services
            .add(transient_as_self::<M>().from(|_| ServiceRef::new(M {})))
            .add(
                transient_as_self::<A>()
                    .depends_on(exactly_one::<B>())
                    .depends_on(exactly_one::<M>())
                    .from(|_| ServiceRef::new(A {})),
            )
            .add(
                transient_as_self::<B>()
                    .depends_on(exactly_one::<M>())
                    .from(|_| ServiceRef::new(B {})),
            )
            .add(
                transient_as_self::<C>()
                    .depends_on(exactly_one::<M>())
                    .from(|_| ServiceRef::new(C {})),
            )
            .add(
                transient_as_self::<Z>()
                    .depends_on(exactly_one::<A>())
                    .depends_on(exactly_one::<C>())
                    .depends_on(exactly_one::<M>())
                    .from(|_| ServiceRef::new(Z {})),
            );

        // act
        let result = validate(&services);

        // assert
        assert!(result.is_ok());
    }
}
