#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use cfg_if::cfg_if;
use std::any::Any;

// Mut<T> is public primarily for code generation in the proc macro. it is
// generally uninteresting, but is required because, while we can detect a
// mutable service, we don't know which alias is behind the 'async' feature.
// the documentation will remain hidden to avoid confusion unless you really,
// really know and need to use it.

cfg_if! {
    if #[cfg(feature = "async")] {
        /// Represents the type alias for a service reference.
        pub type Ref<T> = std::sync::Arc<T>;

        /// Represents the type alias for a mutable service reference.
        #[doc(hidden)]
        pub type Mut<T> = std::sync::RwLock<T>;
    } else {
        /// Represents the type alias for a service reference.
        pub type Ref<T> = std::rc::Rc<T>;

        /// Represents the type alias for a mutable service reference.
        #[doc(hidden)]
        pub type Mut<T> = std::cell::RefCell<T>;
    }
}

/// Represents the type alias for a mutable service reference.
pub type RefMut<T> = Ref<Mut<T>>;

/// Represents the callback function used to create a service.
pub type ServiceFactory = dyn Fn(&ServiceProvider) -> Ref<dyn Any>;

mod collection;
mod dependency;
mod description;
pub(crate) mod fmt;
mod keyed;
mod provider;
mod r#type;
mod validation;

pub use collection::ServiceCollection;
pub use dependency::{ServiceCardinality, ServiceDependency};
pub use description::{ServiceDescriptor, ServiceLifetime};
pub use keyed::{KeyedRef, KeyedRefMut};
pub use provider::{ScopedServiceProvider, ServiceProvider};
pub use r#type::Type;
pub use validation::{validate, ValidationError};

cfg_if! {
    if #[cfg(feature = "builder")] {
        mod builder;

        pub use builder::{
            exactly_one, exactly_one_with_key, existing, existing_as_self, existing_with_key, existing_with_key_as_self,
            scoped, scoped_factory, scoped_with_key, scoped_with_key_factory, singleton, singleton_as_self,
            singleton_factory, singleton_with_key, singleton_with_key_factory, transient, transient_as_self,
            transient_factory, transient_with_key, transient_with_key_as_self, transient_with_key_factory, zero_or_more,
            zero_or_more_with_key, zero_or_one, zero_or_one_with_key,
        };
        pub use description::ServiceDescriptorBuilder;
    }
}

cfg_if! {
    if #[cfg(feature = "inject")] {
        mod activator;
        mod inject;

        pub use activator::Activator;
        pub use inject::{InjectBuilder, Injectable, inject, injectable};
    }
}

cfg_if! {
    if #[cfg(feature = "lazy")] {
        /// Contains support for lazy service resolution.
        pub mod lazy;
    }
}

#[cfg(test)]
mod test;
