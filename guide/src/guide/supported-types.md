{{#include links.md}}

# Supported Types

While you are able to use any `trait` or `struct` you want for service registration, there are a few limitations as to how they can be resolved in order to satisfy the necessary service [lifetime](lifetimes.md) requirements.

There are a few basic forms in which you can request a service:

 1. `Rc` - a required service
 2. `Rc<RefCell>` - a required, mutable service
 3. `Option<Rc>` - an optional service (e.g. unregistered)
 4. `Option<Rc<RefCell>>` - an optional, mutable service (e.g. unregistered)
 5. `Iterator<Item = Rc>` - a sequence of services
 6. `Iterator<Item = Rc<RefCell>>` - a sequence of mutable services
 7. `Lazy` - a [lazy](lazy.md)-initialized service
 8. `KeyedRef<K,T>` - a required, keyed service
 9. `Option<KeyedRef<K,T>>` - an optional, keyed service
10. `Iterator<Item = KeyedRef<K,T>>` - a sequence of keyed services
11. `ServiceProvider` - the service provider itself
12. `ScopedServiceProvider` - a new, scoped service provider from the resolving instance

When the **async** feature is enabled, you **must** use `Arc` instead of `Rc`. To facilitate switching between synchronous and asynchronous contexts as well as making the syntax slightly more succinct, the following type aliases are provided:

- `Ref` = `Rc` or `Arc`
- `RefMut` = `Rc<RefCell>` or `Arc<RwLock>`
- `KeyedRefMut<K,T>` = `KeyedRef<K,RefCell>` or `KeyedRef<K,RwLock>`

## Macro Support

`#[injectable]` understands all of the forms listed above and supports mixed forms as well; for example, `Ref<RefCell>` is equivalent to `RefMut`. Since the results of an iterator must be owned, `#[injectable]` also supports using `Vec` at any injected call site that would otherwise use `Iterator`. The combinations `Option<Vec>` and `Vec<Option>`, however, are invalid.

Injecting `Iterator` is only supported when using an injection constructor. This is useful when you may not want to own the injected sequence of services or you want to use a collection other than `Vec`, such as `HashMap`.

For more information see [macros](macros.md).

## Custom Type Aliases

>These features are only available if the **alias** feature is activated

User-defined type aliases are usually not a problem for a library. When you use the `#[injectable]` attribute macro, however, it becomes important because the macro needs to understand the call site that it inspects so that it can generate the appropriate code. To overcome this limitation, you can define a custom mapping in the crate dependency configuration using the `aliases` table with the following keys:

| Key             | Default Alias |
|---------------- | --------------|
| `ref`           | `Ref`         |
| `ref-mut`       | `RefMut`      |
| `keyed-ref`     | `KeyedRef`    |
| `keyed-ref-mut` | `KeyedRefMut` |

For example, if you prefer the prefix `Svc`, you can remap them all as follows:

```toml
[dependencies.more-di.aliases]
ref = "Svc"
ref-mut = "SvcMut"
keyed-ref = "KeyedSvc"
keyed-ref-mut = "KeyedSvcMut"
```

You are still required define the aliases in your library or application:

```rust
type Svc<T> = Ref<T>;
type SvcMut<T> = RefMut<T>;
type KeyedSvc<K,T> = KeyedRef<K,T>;
type KeyedSvcMut<K,T> = KeyedRefMut<K,T>;
```

The only constraints are that the aliases you define must have the same number of generic type arguments. You are not required to use the built-in aliases.
If you prefer to directly alias the underlying type, that is also allowed:

```rust
type Svc<T> = std::rc::Rc<T>;
```

You are not required to alias every type. If all of your services are read-only and don't use keys, then the configuration can be simplified:

```toml
[dependencies]
more-di = { version = "2.0", features = ["alias"], aliases { ref = "Sr" } }
```

The type aliasing feature comes from the `more-di-macros` crate; however, the `more-di` crate is the dependency that most consumers will reference directly.
You can apply the `aliases` table to either the `more-di` or `more-di-macros` dependency configuration. If you specify both, `more-di` takes precedence.

### Backward Compatibility

In previous library versions, the primary type alias was `ServiceRef`. This added a lot of unnecessary verbosity that becomes prolific in your code. `Ref` is considerably
more succinct and even the qualified form `di::Ref` is shorter, yet unambiguous. To facilitate a smooth upgrade to newer versions, the `ServiceRef` type alias is
still automatically recognized without enabling the **alias** feature nor explicitly configuring the mapping with `key = "ServiceRef"`. The only requirement is that you
must define the alias in your library or application.

```rust
type ServiceRef<T> = Ref<T>;
```