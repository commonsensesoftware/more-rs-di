# Getting Started

The simplest way to get started is to install the crate using the default features.

```bash
cargo add more-di
```

## Example

Let's build the ubiquitous _Hello World_ application. The first thing we need to do is define some traits and structures. We'll use `#[injectable]`, which is highly convenient, but not strictly required.

```rust
use di::*;
use std::rc::Rc;

// let Bar make itself injectable as Bar
#[injectable]
pub struct Bar;

impl Bar {
    pub fn speak(&self) -> &str {
        "Hello world!"
    }
}

pub trait Foo {
    pub fn speak(&self) -> &str;
}

// let FooImpl make itself injectable as dyn Foo
#[injectable(Foo)]
pub struct FooImpl {
    bar: Rc<Bar> // ← assigned by #[injectable]
}

impl Foo for FooImpl {
    fn speak(&self) -> &str {
        self.bar.speak()
    }
}

pub trait Thing {}

// let Thing1 make itself injectable as dyn Thing
#[injectable(Thing)]
pub struct Thing1;

impl Thing for Thing1 {}

// let Thing2 make itself injectable as dyn Thing
#[injectable(Thing)]
pub struct Thing2;

impl Thing for Thing2 {}
```

Now that we have a few injectable types, we can build a basic application.

```rust
use crate::*;
use di::*;

fn main() {
    // create a collection of registered services. the order of
    // registration does not matter.
    let services = ServiceCollection::new()
        .add(FooImpl::transient())
        .add(Bar::singleton())
        .add(Thing1::transient());
        .add(Thing2::transient());

    // build an immutable service provider from the collection
    // of services. validation is performed here to ensure
    // the provider is a good state. if we're not, then a
    // ValidationError will indicate what the problems are.
    // if, for example, we forgot to register Bar, an error
    // would be returned indicating that Bar is missing.
    let provider = services.build_provider().unwrap();

    // get the requested service or panic
    let foo = provider::get_required::<dyn Foo>();

    println!("Foo says '{}'.", foo.speak());

    // get all of the requested services, which could be zero
    let things: Vec<_> = provider::get_all::<Thing>().collect();

    println!("Number of things: {}", things.len());
}
```