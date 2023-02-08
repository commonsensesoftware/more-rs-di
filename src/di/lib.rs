#![doc = include_str!("README.md")]

mod collection;
mod dependency;
mod descriptor;
mod provider;
mod r#type;
mod validation;

#[cfg(feature = "builder")]
mod builder;

#[cfg(feature = "inject")]
mod inject;

#[cfg(feature = "lazy")]
mod lazy_init;

#[cfg(test)]
mod test;

pub use collection::*;
pub use dependency::*;
pub use descriptor::*;
pub use provider::*;
pub use r#type::*;
pub use validation::*;

#[cfg(feature = "builder")]
pub use builder::*;

#[cfg(feature = "inject")]
pub use inject::*;

#[cfg(feature = "inject")]
pub use di_macros::{inject, injectable};

/// Contains support for lazy service resolution.
#[cfg(feature = "lazy")]
pub mod lazy {
    use super::*;
    pub use lazy_init::{empty, exactly_one, missing, zero_or_more, zero_or_one, Lazy};
}
