use di::{inject, injectable, lazy::Lazy, Ref};
use std::marker::PhantomData;

// demonstrates using a user-defined alias for Ref<T>
pub type ServiceRef<T> = Ref<T>;

pub struct Bar;

// make Bar injectable as Bar
// note: type can be omitted
#[injectable]
impl Bar {
    // 'new' function used by convention
    pub fn new() -> Self {
        Self {}
    }

    pub fn echo(&self) -> &str {
        "Success!"
    }
}

pub struct Foo {
    bar: ServiceRef<Bar>,
}

// make Foo injectable as Foo
// note: type explicitly specified
#[injectable(Foo)]
impl Foo {
    // identifies injection call site different from 'new'
    #[inject]
    pub fn create(bar: ServiceRef<Bar>) -> Self {
        Self { bar }
    }

    pub fn echo(&self) -> &str {
        self.bar.echo()
    }
}

pub struct GenericBar<T: Default> {
    _phantom: PhantomData<T>,
}

#[injectable]
impl<T: Default + 'static> GenericBar<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn echo(&self) -> T {
        T::default()
    }
}

pub struct GenericFoo<T>
where
    T: Default + 'static,
{
    bar: Ref<GenericBar<T>>,
}

#[injectable]
impl<T> GenericFoo<T>
where
    T: Default + 'static,
{
    pub fn new(bar: Ref<GenericBar<T>>) -> Self {
        Self { bar }
    }

    pub fn echo(&self) -> T {
        self.bar.echo()
    }
}

pub struct LazyFoo {
    bar: Lazy<Ref<Bar>>,
}

#[injectable]
impl LazyFoo {
    pub fn new(bar: Lazy<Ref<Bar>>) -> Self {
        Self { bar }
    }

    pub fn echo(&self) -> &str {
        self.bar.value().echo()
    }
}

#[injectable]
pub struct UnitStruct;

impl UnitStruct {
    pub fn echo(&self) -> &str {
        "Hello world!"
    }
}

#[injectable]
pub struct NormalStruct {
    pub unit: Ref<UnitStruct>,
    pub lazy: Lazy<Ref<UnitStruct>>,
    pub count: usize,
}

#[injectable]
pub struct Record(
    pub Ref<UnitStruct>,
    pub Lazy<Ref<UnitStruct>>,
    pub usize,
);
