#![doc = include_str!("README.md")]

#[cfg(not(feature = "async"))]
pub(crate) type Mut<T> = std::cell::RefCell<T>;

#[cfg(feature = "async")]
pub(crate) type Mut<T> = std::sync::RwLock<T>;

mod collection;
mod dependency;
mod descriptor;
mod keyed_ref;
mod provider;
mod r#type;
mod validation;

#[cfg(feature = "builder")]
mod builder;

#[cfg(feature = "builder")]
mod descriptor_builder;

#[cfg(feature = "inject")]
mod activator;

#[cfg(feature = "inject")]
mod inject;

#[cfg(feature = "inject")]
mod inject_builder;

#[cfg(feature = "lazy")]
mod lazy_init;

#[cfg(test)]
mod test;

pub use collection::*;
pub use dependency::*;
pub use descriptor::*;
pub use keyed_ref::*;
pub use provider::*;
pub use r#type::*;
pub use validation::*;

#[cfg(feature = "builder")]
pub use builder::*;

#[cfg(feature = "builder")]
pub use descriptor_builder::*;

#[cfg(feature = "inject")]
pub use activator::*;

#[cfg(feature = "inject")]
pub use inject::*;

#[cfg(feature = "inject")]
pub use inject_builder::*;

#[cfg(feature = "inject")]
pub use di_macros::{inject, injectable};

/// Contains support for lazy service resolution.
#[cfg(feature = "lazy")]
pub mod lazy {
    // use super::*;
    use super::lazy_init;
    // pub use lazy_init::{
    //     empty, empty_with_key, exactly_one, exactly_one_mut, exactly_one_with_key, missing, missing_with_key,
    //     zero_or_more, zero_or_more_with_key, zero_or_one, zero_or_one_with_key, Lazy,
    // };
    pub use lazy_init::*;
}
