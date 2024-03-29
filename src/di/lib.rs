#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Mut<T> is public primarily for code generation in the proc macro. it is
// generally uninteresting, but is required because, while we can detect a
// mutable service, we don't know which alias is behind the 'async' feature.
// the documentation will remain hidden to avoid confusion unless you really,
// really know and need to use it.

#[doc(hidden)]
#[cfg(not(feature = "async"))]
pub type Mut<T> = std::cell::RefCell<T>;

#[doc(hidden)]
#[cfg(feature = "async")]
pub type Mut<T> = std::sync::RwLock<T>;

mod collection;
mod dependency;
mod descriptor;
mod keyed;
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
pub use keyed::*;
pub use provider::*;
pub use r#type::*;
pub use validation::*;

#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub use builder::*;

#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub use descriptor_builder::*;

#[cfg(feature = "inject")]
#[cfg_attr(docsrs, doc(cfg(feature = "inject")))]
pub use activator::*;

#[cfg(feature = "inject")]
#[cfg_attr(docsrs, doc(cfg(feature = "inject")))]
pub use inject::*;

#[cfg(feature = "inject")]
#[cfg_attr(docsrs, doc(cfg(feature = "inject")))]
pub use inject_builder::*;

#[cfg(feature = "inject")]
#[cfg_attr(docsrs, doc(cfg(feature = "inject")))]
pub use di_macros::{inject, injectable};

/// Contains support for lazy service resolution.
#[cfg(feature = "lazy")]
pub mod lazy {
    use super::lazy_init;

    #[cfg_attr(docsrs, doc(cfg(feature = "lazy")))]
    pub use lazy_init::*;
}
