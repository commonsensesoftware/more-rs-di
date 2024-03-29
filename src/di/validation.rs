use crate::{
    ServiceCardinality, ServiceCollection, ServiceDependency, ServiceDescriptor, ServiceLifetime,
    Type,
};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

fn expand_type(t: &Type) -> String {
    let (name, key) = Type::deconstruct(t);

    match key {
        Some(val) => format!("'{}' with the key '{}'", name, val),
        _ => format!("'{}'", name),
    }
}

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
    fn evaluate(&mut self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>);
}

struct MissingRequiredType<'a> {
    lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>,
}

impl<'a> MissingRequiredType<'a> {
    fn new(lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>) -> Self {
        Self { lookup }
    }
}

impl<'a> ValidationRule<'a> for MissingRequiredType<'a> {
    fn evaluate(&mut self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>) {
        for dependency in descriptor.dependencies() {
            if dependency.cardinality() == ServiceCardinality::ExactlyOne
                && !self.lookup.contains_key(dependency.injected_type())
            {
                results.push(ValidationResult::fail(format!(
                    "Service '{}' requires dependent service {}, which has not be registered",
                    descriptor.implementation_type().name(),
                    expand_type(dependency.injected_type())
                )));
            }
        }
    }
}

struct CircularDependency<'a> {
    lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>,
    visited: HashSet<&'a Type>,
    queue: Vec<&'a ServiceDependency>,
}

impl<'a> CircularDependency<'a> {
    fn new(lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>) -> Self {
        Self {
            lookup,
            visited: HashSet::new(),
            queue: Vec::new(),
        }
    }

    fn check_dependency_graph(
        &mut self,
        root: &'a ServiceDescriptor,
        dependency: &'a ServiceDependency,
        results: &mut Vec<ValidationResult>,
    ) {
        self.queue.clear();
        self.queue.push(dependency);

        while let Some(current) = self.queue.pop() {
            if let Some(descriptors) = self.lookup.get(current.injected_type()) {
                for descriptor in descriptors {
                    if self.visited.insert(descriptor.service_type()) {
                        self.queue.extend(descriptor.dependencies());
                    }

                    if descriptor.service_type() == root.service_type() {
                        results.push(ValidationResult::fail(format!(
                            "A circular dependency was detected for service {} on service '{}'",
                            expand_type(descriptor.service_type()),
                            root.implementation_type().name()
                        )));
                    }
                }
            }
        }
    }
}

impl<'a> ValidationRule<'a> for CircularDependency<'a> {
    fn evaluate(&mut self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>) {
        for dependency in descriptor.dependencies() {
            self.visited.clear();
            self.visited.insert(descriptor.service_type());
            self.check_dependency_graph(descriptor, dependency, results);
        }
    }
}

struct SingletonDependsOnScoped<'a> {
    lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>,
    visited: HashSet<&'a Type>,
    queue: Vec<&'a ServiceDescriptor>,
}

impl<'a> SingletonDependsOnScoped<'a> {
    fn new(lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>) -> Self {
        Self {
            lookup,
            visited: HashSet::new(),
            queue: Vec::new(),
        }
    }
}

impl<'a> ValidationRule<'a> for SingletonDependsOnScoped<'a> {
    fn evaluate(&mut self, descriptor: &'a ServiceDescriptor, results: &mut Vec<ValidationResult>) {
        if descriptor.lifetime() != ServiceLifetime::Singleton {
            return;
        }

        let mut level = "";

        self.visited.clear();
        self.queue.clear();
        self.queue.push(descriptor);

        while let Some(current) = self.queue.pop() {
            if !self.visited.insert(current.service_type()) {
                continue;
            }

            for dependency in current.dependencies() {
                if let Some(descriptors) = self.lookup.get(dependency.injected_type()) {
                    for next in descriptors {
                        self.queue.push(next);

                        if next.lifetime() == ServiceLifetime::Scoped {
                            results.push(ValidationResult::fail(format!(
                                "The service {} has a singleton lifetime, \
                                 but its {}dependency '{}' has a scoped lifetime",
                                expand_type(descriptor.implementation_type()),
                                level,
                                next.service_type().name()
                            )));
                        }
                    }
                }
            }

            level = "transitive ";
        }
    }
}

