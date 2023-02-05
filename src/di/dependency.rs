use crate::Type;

/// Represents the possible cardinalities of a service dependency.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServiceCardinality {
    /// Indicates a cardinality of zero or one (0:1).
    ZeroOrOne,

    /// Indicates a cardinality of exactly one (1:1).
    ExactlyOne,

    /// Indicates a cardinality of zero or more (0:*).
    ZeroOrMore,
}

/// Represents a service dependency.
#[derive(Clone, Debug)]
pub struct ServiceDependency {
    injected_type: Type,
    cardinality: ServiceCardinality,
}

impl ServiceDependency {
    /// Initializes a new service dependency.
    /// 
    /// # Arguments
    /// 
    /// * `injected_type` - the [injected type](struct.Type.html) of the service dependency
    /// * `cardinality` - the [cardinality](enum.ServiceCardinality.html) of the service dependency
    pub fn new(injected_type: Type, cardinality: ServiceCardinality) -> Self {
        Self {
            injected_type,
            cardinality,
        }
    }

    /// Gets the [injected type](struct.Type.html) associated with the service dependency.
    pub fn injected_type(&self) -> &Type {
        &self.injected_type
    }

    /// Gets the [cardinality](enum.ServiceCardinality.html) associated with the service dependency.
    pub fn cardinality(&self) -> ServiceCardinality {
        self.cardinality
    }
}