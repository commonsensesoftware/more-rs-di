#![doc = include_str!("README.md")]

mod r#type;
mod collection;
mod dependency;
mod descriptor;
mod provider;
mod validation;

#[cfg(feature = "builder")]
mod builder;

#[cfg(feature = "inject")]
mod inject;

#[cfg(test)]
mod test;

pub use r#type::*;
pub use collection::*;
pub use dependency::*;
pub use descriptor::*;
pub use provider::*;
pub use validation::*;

#[cfg(feature = "builder")]
pub use builder::*;

#[cfg(feature = "inject")]
pub use inject::*;

#[cfg(feature = "inject")]
pub use di_macros::{inject, injectable};