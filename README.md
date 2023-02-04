# More Dependency Injection Crate

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/more-di.svg
[crates-url]: https://crates.io/crates/more-di
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/commonsensesoftware/more-rs-di/blob/main/LICENSE

This library contains all of the fundamental abstractions for dependency injection (DI).
A `trait` or `struct` can be used as the injected type.

## Features

This crate provides the following features:

- _Default_ - Provides the abstractions for dependency injection, plus the **builder** and **inject** features
- **builder** - Provides utility functions for configuring service descriptors
- **async** - Provides features for using dependencies in an asynchronous context
- **inject** - Provides constructor injection

## Service Lifetimes

A service can have the following lifetimes:

- **Transient** - a new instance is created every time it is requested
- **Singleton** - a single, new instance is created the first time it is requested
- **Scoped** - a new instance is created once per `ServiceProvider` the first time it is requested

## Examples

Consider the following traits and structures:

```rust
use di::ServiceRef;

trait Foo {
    fn speak(&self) -> String;
}

trait Bar {
    fn speak(&self) -> String;
}

#[derive(Default)]
struct FooImpl { }

impl Foo for FooImpl {
    fn speak(&self) -> String {
        String::from("foo")
    }
}

struct BarImpl {
    foo: ServiceRef<dyn Foo>
}

impl BarImpl {
    fn new(foo: ServiceRef<dyn Foo>) -> Self {
        Self { foo }
    }
}

impl Bar for BarImpl {
    fn speak(&self) -> String {
        let mut text = self.foo.speak();
        text.push_str(" bar");
        text
    }
}
```

### Service Registration and Resolution

```rust
fn main() {
    let mut services = ServiceCollection::new();

    services.add(
        singleton::<dyn Foo, FooImpl>()
        .from(|_| Rc::new(FooImpl::default())));
    services.add(
        transient::<dyn Bar, BarImpl>()
        .from(|sp| Rc::new(BarImpl::new(sp.get_required::<dyn Foo>()))));

    let provider = services.build_provider().unwrap();
    let bar = provider.get_required::<dyn Bar>();
    let text = bar.speak();

    assert_eq!(text, "foo bar")
}
```

_Figure 1: Basic usage_

>Note: `singleton` and `transient` are utility functions provided by the **builder** feature.

In the preceding example, `ServiceCollection::add` is used to add `ServiceDescriptor` instances.
The framework also provides `ServiceCollection::try_add`, which only registers the service if
there isn't already an implementation registered.

In the following example, the call to `try_add` has no effect because the service has already
been registered:

```rust
let mut services = ServiceCollection::new();

services.add(transient::<dyn Foo, Foo2>().from(|_| Rc::new(Foo2::default())));
services.try_add(transient::<dyn Foo, FooImpl>().from(|_| Rc::new(FooImpl::default())));
```

### Scope Scenarios

There scenarios where a service needs to be _scoped_; for example, for the lifetime of a
HTTP request. A service definitely shouldn't live for the life of the application (e.g. _singleton_),
but it also shouldn't be created each time it's requested within the request (e.g. _transient_).
A _scoped_ service lives for the lifetime of the container it was created from.

```rust
let provider = ServiceCollection::new()
    .add(
        scoped::<dyn Foo, FooImpl>()
        .from(|_| Rc::new(FooImpl::default())))
    .add(
        transient::<dyn Bar, BarImpl>()
        .from(|sp| Rc::new(BarImpl::new(sp.get_required::<dyn Foo>()))))
    .build_provider()
    .unwrap();

{
    // create a scope where Bar is shared
    let scope = provider.create_scope();
    let bar1 = provider.get_required::<dyn Bar>();
    let bar2 = provider.get_required::<dyn Bar>();
    
    assert!(Rc::ptr_eq(&bar1, &bar2));
}

{
    // create a new scope where Bar is shared and different from before
    let scope = provider.create_scope();
    let bar1 = provider.get_required::<dyn Bar>();
    let bar2 = provider.get_required::<dyn Bar>();
    
    assert!(Rc::ptr_eq(&bar1, &bar2));
}
```

_Figure 2: Using scoped services_

### Validation

The consumers of a `ServiceProvider` expect that is correctly configured and ready for use. There are edge cases,
however, which could lead to runtime failures.

- A required, dependent service that has not be registered
- A circular dependency, which will trigger a stack overflow

Intrinsic validation has been added to ensure this cannot happen. The `build_provider()` method will return
`Result<ServiceProvider, ValidationError>`, which will either contain a valid `ServiceProvider` or a
`ValidationError` that will detail all of the errors. From that point forward, the `ServiceProvider` will be
considered semantically correct and safe to use. The same validation process can also be invoked imperatively
on-demand by using the `di::validate` method.

The `ServiceDescriptorBuilder` cannot automatically determine the dependencies your service may require. This
means that validation is an explicit, opt-in capability. If you do not configure any dependencies for a
`ServiceDescriptor`, then no validation will occur.

```rust
fn main() {
    let mut services = ServiceCollection::new();

    services.add(
        singleton::<dyn Foo, FooImpl>()
        .from(|_| Rc::new(FooImpl::default())));
    services.add(
        transient::<dyn Bar, BarImpl>()
        .depends_on(ServiceDependency::new(Type::of::<dyn Foo>(), ServiceMultiplicity::ExactlyOne))
        .from(|sp| Rc::new(BarImpl::new(sp.get_required::<dyn Foo>()))));

    match services.build_provider() {
        Ok(provider) => {
            let bar = provider.get_required::<dyn Bar>();
            assert_eq!(&bar.speak(), "foo bar");
        },
        Err(error) => {
            println!("The service configuration is invalid.\n{}", &error.to_string());
        }
    }
}
```
_Figure 3: Validating service configuration_

