#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use cfg_if::cfg_if;

// Mut<T> is public primarily for code generation in the proc macro. it is
// generally uninteresting, but is required because, while we can detect a
// mutable service, we don't know which alias is behind the 'async' feature.
// the documentation will remain hidden to avoid confusion unless you really,
// really know and need to use it.

cfg_if! {
    if #[cfg(feature = "async")] {
        /// Represents the type alias for a mutable service reference.
        #[doc(hidden)]
        pub type Mut<T> = std::sync::RwLock<T>;
    } else {
        /// Represents the type alias for a mutable service reference.
        #[doc(hidden)]
        pub type Mut<T> = std::cell::RefCell<T>;
    }
}

mod collection;
mod dependency;
mod descriptor;
mod keyed;
mod provider;
mod r#type;
mod validation;

pub use collection::ServiceCollection;
pub use dependency::{ServiceCardinality, ServiceDependency};
pub use descriptor::{Ref, RefMut, ServiceDescriptor, ServiceFactory, ServiceLifetime};
pub use keyed::{KeyedRef, KeyedRefMut};
pub use provider::{ScopedServiceProvider, ServiceProvider};
pub use r#type::Type;
pub use validation::{validate, ValidationError};

cfg_if! {
    if #[cfg(feature = "builder")] {
        mod builder;
        mod descriptor_builder;

        pub use builder::{
            exactly_one, exactly_one_with_key, existing, existing_as_self, existing_with_key, existing_with_key_as_self,
            scoped, scoped_factory, scoped_with_key, scoped_with_key_factory, singleton, singleton_as_self,
            singleton_factory, singleton_with_key, singleton_with_key_factory, transient, transient_as_self,
            transient_factory, transient_with_key, transient_with_key_as_self, transient_with_key_factory, zero_or_more,
            zero_or_more_with_key, zero_or_one, zero_or_one_with_key,
        };
        pub use descriptor_builder::ServiceDescriptorBuilder;
    }
}

cfg_if! {
    if #[cfg(feature = "inject")] {
        mod activator;
        mod inject;
        mod inject_builder;

        pub use activator::Activator;
        pub use inject::Injectable;
        pub use inject_builder::InjectBuilder;

        /// Represents the metadata used to identify an injected function.
        ///
        /// # Remarks
        ///
        /// The default behavior looks for an associated function with the
        /// name `new`. To change this behavior, decorate the function to
        /// be used with `#[inject]`. This attribute may only be applied
        /// to a single function.
        pub use di_macros::inject;

        /// Represents the metadata used to implement the `Injectable` trait.
        ///
        /// # Arguments
        ///
        /// * `trait` - the optional name of the trait the implementation satisfies.
        ///
        /// # Remarks
        ///
        /// This attribute may be applied a struct definition or a struct `impl`
        /// block. The defining struct implementation block must either have an
        /// associated function named `new` or decorate the injected function with
        /// `#[inject]`. The injected function does not have to be public.
        ///
        /// If `trait` is not specified, then the implementation will
        /// injectable as the defining struct itself.
        ///
        /// The injected call site arguments are restricted to the same return
        /// values supported by `ServiceProvider`, which can only be:
        ///
        /// * `Ref<T>`
        /// * `RefMut<T>`
        /// * `Option<Ref<T>>`
        /// * `Option<RefMut<T>>`
        /// * `Vec<Ref<T>>`
        /// * `Vec<RefMut<T>>`
        /// * `impl Iterator<Item = Ref<T>>`
        /// * `impl Iterator<Item = RefMut<T>>`
        /// * `Lazy<Ref<T>>`
        /// * `Lazy<RefMut<T>>`
        /// * `Lazy<Option<Ref<T>>>`
        /// * `Lazy<Option<RefMut<T>>>`
        /// * `Lazy<Vec<Ref<T>>>`
        /// * `Lazy<Vec<RefMut<T>>>`
        /// * `KeyedRef<TKey, TSvc>`
        /// * `KeyedRefMut<TKey, TSvc>`
        /// * `ServiceProvider`
        /// * `ScopedServiceProvider`
        ///
        /// `Ref<T>` is a type alias for `Rc<T>` or `Arc<T>` and `RefMut<T>` is a
        /// type alias for `Rc<RefCell<T>>` or `Arc<RwLock<T>>` depending on whether
        /// the **async** feature is activated; therefore, `Rc<T>` and `Arc<T>`
        /// are allowed any place `Ref<T>` is allowed and `Rc<RefCell<T>>`
        /// and `Arc<RwLock<T>>` are allowed any place `RefMut<T>` is allowed.
        ///
        /// # Examples
        ///
        /// Injecting a struct as a trait.
        ///
        /// ```
        /// use di::*;
        ///
        /// pub trait Foo {
        ///    fn do_work(&self);
        /// }
        ///
        /// pub struct FooImpl;
        ///
        /// impl Foo for FooImpl {
        ///     fn do_work(&self) {
        ///         println!("Did something!");
        ///     }
        /// }
        ///
        /// #[injectable(Foo)]
        /// impl FooImpl {
        ///     pub fn new() -> Self {
        ///         Self {}
        ///     }
        /// }
        /// ```
        ///
        /// Injecting a struct as itself.
        ///
        /// ```
        /// use di::*;
        ///
        /// #[injectable]
        /// pub struct Foo;
        ///
        /// impl Foo {
        ///     fn do_work(&self) {
        ///         println!("Did something!");
        ///     }
        /// }
        /// ```
        ///
        /// Define a custom injection function.
        ///
        /// ```
        /// use di::*;
        ///
        /// pub struct Bar;
        /// pub struct Foo {
        ///     bar: di::Ref<Bar>
        /// };
        ///
        /// #[injectable]
        /// impl Foo {
        ///     #[inject]
        ///     pub fn create(bar: di::Ref<Bar>) -> Self {
        ///         Self { bar }
        ///     }
        /// }
        pub use di_macros::injectable;
    }
}

cfg_if! {
    if #[cfg(feature = "lazy")] {
        mod lazy_init;

        /// Contains support for lazy service resolution.
        pub mod lazy {
            use super::lazy_init;

            pub use lazy_init::{
                Lazy, empty, empty_with_key, exactly_one, exactly_one_mut, exactly_one_with_key,
                exactly_one_with_key_mut, init, init_mut, init_with_key, init_with_key_mut, missing, missing_with_key,
                zero_or_more, zero_or_more_mut, zero_or_more_with_key, zero_or_more_with_key_mut, zero_or_one,
                zero_or_one_mut, zero_or_one_with_key, zero_or_one_with_key_mut,
            };
        }
    }
}

#[cfg(test)]
mod test;
