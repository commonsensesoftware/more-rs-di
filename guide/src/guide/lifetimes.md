# Service Lifetimes

A service can have the following lifetimes:

- **Transient** - a new instance is created every time it is requested
- **Singleton** - a single, new instance is created the first time it is requested
- **Scoped** - a new instance is created once per provider that it is requested from

A service with a **Singleton** lifetime, which depends on service that has a **Transient** lifetime will effectively promote that service to a **Singleton** lifetime as well. A service with a **Singleton** lifetime, which depends on service that has a **Scoped** lifetime will result in a validation error.

## Lifetime Management

The lifetime of a service determines how long a service lives relative to its owning `ServiceProvider`. When a `ServiceProvider` is dropped, all of the service instances it owns are also dropped. If a service instance somehow outlives its owning `ServiceProvider`, when the last of the owners is dropped, the service will also be dropped. The `ServiceProvider` itself will never leak any instantiated services.

There are scenarios where you might want a `ServiceProvider` to be _scoped_ for a limited amount of time. A HTTP request, for example, has a _Per-Request_ lifetime where you might want some services to be shared within the scope of the request (e.g. _scoped_) and then dropped. A new scope can be created via `create_scope` from any `ServiceProvider`.

Consider the following structures:

```rust
use di::*;

#[injectable]
pub struct Bar;

#[injectable]
pub struct Foo {
    bar: ServiceRef<Bar>
}
```

## Service Provider Singletons

A **Singleton** is created exactly once and lives for the lifetime of the root `ServiceProvider` no matter where it is actually first instantiated.

A **Singleton** in the root scope will be the same as a **Singleton** created in a nested scope.

```rust
use crate::*;
use di::*;

let provider = ServiceCollection::new()
    .add(Bar::transient())
    .add(Foo::singleton()
    .build_provider()
    .unwrap();

let foo1 = provider.get_required::<Foo>();

{
    let scope = provider.create_scope();
    let foo2 = scope.get_required::<Foo>();

    assert!(ServiceRef::ptr_eq(&foo1, &foo2));
}
```

In addition, if a **Singleton** is first created in a nested scoped, it will still be the same instance in the root scope.

```rust
use crate::*;
use di::*;

let provider = ServiceCollection::new()
    .add(Bar::transient())
    .add(Foo::singleton()
    .build_provider()
    .unwrap();
let foo1;

{
    let scope = provider.create_scope();
    foo1 = scope.get_required::<Foo>();
}

let foo2 = provider.get_required::<Foo>();
assert!(ServiceRef::ptr_eq(&foo1, &foo2));
```

## Service Provider Scopes

A **Scoped** service only lives as long as the lifetime of the owning service provider.

```rust
use crate::*;
use di::*;

let provider = ServiceCollection::new()
    .add(Bar::transient())
    .add(Foo::scoped()
    .build_provider()
    .unwrap();
let foo1 = provider.get_required::<Foo>();
let foo2;

{
    let scope = provider.create_scope();
    foo2 = scope.get_required::<Foo>();
    let foo3 = scope.get_required::<Foo>();

    // foo2 == foo3 because they have the same scope
    assert!(ServiceRef::ptr_eq(&foo2, &foo3));
} // ← all instances owned by 'scope' are dropped here

// 'foo2' outlived the scope because we held onto it for
// testing/demonstration purposes. as soon as it goes
// out of scope, it will be dropped. transient services
// behave in the same way and no instances are held by
// the ServiceProvider they were resolved from
//
// foo1 != foo2 because they came from different scopes
assert!(!ServiceRef::ptr_eq(&foo1, &foo2));
```