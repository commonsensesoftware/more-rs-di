{{#include links.md}}

# Service Validation

The consumers of a [`ServiceProvider`] expect that it is correctly configured and ready for use. There are edge cases,
however, which could lead to runtime failures or incorrect behavior such as:

- A required, dependent service that has not been registered
- A circular dependency, which will result in a stack overflow
- A service with a singleton lifetime that has a dependent service with a scoped lifetime

Intrinsic validation is provided to ensure those scenarios cannot happen. The [`ServiceCollection::build_provider()`] function will return `Result<ServiceProvider, ValidationError>`, which will either contain a valid [`ServiceProvider`] or a [`ValidationError`] that will detail all of the errors. From that point forward, the [`ServiceProvider`] will be considered semantically correct and safe to use. The same validation process can also be invoked imperatively on-demand by using the [`validate`] function on a given [`ServiceCollection`].

## Service Dependency

A [`ServiceDependency`] is a simple mapping that indicates the dependent [`Type`] and its [`ServiceCardinality`]. The set of dependencies for a service are defined by the arity of the arguments required to construct it, which is based on either its constructor arguments or all of its fields.

Rust does not have a Reflection API so the [`ServiceDescriptorBuilder`] cannot automatically determine the dependencies your service requires; therefore, validation is an explicit, opt-in capability. If you do not configure any dependencies for a [`ServiceDescriptor`], then no validation will occur.

While you can create a [`ServiceDependency`] in its long-form, there are several shorthand functions available to make it more succinct:

| Function                  | Dependency Type                                   |
| ------------------------- | ------------------------------------------------- |
| [`exactly_one`]           | Exactly one service of a specified type           |
| [`exactly_one_with_key`]  | Exactly one service of a specified type and key   |
| [`zero_or_one`]           | Zero or one services of a specified type          |
| [`zero_or_one_with_key`]  | Zero or one services of a specified type and key  |
| [`zero_or_more`]          | Zero or more services of a specified type         |
| [`zero_or_more_with_key`] | Zero or more services of a specified type and key |

>Note: These functions are only available if the **builder** feature is activated

Consider the following:

```rust
use di::*;

pub struct Bar;

pub struct Foo {
    pub bar: Ref<Bar>
}
```

Let's assume that we forgot to register `Bar`:

```rust
use di::*;

let services = Services::new()
    .add(transient_as_self::<Foo>().from(|_| Ref::new(Foo)))
    .build_provider()
    .unwrap(); // ← this will not panic

// the following panics because Bar is required and it has not be registered
let foo = provider.get_required::<Foo>();
```

While the mistake will be discovered at some point, it could be a long-time coming in a larger, more complex application. To alleviate that situation, we want to fail as early as possible.

Let's refactor the service registration with some dependencies:

```rust
use di::*;

let services = Services::new()
    .add(transient_as_self::<Foo>()
         .depends_on(exactly_one::<Bar>()) // ← indicate a Bar is required
         .from(|_| Ref::new(Foo)))
    .build_provider()
    .unwrap(); // ← now panics because Bar is an unregistered dependency
```

Specifying dependencies using their long-form, while a valid configuration, is verbose and tedious. The `#[injectable]` attribute will automatically build dependencies for each injected call site and is the preferred approach.