### Inject Feature

The `Injectable` trait can be implemented so that structures can be injected as a
single, supported trait or as themselves.

```rust
use di::*;
use std::rc::Rc;

impl Injectable for FooImpl {
    fn inject(lifetime: ServiceLifetime) -> ServiceDescriptor {
        ServiceDescriptorBuilder::<dyn Foo, Self>::new(lifetime, Type::of::<Self>())
            .from(|_| Rc::new(FooImpl::default()))
    }
}

impl Injectable for BarImpl {
    fn inject(lifetime: ServiceLifetime) -> ServiceDescriptor {
        ServiceDescriptorBuilder::<dyn Bar, Self>::new(lifetime, Type::of::<Self>())
            .from(|sp| Rc::new(BarImpl::new(sp.get_required::<dyn Foo>())))
    }
}
```

_Figure 4: Implementing `Injectable`_

While implementing `Injectable` _might_ be necessary or desired in a handful of scenarios, it is mostly tedious ceremony.
If the injection call site were known, then it would be possible to provide the implementation automatically. This is exactly
what the `#[injectable]` attribute provides.

Instead of implementing `Injectable` by hand, the implementation simply applies a decorator:

```rust
use di::injectable;
use std::rc::Rc;

#[injectable(Bar)]
impl BarImpl {
    fn new(foo: Rc<dyn Foo>) -> Self {
        Self { foo: foo }
    }
}
```

_Figure 5: Automatically implementing `Injectable`_

#### Injection Rules

Notice that the attribute is decorated on the `impl` of the struct as opposed to a trait implementation. This is because this is
the location where the associated function that will be used to construct the struct is expected to be found. This allows the
attribute to inspect the injection call site to build the proper implementation. The attribute contains the trait to be satisfied.
If this process where reversed, it would require a _lookahead_ or _lookbehind_ to search for the implementation.

By default, the attribute will search for an associated function named `new`. The function does not need to be `pub`. This is
a simple convention that works for most cases; however, if you want to use a different name, the intended function must be
decorated with the `#[inject]` attribute. This attribute simply indicates which function to use. If `new` and a decorated function
are defined, the decorated function will take precedence. If multiple functions have `#[inject]` applied, an error will occur.

Call site arguments must conform to the return values from:

- `ServiceProvider` - return the provider itself as a dependency
- `ServiceProvider.get` - return an optional dependency
- `ServiceProvider.get_required`- return a required dependency (or panic)
- `ServiceProvider.get_all` - return all dependencies of a known type, which could be zero

This means that the only allowed arguments are:

- `ServiceRef<T>` 
- `Option<ServiceRef<T>>`
- `Vec<ServiceRef<T>>`
- `ServiceProvider`

`ServiceRef<T>` is a provided type alias for `Rc<T>` by default, but becomes `Arc<T>` when the **async** feature is enabled. `Rc<T>` and `Arc<T>` are also allowed anywhere `ServiceRef<T>` is allowed. For every injected type `T`, the appropriate `ServiceDependency` configuration is also added so that injected types can be validated.

The following is an advanced example with all of these concepts applied:

```rust
trait Logger {
    fn log(&self, message: &str);
}

trait Translator {
    fn translate(&self, text: &str, lang: &str) -> String;
}

#[injectable(Bar)]
impl BarImpl {
    #[inject]
    fn create(
        foo: ServiceRef<dyn Foo>,
        translator: Option<ServiceRef<dyn Translator>>,
        loggers: Vec<ServiceRef<dyn Logger>>) -> Self {
        Self {
            foo: foo,
            translator,
            loggers: loggers,
        }
    }
}
```

_Figure 6: Advanced `Injectable` configuration_

Which will expand to:

```rust
impl Injectable for BarImpl {
    fn inject(lifetime: ServiceLifetime) -> ServiceDescriptor {
        ServiceDescriptorBuilder::<dyn Bar, Self>::new(lifetime, Type::of::<Self>())
            .depends_on(ServiceDependency::new(Type::of::<dyn Foo>(), ServiceMultiplicity::ExactlyOne))
            .depends_on(ServiceDependency::new(Type::of::<dyn Translator>(), ServiceMultiplicity::ZeroOrOne))
            .depends_on(ServiceDependency::new(Type::of::<dyn Logger>(), ServiceMultiplicity::ZeroOrMore))
            .from(|sp| Rc::new(
                BarImpl::create(
                    sp.get_required::<dyn Foo>(),
                    sp.get::<dyn Translator>(),
                    sp.get_all::<dyn Logger>().collect())))
    }
}
```

_Figure 7: Advanced `Injectable` implementation_

#### Simplified Registration

Blanket implementations are provided for:

- `Injectable.singleton`
- `Injectable.scoped`
- `Injectable.transient`

This simplifies registration to:

```rust
fn main() {
    let provider = ServiceCollection::new()
        .add(FooImpl::singleton())
        .add(BarImpl::transient())
        .build_provider()
        .unwrap();

    let bar = provider.get_required::<dyn Bar>();
    let text = bar.speak();

    assert_eq!(text, "foo bar")
}
```
_Figure 8: **inject** feature usage_

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/more-rs-di/blob/main/LICENSE