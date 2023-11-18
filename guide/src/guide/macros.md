# Macros

>These features are only available if the **inject** feature is activated

## Injectable

The `Injectable` trait provides the ability for a struct to be injected as a single trait that it implements or as itself.

```rust
pub trait Injectable: Sized {
    fn inject(lifetime: ServiceLifetime) -> InjectBuilder;

    fn singleton() -> InjectBuilder {
        Self::inject(ServiceLifetime::Singleton)
    }

    fn scoped() -> InjectBuilder {
        Self::inject(ServiceLifetime::Scoped)
    }

    fn transient() -> InjectBuilder {
        Self::inject(ServiceLifetime::Transient)
    }
}
```

Default implementations are provided each of the specific [lifetimes](lifetimes.md), thereby requiring only a single function to be implemented.

```rust
use di::*;

pub struct Bar;

pub struct Foo {
    bar: Ref<Bar>
}

impl Injectable for Bar {
  fn inject(lifetime: ServiceLifetime) -> InjectBuilder {
    InjectBuilder::new(
      Activator::new::<Self, Self>(
        |_| Ref::new(Self),
        |_| RefMut::new(Self.into()),
      ),
      lifetime,
    )
  }
}

impl Injectable for Foo {
  fn inject(lifetime: ServiceLifetime) -> InjectBuilder {
    InjectBuilder::new(
      Activator::new::<Self, Self>(
        |sp| Ref::new(Self { bar: sp.get_required::<Bar>() }),
        |sp| RefMut::new(Self { bar: sp.get_required::<Bar>() }.into()),
      ),
      lifetime,
    )
    .depends_on(
      ServiceDependency::new(
        Type::of::<Bar>(),
        ServiceCardinality::ExactlyOne))
  }
}
```

## ``#[injectable]``

While implementing `Injectable` _might_ be necessary or desired in a handful of scenarios, it is mostly tedious ceremony. If the injection call site were known, then it would be possible to provide the implementation automatically. This is exactly what the `#[injectable]` proc macro attribute provides.

Instead of implementing `Injectable` explicitly, the entire implementation can be achieved with a simple decorator:

```rust
use di::*;

#[injectable]
pub struct Bar;

#[injectable]
pub struct Foo {
    bar: Ref<Bar>
}
```

The `#[injectable]` attribute also supports a single, optional parameter value: the name of the injected trait. When no value is specified, it is assumed that the struct will be injected as itself. When a value is specified, a constructed `ServiceDescriptor` will map the struct to the specified trait.

```rust
use di::*;

pub trait Foo;

#[injectable(Foo)]  // dyn Foo → FooImpl
pub struct FooImpl;
```

### Injection Rules

The most basic form of injection allows `#[injectable]` to be applied to any struct or tuple struct, including generics.

>A generic type parameter requires a `'static` lifetime on it bounds due to the `Any` requirement; however, the actual type used will typically be coerced to a shorter lifetime.

```rust
use di::*;

#[injectable]
pub struct Simple;

#[injectable]
pub struct Tuple(pub Ref<Simple>);

#[injectable]
pub struct Generic<T: 'static> {
    value: Ref<T>,
}
```

If the target struct defines fields that are not meant to be injected, then it is assumed that those types implement `Default`. If they don't, then an error will occur.

```rust
use di::*;

#[injectable]
pub struct Complex {
    simple: Ref<Simple>, // ← ServiceProvider.get_required::<Simple>()
    counter: usize,      // ← Default::default()
}
```

This behavior might be undesirable, unsupported, or you may just want more control over initialization. To support that capability, `#[injectable]` can also be applied on a struct `impl` block. This is because that is the location where the function that will be used to construct the struct is expected to be found. This allows the attribute to inspect the injection call site of the function to build the proper implementation.

By default, `#[injectable]` will search for an associated function named `new`. The function does not need to be `pub`. This is a simple convention that works for most cases; however, if you want to use a different name, the intended function must be decorated with `#[inject]`. `#[inject]` simply indicates which function to use. If `new` and a decorated function are defined, the decorated function will take precedence. If multiple functions have `#[inject]` applied, an error will occur.

The following basic example uses a constructor:

```rust
use di::*;

pub struct Complex2 {
    simple: Ref<Simple>
    counter: usize
}

#[injectable]
impl Complex2 {
    // assumed to be the injection constructor by naming convention
    pub fn new(simple: Ref<Simple>) -> Self {
        Self {
            simple,
            counter: 0,
        }
    }
}
```

The following advanced example uses a custom constructor:

```rust
use di::*;

pub trait Input { }

pub trait Translator {
    fn translate(&self, text: &str, lang: &str) -> String;
}

pub trait Logger {
    fn log(&self, message: &str);
}


pub trait Runner {
    fn run(&self);
}

pub struct DefaultRunner {
    input: Ref<dyn Input>,
    translator: Option<Ref<dyn Input>>,
    loggers: Vec<Ref<dyn Logger>>,
}

#[injectable(Runner)]
impl DefaultRunner {
    #[inject] // ↓ use 'create' instead of inferring 'new'
    pub fn create(
        input: Ref<dyn Input>,
        translator: Option<Ref<dyn Input>>,
        loggers: Vec<Ref<dyn Logger>>) -> Self {
        Self {
            input,
            translator,
            loggers,
        }
    }
}

impl Runner for DefaultRunner {
    fn run(&self) {
        // TODO: implementation
    }
}
```

The `Injectable` implementation for `DefaultRunner` expands to:

```rust
impl Injectable for DefaultRunner {
    fn inject(lifetime: ServiceLifetime) -> InjectBuilder {
        InjectBuilder::new(
            Activator::new::<dyn Runner, Self>(
                |sp| Ref::new(
                        Self::create(
                            sp.get_required::<dyn Input>(),
                            sp.get::<dyn Input>(),
                            sp.get_all::<dyn Logger>().collect())),
                |sp| RefMut::new(
                        Self::create(
                            sp.get_required::<dyn Input>(),
                            sp.get::<dyn Input>(),
                            sp.get_all::<dyn Logger>().collect()).into()),
            )
        )
        .depends_on(
            ServiceDependency::new(
                Type::of::<dyn Input>(),
                ServiceCardinality::ExactlyOne))
        .depends_on(
            ServiceDependency::new(
                Type::of::<dyn Translator>(),
                ServiceCardinality::ZeroOrOne))
        .depends_on(
            ServiceDependency::new(
                Type::of::<dyn Logger>(),
                ServiceCardinality::ZeroOrMore))
    }
}
```

## Builder

`InjectBuilder` is similar to, but not exactly the same as, `ServiceDescriptorBuilder<TSvc,TImpl>`. `InjectBuilder` is part of the **inject** feature, while `ServiceDescriptorBuilder<TSvc,TImpl>` is part of the **builder** feature. The key implementation differences are a non-generic type, mutable construction (`as_mut`), and deferred key configuration (`with_key<TKey>`). This enables multiple registration scenarios with a single implementation. 

```rust
let provider = ServiceCollection::new()
    .add(Simple::transient())          // ← Ref<Simple>
    .add(Simple::transient().as_mut()) // ← RefMut<Simple>
    .add(Simple::transient()
         .with_key::<key::Alt>())      // ← KeyedRef<key::Alt, Simple>
    .add(Simple::transient()
         .with_key::<key::Alt>()
         .as_mut())                    // ← KeyedRefMut<key::Alt, Simple>
    .build_provider()
    .unwrap();
```