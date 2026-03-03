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
