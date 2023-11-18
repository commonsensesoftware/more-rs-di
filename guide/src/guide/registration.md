# Service Registration

## Service Descriptors

The foundation of the entire crate revolves around a `ServiceDescriptor`. A descriptor describes the following about a service:

- The service type
- The implementation type
- Its [lifetime](lifetimes.md)
- Its [dependencies](validation.md#service-dependency), if any
- The factory function used to instantiate the service

Rust does not have a Reflection API so the `di::Type` struct is used to represent a pseudo-type. A `ServiceDescriptor` also enables a collection of services to be explored, validated, and/or modified.

To ensure that a `ServiceDescriptor` is properly constructed, you can only create an instance through one of the provided factories:

- `ServiceDescriptorBuilder<TSvc,TImpl>`
- `InjectBuilder`

## Service Collection

A `ServiceCollection` is a mutable container of `ServiceDescriptor` instances that you can modify before creating an immutable `ServiceProvider`. The `ServiceCollection` allows you to register or modify services in any order. When you're ready to create a `ServiceProvider`, the `ServiceCollection` will validate all service dependencies before constructing the instance. The `ServiceCollection` cannot guarantee you won't ask for a service that doesn't exist, but it can guarantee any service it knows about can be correctly resolved. The `ServiceCollection` is ultimately a factory and can create multiple, independent `ServiceProvider` instances if you want.

For binary applications, most users will only add descriptors to the `ServiceCollection`. The `ServiceCollection` becomes much more useful in library crates and test applications. Here is a summary of the most useful functions:

| Function         | Description                                                    |
| ---------------- | -------------------------------------------------------------- |
| `add`            | Adds a new item                                                |
| `try_add`        | Attempts to add a new item if the same service is unregistered |
| `try_add_to_all` | Attempts to add a new item to a set if it's unregistered       |
| `try_add_all`    | Adds a sequence of new items                                   |
| `replace`        | Adds a new item or replaces an existing registration           |
| `try_replace`    | Equivalent to `try_add`                                        |

### Best Practice

To make it easy to test a binary application, it is recommended that you expose a public function that configures the default set of services. This will make it simple to use the same default configuration as the application and replace only the parts that are necessary for testing.

```rust
use di::*;

#[injectable]
pub struct Runner;

impl Runner {
    pub fn run(&self) {
        // TODO: implementation
    }
}
    
pub fn config_default_services(services: &mut ServiceCollection) {
    services.add(Runner::singleton());
    // TODO: register other services
    // replaceable services should use try_add so that any
    // existing registration, say from a test, is not overridden
}

fn main() {
    let mut services = ServiceCollection::new();

    config_default_services(&mut services);

    let provider = services.build_provider().unwrap();
    let runner = provider.get_required::<Runner>();

    runner.run();
}
```

You can now create a test replicating the same setup as the release application, but only changing the parts you need to for testing.

```rust
use crate::*;
use di::*;

#[test]
fn runner_should_do_expected_work() {
    // arrange
    let mut services = ServiceCollection::new();

    // TODO: add test replacements with: services.add(?);

    config_default_services(&mut services);

    // TODO: optionally, override defaults with: services.replace(?);

    let provider = services.build_provider().unwrap();
    let runner = provider.get_required::<Runner>();

    // act
    runner.run();

    // assert
    // TODO: assertions
}
```

## Mutable Services

The borrowing rules imposed by Rust places limitations on creating mutable services. The service lifetimes supported by dependency injection make using the `mut` keyword in an idiomatic way impossible. There are, at least, three possible alternate solutions:

1. Use _Interior Mutability_ within your service implementation
2. Design your service as a factory which is shared within DI, but can create instances owned outside the factory that are idiomatically mutable
3. Decorate your service with `RefCell<T>` or, if the **async** feature is activated, `Mutex<T>`

**Option 3** is the only method provided out-of-the-box as the other options are subjective design choices within the scope of your application. One of the consequences of this approach is that the types `RefCell<T>` and `Mutex<T>` themselves become part of the service registration; `Ref<T>` and `Ref<RefCell<T>>` (or `RefMut<T>` for short) are considered different services. In most use cases, this is not a problem. Your service is either entirely read-only or it is read-write. If you need both and two different service instances will not work for you or you want finer-grained control over synchronization, you should consider _Interior Mutability_ instead.

## Builder

>These features are only available if the **builder** feature is activated

The `ServiceDescriptorBuilder<TSvc,TImpl>` is the long-form approach used to create `ServiceDescriptor` instances. It is most useful when you need to create `ServiceDescriptor` instances and you don't want to use the provided macros. You might also need this capability for a scenario not supported by the macros or because you need to inject types defined in an external crate that do not provide extensibility points from the `more-di` crate.

The `ServieDescriptorBuilder<TSvc,TImpl>` is accompanied by numerous shorthand functions to simplify registration:

| Function                     | Starts Building                                          |
| ---------------------------- | -------------------------------------------------------- |
| `singleton`                  | A singleton service                                      |
| `singleton_as_self`          | A singleton service for a struct                         |
| `singleton_factory`          | A singleton service from a factory function              |
| `singleton_with_key`         | A singleton service with a key                           |
| `singleton_with_key_factory` | A singleton service using a key and factory function     |
| `scoped`                     | A scoped service                                         |
| `scoped_factory`             | A scoped service from a factory function                 |
| `scoped_with_key`            | A scoped service with a key                              |
| `scoped_with_key_factory`    | A scoped service using a key and factory function        |
| `transient`                  | A transient service                                      |
| `transient_factory`          | A transient service using a factory function             |
| `transient_as_self`          | A transient service for struct                           |
| `transient_with_key`         | A transient service with a key                           |
| `transient_with_key_factory` | A transient service using a key and factory function     |
| `transient_with_key_as_self` | A transient service with key for a struct                |
| `existing`                   | A singleton service from an existing instance            |
| `existing_as_self`           | A singleton service from an existing struct              |
| `existing_with_key`          | A singleton service from an existing instance with a key |
| `existing_with_key_as_self`  | A singleton service from an existing struct for a struct |

The following registers arbitrary traits and structs as services:

```rust
use di::*;
use std::rc::Rc;

pub struct Bar;

impl Bar {
    pub fn speak(&self) -> &str {
        "Hello world!"
    }
}

pub trait Foo {
    fn speak(&self) -> &str;
}

pub struct FooImpl {
    bar: Rc<Bar>
}

impl Foo for FooImpl {
    fn speak(&self) -> &str {
        self.bar.speak()
    }
}

fn run() {
    let provider = ServiceCollection::new()
        .add(transient_as_self::<Bar>().from(|_| Rc::new(Bar)))
        .add(singleton::<dyn Foo, FooImpl>()
             .from(|sp| Rc::new(FooImpl { bar: sp.get_required::<Bar>() })))
        .build_provider()
        .unwrap();
    let foo = provider.get_required::<dyn Foo>();

    println!("{}", foo.speak());
}
```

## Multiple Traits

In a few advanced scenarios, you might need a single service implementation to be mapped to multiple traits. This can be achieved, but ancillary service registrations must be explicit. There is currently no macro support for such a configuration.

Consider the following:

```rust
use di::*;

trait Service1 { }

trait Service2 { }

#[injectable]
struct MultiService;

impl Service1 for MultiService { }

impl Service2 for MultiService { }
```

It is now possible to register a single service with multiple traits as follows:

```rust
use crate::*;
use di::*;

let provider = ServiceCollection::new()
     // MultiService → Self
    .add(MultiService::singleton())
     // MultiService → dyn Service1
    .add(transient_factory::<dyn Service1>(|sp| sp.get_required::<MultiService>()))
     // MultiService → dyn Service2
    .add(transient_factory::<dyn Service2>(|sp| sp.get_required::<MultiService>()))
    .build_provider()
    .unwrap();

let svc1 = provider.get_required::<dyn Service1>();
let svc2 = provider.get_required::<dyn Service2>();
```

Care must be taken to ensure the lifetime of the primary service is compatible with the ancillary services. Each ancillary service should never live longer than the primary service. This configuration is most common when primary service is a **Singleton** or **Scoped**. If the primary service is **Transient**, the two independent registrations can be used instead.

## Keyed Services

Occasionally there are edge cases where the same service might need to be registered more than once for different contexts. A few scenarios include the same service, but with different lifetimes or different implementations of the same service in an otherwise ambiguous context.

Consider the following:

```rust
use di::*;

pub trait Thing : ToString;

#[injectable(Thing)]
pub struct Thing1;

impl Thing for Thing1;

impl ToString for Thing1 {
    fn to_string(&self) -> String {
        String::from(std::any::type_name::<Self>())
    }
}

#[injectable(Thing)]
pub struct Thing2;

impl Thing for Thing2;

impl ToString for Thing2 {
    fn to_string(&self) -> String {
        String::from(std::any::type_name::<Self>())
    }
}

#[injectable]
pub struct CatInTheHat {
    pub thing1: Ref<dyn Thing>,
    pub thing2: Ref<dyn Thing>,
}
```

`CatInTheHat` has two different dependencies of `dyn Thing`, but they are not expected to be same implementation. One solution would be to simply use `Thing1` and `Thing2` directly. Another solution would be to have complementary `dyn Thing1` and `dyn Thing2` traits. The final approach would be to used _keyed_ services.

A keyed service allows a service to be resolved in conjunction with a key. In many dependency injection frameworks, keyed services are supported by using a `String` as the key. That approach has a number of different problems. The `more-di` crate uses a type as a key instead. This approach provides the following advantages:

- No _magic strings_
- No attributes or other required metadata
- No hidden service location lookups
- No name collisions (because types are unique)
- No changes to `ServiceDescriptor`

In the previous code example there is nothing in place that restricts or defines which `dyn Thing` needs to be mapped. By definition, any `dyn Thing` _could_ be used, but a specific mapping is expected. To address that, we can refactor to use a `KeyedRef<K,T>`.

We also need to define some _keys_. A key is just a type used as a marker. A zero-sized `struct` is perfect for this case. For all intents and purposes, this struct acts like an enumeration. A key difference is that the required value is defined as part of the requested type, which an enumeration cannot do.

Let's perform a little refactoring:

```rust
use crate::*;
use di::*;

pub mod key {
    pub struct Thing1;
    pub struct Thing2;
}

#[injectable]
pub struct CatInTheHat {
    pub thing1: KeyedRef<key::Thing1, dyn Thing>,
    pub thing2: KeyedRef<key::Thing2, dyn Thing>,
}
```

Introducing a key means that we can no longer provide just any `dyn Thing`; a specific registration must be mapped. Although it is still possible to configure the wrong key, the key specified will never collide with a key defined by another crate. The compiler will enforce the key specified exists and the configuration will be validated when the `ServiceProvider` is created. Key types do not be need to be public or in nested modules unless you want them to be.

It's important to know that we only need the key at the injection call site. We can safely convert down to `Ref` if we use an injected constructor as follows:

```rust
use crate::*;
use di::*;

pub struct CatInTheHat {
    pub thing1: Ref<dyn Thing>,
    pub thing2: Ref<dyn Thing>,
}

#[injectable]
impl CatInTheHat {
    pub fn new(
        thing1: KeyedRef<key::Thing1, dyn Thing>,
        thing2: KeyedRef<key::Thing2, dyn Thing>) -> Self {
        // the key isn't useful after the correct service is injected
        Self {
            thing1: thing1.into(),
            thing2: thing2.into(),
        }
    }
}
```

Putting it all together, the service registration now looks like:

```rust
use crate::*;
use di::*;

let services = ServiceCollection::new()
    .add(Thing1::transient().with_key::<key::Thing1>())
    .add(Thing2::transient().with_key::<key::Thing2>())
    .add(CatInTheHat::singleton())
    .build_provider()
    .unwrap();

let cat = provider.get_required::<CatInTheHat>();

println!("Hi from {}", cat.thing1.to_string());
println!("Hi from {}", cat.thing2.to_string());
```

>If you're not using `#[injectable]`, the long-form [builder](#builder) functions provide variants that support specifying a key while creating a `ServiceDescriptor`.

Creating a keyed service explicitly is still possible and useful for some scenarios such as testing:

```rust
#[test]
fn setup_cat_in_the_hat() {
    // arrange
    let thing1 = KeyedRef::<key::Thing1, dyn Thing>::new(Ref::new(Thing1::default()));
    let thing2 = KeyedRef::<key::Thing2, dyn Thing>::new(Ref::new(Thing2::default()));
    let cat = CatInTheHat::new(thing1, thing2);

    // act
    let name = cat.thing1.to_string();

    // assert
    assert_eq!(&name, "crate::Thing1");
}
```