/// Validates the specified [`ServiceCollection`](crate::ServiceCollection).
///
/// # Arguments
///
/// * `services` - The [`ServiceCollection`](crate::ServiceCollection) to validate
pub fn validate(services: &ServiceCollection) -> Result<(), ValidationError> {
    let mut lookup = HashMap::with_capacity(services.len());

    for item in services.iter() {
        let key = item.service_type();
        let descriptors = lookup.entry(key).or_insert_with(Vec::new);
        descriptors.push(item);
    }

    let mut results = Vec::new();
    let mut missing_type = MissingRequiredType::new(&lookup);
    let mut circular_dep = CircularDependency::new(&lookup);
    let mut scoped_in_singleton = SingletonDependsOnScoped::new(&lookup);
    let mut rules: Vec<&mut dyn ValidationRule> = vec![
        &mut missing_type,
        &mut circular_dep,
        &mut scoped_in_singleton,
    ];

    for descriptor in services {
        for rule in rules.iter_mut() {
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
                    Ref::new(OtherTestServiceImpl::new(
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
    fn validate_should_report_missing_required_keyed_type() {
        // arrange
        let mut services = ServiceCollection::new();

        services
            .add(
                singleton_as_self::<CatInTheHat>()
                    .depends_on(exactly_one_with_key::<key::Thing1, dyn Thing>())
                    .depends_on(zero_or_one_with_key::<key::Thing2, dyn Thing>())
                    .from(|sp| {
                        Ref::new(CatInTheHat::new(
                            sp.get_required_by_key::<key::Thing1, dyn Thing>(),
                            sp.get_by_key::<key::Thing2, dyn Thing>(),
                        ))
                    }),
            )
            .add(
                transient_with_key::<key::Thing2, dyn Thing, Thing2>()
                    .from(|_| Ref::new(Thing2::default())),
            );

        // act
        let result = validate(&services);

        // assert
        assert_eq!(
            &result.err().unwrap().to_string(),
            "Service 'di::test::CatInTheHat' requires dependent service \
             'dyn di::test::Thing' with the key 'di::test::key::Thing1', which has not be registered"
        );
    }

    #[test]
    fn validate_should_ignore_missing_optional_type() {
        // arrange
        let mut services = ServiceCollection::new();

        services.add(
            singleton::<dyn OtherTestService, TestOptionalDepImpl>()
                .depends_on(zero_or_one::<dyn TestService>())
                .from(|sp| Ref::new(TestOptionalDepImpl::new(sp.get::<dyn TestService>()))),
        );

        // act
        let result = validate(&services);

        // assert
        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_ignore_missing_optional_keyed_type() {
        // arrange
        let mut services = ServiceCollection::new();

        services
            .add(
                singleton_as_self::<CatInTheHat>()
                    .depends_on(exactly_one_with_key::<key::Thing1, dyn Thing>())
                    .depends_on(zero_or_one_with_key::<key::Thing2, dyn Thing>())
                    .from(|sp| {
                        Ref::new(CatInTheHat::new(
                            sp.get_required_by_key::<key::Thing1, dyn Thing>(),
                            sp.get_by_key::<key::Thing2, dyn Thing>(),
                        ))
                    }),
            )
            .add(
                transient_with_key::<key::Thing1, dyn Thing, Thing1>()
                    .from(|_| Ref::new(Thing1::default())),
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
                    Ref::new(TestCircularDepImpl::new(
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
                        Ref::new(TestAllKindOfProblems::new(
                            sp.get_required::<dyn OtherTestService>(),
                            sp.get_required::<dyn AnotherTestService>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn OtherTestService, OtherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn TestService>())
                    .from(|sp| {
                        Ref::new(OtherTestServiceImpl::new(
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
                    .from(|_| Ref::new(TestServiceImpl::default())),
            )
            .add(
                singleton::<dyn OtherTestService, OtherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn TestService>())
                    .from(|sp| {
                        Ref::new(OtherTestServiceImpl::new(
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
                    .from(|_| Ref::new(TestServiceImpl::default())),
            )
            .add(
                transient::<dyn OtherTestService, OtherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn TestService>())
                    .from(|sp| {
                        Ref::new(OtherTestServiceImpl::new(
                            sp.get_required::<dyn TestService>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn AnotherTestService, AnotherTestServiceImpl>()
                    .depends_on(exactly_one::<dyn OtherTestService>())
                    .from(|sp| {
                        Ref::new(AnotherTestServiceImpl::new(
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
    fn validate_should_not_report_circular_dependency_when_visited_multiple_times() {
        // arrange
        let mut services = ServiceCollection::new();

        services
            .add(
                singleton::<dyn ServiceM, ServiceMImpl>().from(|_sp| Ref::new(ServiceMImpl)),
            )
            .add(
                singleton::<dyn ServiceB, ServiceBImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .from(|sp| {
                        Ref::new(ServiceBImpl::new(sp.get_required::<dyn ServiceM>()))
                    }),
            )
            .add(
                singleton::<dyn ServiceC, ServiceCImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .from(|sp| {
                        Ref::new(ServiceCImpl::new(sp.get_required::<dyn ServiceM>()))
                    }),
            )
            .add(
                singleton::<dyn ServiceA, ServiceAImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceB>())
                    .from(|sp| {
                        Ref::new(ServiceAImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceB>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn ServiceY, ServiceYImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceC>())
                    .from(|sp| {
                        Ref::new(ServiceYImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceC>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn ServiceX, ServiceXImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceY>())
                    .from(|sp| {
                        Ref::new(ServiceXImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceY>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn ServiceZ, ServiceZImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceA>())
                    .depends_on(exactly_one::<dyn ServiceX>())
                    .from(|sp| {
                        Ref::new(ServiceZImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceA>(),
                            sp.get_required::<dyn ServiceX>(),
                        ))
                    }),
            );

        // act
        let result = validate(&services);

        // assert
        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_report_circular_dependency_in_complex_dependency_tree() {
        // arrange
        let mut services = ServiceCollection::new();

        services
            .add(
                singleton::<dyn ServiceM, ServiceMImpl>().from(|_sp| Ref::new(ServiceMImpl)),
            )
            .add(
                singleton::<dyn ServiceB, ServiceBImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .from(|sp| {
                        Ref::new(ServiceBImpl::new(sp.get_required::<dyn ServiceM>()))
                    }),
            )
            .add(
                singleton::<dyn ServiceC, ServiceCWithCircleRefToXImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceX>())
                    .from(|sp| {
                        Ref::new(ServiceCWithCircleRefToXImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceX>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn ServiceA, ServiceAImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceB>())
                    .from(|sp| {
                        Ref::new(ServiceAImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceB>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn ServiceY, ServiceYImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceC>())
                    .from(|sp| {
                        Ref::new(ServiceYImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceC>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn ServiceX, ServiceXImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceY>())
                    .from(|sp| {
                        Ref::new(ServiceXImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceY>(),
                        ))
                    }),
            )
            .add(
                singleton::<dyn ServiceZ, ServiceZImpl>()
                    .depends_on(exactly_one::<dyn ServiceM>())
                    .depends_on(exactly_one::<dyn ServiceA>())
                    .depends_on(exactly_one::<dyn ServiceX>())
                    .from(|sp| {
                        Ref::new(ServiceZImpl::new(
                            sp.get_required::<dyn ServiceM>(),
                            sp.get_required::<dyn ServiceA>(),
                            sp.get_required::<dyn ServiceX>(),
                        ))
                    }),
            );

        // act
        let result = validate(&services);

        // assert
        assert!(result.is_err());
    }
}
