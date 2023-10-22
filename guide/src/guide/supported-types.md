# Supported Types

While you are able to use any `trait` or `struct` you want for service registration, there are a few limitations as to how they can be resolved in order to satisfy the necessary service [lifetime](lifetimes.md) requirements.

There are a few basic forms in which you can request a service:

 1. `Rc<T>` - a required service
 2. `Rc<Mutex<T>>` - a required, mutable service
 3. `Option<Rc<T>>` - an optional service (e.g. unregistered)
 4. `Option<Rc<Mutex<T>>>` - an optional, mutable service (e.g. unregistered)
 5. `Iterator<Item = Rc<T>>` - a sequence of services
 6. `Iterator<Item = Rc<Mutex<T>>>` - a sequence of mutable services
 7. `Lazy<T>` - a [lazy](lazy.md)-initialized service
 8. `KeyedServiceRef<K,T>` - a required, keyed service
 9. `Option<KeyedServiceRef<K,T>>` - an optional, keyed service
10. `Iterator<Item = KeyedServiceRef<K,T>>` - a sequence of keyed services
11. `ServiceProvider` - the service provider itself
12. `ScopedServiceProvider` - a new, scoped service provider from the resolving instance

When the **async** feature is enabled, you **must** use `Arc` instead of `Rc`. To facilitate switching between synchronous and asynchronous contexts as well as making the syntax slightly more succinct, the following type aliases are provided:

- `ServiceRef<T>` = `Rc<T>` or `Arc<T>`
- `ServiceRefMut<T>` = `Rc<Mutex<T>>` or `Arc<Mutex<T>>`
- `KeyedServiceRefMut<K,T>` = `KeyedServiceRef<K,Mutex<T>>`

## Macro Support

`#[injectable]` understands all of the forms listed above and supports mixed forms as well; for example, `ServiceRef<Mutex<T>>` is equivalent to `ServiceRefMut<T>`. Since the results of an iterator must be owned, `#[injectable]` also supports using `Vec<T>` at any injected call site that would otherwise use `Iterator<Item>`. The combinations `Option<Vec<T>>` and `Vec<Option<T>>`, however, are invalid.

Injecting `Iterator<Item>` is only supported when using an injection constructor. This is useful when you may not want to own the injected sequence of services or you want to use a collection other than `Vec<T>`, such as `HashMap<K,V>`.

For more information see [macros](macros.md).