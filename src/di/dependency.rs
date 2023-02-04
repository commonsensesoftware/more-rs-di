use crate::Type;

/// Represents the possible multiplicities of a service dependency.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServiceMultiplicity {
    /// Indicates a multiplicity of zero or one (0:1).
    ZeroOrOne,

    /// Indicates a multiplicity of exactly one (1:1).
    ExactlyOne,

    /// Indicates a multiplicity of zero or more (0:*).
    ZeroOrMore,
}

/// Represents a service dependency.
#[derive(Clone)]
pub struct ServiceDependency {
    injected_type: Type,
    multiplicity: ServiceMultiplicity,
}

impl ServiceDependency {
    /// Initializes a new service dependency.
    /// 
    /// # Arguments
    /// 
    /// * `injected_type` - the [injected type](struct.Type.html) of the service dependency
    /// * `multiplicity` - the [multiplicity](enum.ServiceMultiplicity.html) of the service dependency
    pub fn new(injected_type: Type, multiplicity: ServiceMultiplicity) -> Self {
        Self {
            injected_type,
            multiplicity,
        }
    }

    /// Gets the [injected type](struct.Type.html) associated with the service dependency.
    pub fn injected_type(&self) -> &Type {
        &self.injected_type
    }

    /// Gets the [multiplicity](enum.ServiceMultiplicity.html) associated with the service dependency.
    pub fn multiplicity(&self) -> ServiceMultiplicity {
        self.multiplicity
    }
}