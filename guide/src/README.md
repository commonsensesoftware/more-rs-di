# Introduction

`more-di` is a crate containing all of the fundamental abstractions for dependency injection (DI) in Rust.
Any `trait` or `struct` can be used as an injected service.

## Design Tenets

- Add, remove, or replace injected services
- Mitigate sequence coupling in service registration
- Support the most common service lifetimes
- Service registry exploration
- Separation of mutable service collection and immutable service provider
- Proc macros are a convenience, not a requirement
- Enable validation of required services, missing services, and circular references
- Support traits and structures defined in external crates
- Support asynchronous contexts
- Enable extensibility across crates

## Crate Features

This crate provides the following features:

- _default_ - Abstractions for dependency injection, plus the **builder** and **inject** features
- **builder** - Functions for configuring service descriptors
- **async** - Use dependencies in an asynchronous context
- **inject** - Code-generate common injection scenarios
- **lazy** - Lazy-initialize service resolution
- **fmt** - Additional output formatting
- **alias** - Use alternate type aliases

## Contributing

`more-di` is free and open source. You can find the source code on [GitHub](https://github.com/commonsensesoftware/more-rs-di)
and issues and feature requests can be posted on the [GitHub issue tracker](https://github.com/commonsensesoftware/more-rs-di/issues).
`more-di` relies on the community to fix bugs and add features: if you'd like to contribute, please read the
[CONTRIBUTING](https://github.com/commonsensesoftware/more-rs-di/blob/main/CONTRIBUTING.md) guide and consider opening
a [pull request](https://github.com/commonsensesoftware/more-rs-di/pulls).

## License

This project is licensed under the [MIT](https://github.com/commonsensesoftware/more-rs-di/blob/main/LICENSE) license.