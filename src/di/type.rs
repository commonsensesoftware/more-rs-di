use std::any::{type_name, Any};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::hash::{Hash, Hasher};

/// Represents a type.
#[derive(Clone, Debug, Eq)]
pub struct Type {
	id: u64,
	name: String,
}

impl Type {
	/// Initializes a new instance of a type.
	pub fn of<T: Any + ?Sized>() -> Self {
		let name = type_name::<T>().to_string();
		let mut hasher = DefaultHasher::new();

		name.hash(&mut hasher);

		Self {
			id: hasher.finish(),
			name,
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
