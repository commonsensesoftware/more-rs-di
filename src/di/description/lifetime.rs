/// Represents the possible service lifetimes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServiceLifetime {
    /// Indicates that a single instance of the service will be created.
    Singleton,

    /// Indicates that a new instance of the service will be created for each scope.
    Scoped,

    /// Indicates that a new instance of the service will be created every time it is requested.
    Transient,
}
