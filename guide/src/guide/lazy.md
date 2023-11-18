# Lazy Initialization

>These features are only available if the **lazy** feature is activated

There are some scenarios where you know or have high reason to believe that a particular service composition will
be expensive to create. The requirement to eagerly load every injected service instance in such situations is
undesirable. There are several methods by which you can differ dependency resolution, including declaring a
parameter which would inject the `ServiceProvider` itself. Using the _Service Locator_ pattern in this manner
hides dependencies and is considered to be an _anti-pattern_. The **lazy** feature provides an out-of-the-box
facility to address this problem.

The `Lazy<T>` struct is a holder that resolves a service in a lazily evaluated manner. The `Lazy<T>` struct itself is owned by the struct it is injected into and the lifetime of the service resolved is unchanged. The key difference is that the injected service dependency is well-known at the call site, but its evaluation is differed.

Consider the following:

```rust
use di::*;

#[derive(Default)]
pub struct Expensive {
    // expensive stuff here
}

impl Expensive {
    pub fn do_work(&self) {
        // use expensive stuff
    }
}

pub struct Needy {
    expensive: Lazy<Ref<Expensive>>
}

impl Needy {
    pub fn new(expensive: Lazy<Ref<Expensive>>) -> Self {
        Self { expensive }
    }

    pub fn run(&self) {
        self.expensive.value().do_work()
    }
}
```

The `Needy` struct defines a `Lazy<T>` that wraps around a service dependency. This allows the service to
be evaluated on-demand and also keeps the `Expensive` struct visible as a required collaborator.

Despite being a generic type, `Lazy<T>` can only be created using the utility functions from the `lazy`
module as follows:

| Function                   | Resolution                                               |
| -------------------------- | -------------------------------------------------------- | 
| `exactly_one`              | A required service lazily                                |
| `exactly_one_mut`          | A required, mutable service lazily                       |
| `exactly_one_with_key`     | A required service with a key lazily                     |
| `exactly_one_with_key_mut` | A required, mutable service with key lazily              |
| `zero_or_one`              | An optional service lazily                               |
| `zero_or_one_mut`          | An optional, mutable service lazily                      |
| `zero_or_one_by_key`       | An optional, service with a key lazily                   |
| `zero_or_one_by_key_mut`   | An optional, mutable service with a key lazily           |
| `zero_or_more`             | One or more services lazily                              |
| `zero_or_more_mut`         | One or more mutable services lazily                      |
| `zero_or_more_by_key`      | One or more services with a key lazily                   |
| `zero_or_more_by_key_mut`  | One or more mutable services with a key lazily           |
| `missing`                  | Always resolves `None`                                   |
| `missing_with_key`         | Always resolves `None`                                   |
| `empty`                    | Always resolves `Vec::with_capacity(0)`                  |
| `empty_with_key`           | Always resolves `Vec::with_capacity(0)`                  |
| `init`                     | Initializes from an instance (ex: testing)               |
| `init_mut`                 | Initializes from a mutable instance (ex: testing)        |
| `init_by_key`              | Initializes from a keyed instance (ex: testing)          |
| `init_by_key_mut`          | Initializes from a mutable, keyed instance (ex: testing) |

`Lazy<T>` is a _special_ type which cannot be resolved directly from a `ServiceProvider`. You will
need construct one or more `Lazy<T>` registrations in the activation factory method. For example:

```rust
use crate::*;
use di::*;

fn main() {
    let provider = ServiceCollection::new()
        .add(singleton_as_self::<Expensive>()
             .from(|_| Rc::new(Expensive::default())));
        .add(singleton_as_self::<Needy>()
             .depends_on(exactly_one::<Expensive>())
             .from(|sp| Rc::new(Needy::new(lazy::exactly_one(sp.clone())))))
        .build_provider()
        .unwrap();
    let needy = provider.get_required::<Needy>();
    needy.run()
}
```

>Note: `singleton_as_self` and `exactly_one` are functions provided by the **builder** feature, while `lazy::exactly_one` is provided by the **lazy** feature.

When `#[injectable]` is used, it will generate the appropriate `lazy` function for the injected call site.