# Service Resolution

Once you've registered, validated, and instantiated a `ServiceProvider`, you'll eventually need to get something out of it. This should typically only happen at the root of your application, but it might occur in other scenarios such as creating a new scope. The following functions are provided to resolve services:
     
| Function                  | Resolution                                           |
| ------------------------- | ---------------------------------------------------- |
| `get`                     | A single service, if it's registered                 |
| `get_mut`                 | A single, mutable service, if it's registered        |
| `get_by_key`              | A single service by key, if it's registered          |
| `get_by_key_mut`          | A single, mutable service by key, if it's registered |
| `get_all`                 | All services of the specified type                   |
| `get_all_mut`             | All mutable services of the specified type           |
| `get_all_by_key`          | All services of the specified type and key           |
| `get_all_by_key_mut`      | All mutable services of the specified type and key   |
| `get_required`            | A single service or panics                           |
| `get_required_mut`        | A single, mutable service or panics                  |
| `get_required_by_key`     | A single service by key or panics                    |
| `get_required_by_key_mut` | A single, mutable service by key or panics           |

## Examples

Consider the following structures:

```rust
use di::*;

trait Thing { }

struct Thing1;

impl Thing for Thing1 { }

struct Thing2;

impl Thing for Thing2 { }

struct Thing3;

impl Thing for Thing3 { }
```

Here are some ways that we can register and resolve them:

```rust
use crate::*;
use di::*;

let provider = ServiceCollection::new()
    .add(transient_as_self::<Thing1>().from(|_| Ref::new(Thing1)))
    .add(transient::<dyn Thing, Thing1>().from(|_| Ref::new(Thing1)))
    .add(transient::<dyn Thing, Thing2>().from(|_| Ref::new(Thing2)))
    .add(transient_mut::<dyn Thing, Thing3>().from(|_| RefMut::new(Thing3.into())))
    .build_provider()
    .unwrap();

// Some(Thing1)
assert!(provider.get::<Thing1>().is_some());

// None
assert!(provider.get::<Thing3>().is_none());

// RwLock<dyn Thing> → RwLock<Thing3>
assert!(provider.get_mut::<dyn Thing>().is_some());

// dyn Thing → Thing1
// dyn Thing → Thing2
assert_eq!(provider.get_all::<dyn Thing>().count(), 2);
```