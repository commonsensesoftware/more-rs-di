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

/// Represents the metadata used to identify an injected function.
///
/// # Remarks
///
/// The default behavior looks for an associated function with the
/// name `new`. To change this behavior, decorate the function to
/// be used with `#[inject]`. This attribute may only be applied
/// to a single function.
#[cfg(feature = "inject")]
#[cfg_attr(docsrs, doc(cfg(feature = "inject")))]
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
#[cfg(feature = "inject")]
#[cfg_attr(docsrs, doc(cfg(feature = "inject")))]
pub use di_macros::injectable;

/// Contains support for lazy service resolution.
#[cfg(feature = "lazy")]
pub mod lazy {
    use super::lazy_init;

    #[cfg_attr(docsrs, doc(cfg(feature = "lazy")))]
    pub use lazy_init::*;
}
