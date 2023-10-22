# More Dependency Injection &emsp; ![CI](https://github.com/commonsensesoftware/more-rs-di/actions/workflows/ci.yml/badge.svg) [![Crates.io][crates-badge]][crates-url] [![MIT licensed][mit-badge]][mit-url] 

[crates-badge]: https://img.shields.io/crates/v/more-di.svg
[crates-url]: https://crates.io/crates/more-di
[mit-badge]: https://img.shields.io/badge/license-MIT-blueviolet.svg
[mit-url]: https://github.com/commonsensesoftware/more-rs-di/blob/main/LICENSE

More DI is a dependency injection (DI) for Rust. A `trait` or `struct` can be used as the injected type.

You may be looking for:

- [User Guide](https://commonsensesoftware.github.io/more-rs-di)
- [API Documentation](https://docs.rs/more-di)
- [Release Notes](https://github.com/commonsensesoftware/more-rs-di/releases)

## Features

This crate provides the following features:

- _default_ - Provides the abstractions for dependency injection, plus the **builder** and **inject** features
- **builder** - Provides functions for configuring service descriptors
- **async** - Provides features for using dependencies in an asynchronous context
- **inject** - Code-generates common injection scenarios
- **lazy** - Provides features for lazy-initialized service resolution

## Supported Lifetimes

A service can have the following lifetimes:

- **Transient** - a new instance is created every time it is requested
- **Singleton** - a single, new instance is created the first time it is requested
- **Scoped** - a new instance is created once per provider that it is requested from

## Dependency Injection in Action

Consider the following traits and structures.

>Proc macro attributes are not required, but they the fastest and simplest approach to add DI in your applications.

```rust
use di::*;
use std::rc::Rc;

trait Phrase {
    fn salutation(&self) -> &str;
}

#[injectable(Phrase)]
struct EnglishPhase;

impl Phrase for EnglishPhrase {
    fn salutation(&self) -> &str {
        "Hello world!"
    }
}

#[injectable]
struct Person {
    phase: Rc<dyn Phrase>,
}

impl Person {
    fn speak(&self) -> &str {
        self.phrase.salutation()
    }
}
```

This information can now be composed into a simple application:

```rust
use crate::*;
use di::*;

fn main() {
    let provider = ServiceCollection::new()
        .add(EnglishPhrase::singleton())
        .add(Person::transient())
        .build_provider()
        .unwrap();

    let person = provider.get_required::<Person>();

    println!("{}", person.speak());
}
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/more-rs-di/blob/main/LICENSE