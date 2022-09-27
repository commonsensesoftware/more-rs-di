#![doc = include_str!("README.md")]

mod r#type;
mod collection;
mod descriptor;
mod provider;

#[cfg(feature = "builder")]
mod builder;

#[cfg(feature = "inject")]
mod inject;

#[cfg(test)]
mod test;

pub use r#type::*;
pub use collection::*;
pub use descriptor::*;
pub use provider::*;

#[cfg(feature = "builder")]
pub use builder::*;

#[cfg(feature = "inject")]
pub use inject::*;

#[cfg(feature = "inject")]
pub use di_macros::{inject, injectable};