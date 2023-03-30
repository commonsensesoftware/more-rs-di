use daggy::{Dag, NodeIndex};

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
    graph: RefCell<Dag<&'a Type, ()>>,
    node_lookup: RefCell<HashMap<&'a Type, NodeIndex>>,
}

impl<'a> CircularDependency<'a> {
    fn new(lookup: &'a HashMap<&'a Type, &'a ServiceDescriptor>) -> Self {
        Self {
            lookup,
            graph: RefCell::new(Dag::<&Type, ()>::new()),
            node_lookup: RefCell::new(HashMap::new()),
        }
    }

    fn check_dependency_graph(
        &self,
        root: &'a ServiceDescriptor,
        dependency: &'a ServiceDependency,
        results: &mut Vec<ValidationResult>,
    ) {
        self.clean_graph();
        let root_node = self.get_service_descriptor_node_index(root.service_type());
        let dependency_node = self.get_service_descriptor_node_index(dependency.injected_type());

        let add_edge_result = self.add_edge(root_node, dependency_node);
        match add_edge_result {
            Ok(_) => {
                self.add_nodes(dependency, results);
            }
            Err(_) => results.push(ValidationResult::fail(format!(
                "A circular dependency was detected for service '{}' on service '{}'",
                dependency.injected_type().name(),
                root.implementation_type().name(),
            ))),
        }

        self.add_nodes(dependency, results);

        results.dedup_by(|a, b| a.message.eq_ignore_ascii_case(&b.message));
    }

    fn clean_graph(&self) {
        let mut graph_nodes = self.node_lookup.borrow_mut();
        let mut graph = self.graph.borrow_mut();
        graph_nodes.clear();
        graph.clear();
    }

    fn add_nodes(&self, current: &'a ServiceDependency, results: &mut Vec<ValidationResult>) {
        if let Some(descriptor) = self.lookup.get(current.injected_type()) {
            let root_node = self.get_service_descriptor_node_index(descriptor.service_type());

            for dependency in descriptor.dependencies() {
                let dependency_node =
                    self.get_service_descriptor_node_index(dependency.injected_type());
                let add_edge_result = self.add_edge(root_node, dependency_node);
                match add_edge_result {
                    Ok(_) => {
                        self.add_nodes(dependency, results);
                    }
                    Err(_) => results.push(ValidationResult::fail(format!(
                        "A circular dependency was detected for service '{}' on service '{}'",
                        dependency.injected_type().name(),
                        descriptor.implementation_type().name(),
                    ))),
                }
            }
        }
    }

    fn add_edge(
        &self,
        root_node: NodeIndex,
        dependency_node: NodeIndex,
    ) -> Result<daggy::EdgeIndex, daggy::WouldCycle<()>> {
        let mut graph = self.graph.borrow_mut();
        graph.add_edge(root_node, dependency_node, ())
    }

    fn get_service_descriptor_node_index(&self, descriptor_type: &'a Type) -> NodeIndex {
        let mut graph_nodes = self.node_lookup.borrow_mut();
        let mut graph = self.graph.borrow_mut();
        if let Some(index) = graph_nodes.get(descriptor_type) {
            *index
        } else {
            let index = graph.add_node(descriptor_type);
            graph_nodes.insert(descriptor_type, index);
            index
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
            "A circular dependency was detected for service 'dyn di::test::TestService' on service 'di::test::TestCircularDepImpl'"
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
              [2] A circular dependency was detected for service 'dyn di::test::TestService' on service 'di::test::OtherTestServiceImpl'\n  \
              [3] A circular dependency was detected for service 'dyn di::test::OtherTestService' on service 'di::test::TestAllKindOfProblems'");
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

    #[test]
    fn dependency_visited_multiple_times_should_not_be_circular() {
        let mut services = ServiceCollection::new();

        services.add(
            singleton::<dyn ServiceM, ServiceMImpl>().from(|_sp| ServiceRef::new(ServiceMImpl)),
        );
        services.add(
            singleton::<dyn ServiceB, ServiceBImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .from(|sp| ServiceRef::new(ServiceBImpl::new(sp.get_required::<dyn ServiceM>()))),
        );
        services.add(
            singleton::<dyn ServiceC, ServiceCImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .from(|sp| ServiceRef::new(ServiceCImpl::new(sp.get_required::<dyn ServiceM>()))),
        );
        services.add(
            singleton::<dyn ServiceA, ServiceAImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceB>())
                .from(|sp| {
                    ServiceRef::new(ServiceAImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceB>(),
                    ))
                }),
        );
        services.add(
            singleton::<dyn ServiceY, ServiceYImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceC>())
                .from(|sp| {
                    ServiceRef::new(ServiceYImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceC>(),
                    ))
                }),
        );
        services.add(
            singleton::<dyn ServiceX, ServiceXImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceY>())
                .from(|sp| {
                    ServiceRef::new(ServiceXImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceY>(),
                    ))
                }),
        );
        services.add(
            singleton::<dyn ServiceZ, ServiceZImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceA>())
                .depends_on(exactly_one::<dyn ServiceX>())
                .from(|sp| {
                    ServiceRef::new(ServiceZImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceA>(),
                        sp.get_required::<dyn ServiceX>(),
                    ))
                }),
        );

        // act
        let result = validate(&services);
        assert_eq!(result.is_err(), false) //I will add the proper assert here;
    }

    #[test]
    fn validate_should_report_circular_dependency_on_complex_dependency_tree() {
        let mut services = ServiceCollection::new();

        services.add(
            singleton::<dyn ServiceM, ServiceMImpl>().from(|_sp| ServiceRef::new(ServiceMImpl)),
        );
        services.add(
            singleton::<dyn ServiceB, ServiceBImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .from(|sp| ServiceRef::new(ServiceBImpl::new(sp.get_required::<dyn ServiceM>()))),
        );
        services.add(
            singleton::<dyn ServiceC, ServiceCWithCircleRefToXImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceX>())
                .from(|sp| {
                    ServiceRef::new(ServiceCWithCircleRefToXImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceX>(),
                    ))
                }),
        );
        services.add(
            singleton::<dyn ServiceA, ServiceAImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceB>())
                .from(|sp| {
                    ServiceRef::new(ServiceAImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceB>(),
                    ))
                }),
        );
        services.add(
            singleton::<dyn ServiceY, ServiceYImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceC>())
                .from(|sp| {
                    ServiceRef::new(ServiceYImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceC>(),
                    ))
                }),
        );
        services.add(
            singleton::<dyn ServiceX, ServiceXImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceY>())
                .from(|sp| {
                    ServiceRef::new(ServiceXImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceY>(),
                    ))
                }),
        );
        services.add(
            singleton::<dyn ServiceZ, ServiceZImpl>()
                .depends_on(exactly_one::<dyn ServiceM>())
                .depends_on(exactly_one::<dyn ServiceA>())
                .depends_on(exactly_one::<dyn ServiceX>())
                .from(|sp| {
                    ServiceRef::new(ServiceZImpl::new(
                        sp.get_required::<dyn ServiceM>(),
                        sp.get_required::<dyn ServiceA>(),
                        sp.get_required::<dyn ServiceX>(),
                    ))
                }),
        );

        // act
        let result = validate(&services);
        assert_eq!(result.is_err(), true) // I will add the proper assert here;
    }
}
