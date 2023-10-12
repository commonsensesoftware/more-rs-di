use std::any::{type_name, Any};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

/// Represents a type.
#[derive(Clone, Debug, Eq)]
pub struct Type {
    id: u64,
    name: String,
    key: Option<String>,
}

impl Type {
    /// Initializes a new instance of a type.
    pub fn of<T: Any + ?Sized>() -> Self {
        Type::new(type_name::<T>().to_string(), None)
    }

    /// Initializes a new instance of a type based on another type as a key.
    pub fn keyed<TKey, TType: Any + ?Sized>() -> Self {
        Type::new(
            type_name::<TType>().to_string(),
            Some(type_name::<TKey>().to_string()),
        )
    }

    fn new(name: String, key: Option<String>) -> Self {
        let mut hasher = DefaultHasher::new();

        name.hash(&mut hasher);

        if let Some(ref val) = key {
            val.hash(&mut hasher);
        }

        Self {
            id: hasher.finish(),
            name,
            key,
        }
    }

    /// Gets the type identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the type name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Deconstructs the specified type into its name component
    /// and key component, if it is defined.
    pub fn deconstruct(t: &Type) -> (&str, Option<&str>) {
        (&t.name, t.key.as_deref())
    }
}

impl PartialEq<Type> for Type {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq<Type> for &Type {
    fn eq(&self, other: &Type) -> bool {
        self.id == other.id
    }
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Display for Type {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.name)
    }
}