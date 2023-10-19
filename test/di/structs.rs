use di::{inject, injectable, lazy::Lazy, ServiceRef};
use std::marker::PhantomData;

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
    bar: ServiceRef<GenericBar<T>>,
}

#[injectable]
impl<T> GenericFoo<T>
where
    T: Default + 'static,
{
    pub fn new(bar: ServiceRef<GenericBar<T>>) -> Self {
        Self { bar }
    }

    pub fn echo(&self) -> T {
        self.bar.echo()
    }
}

pub struct LazyFoo {
    bar: Lazy<ServiceRef<Bar>>,
}

#[injectable]
impl LazyFoo {
    pub fn new(bar: Lazy<ServiceRef<Bar>>) -> Self {
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
    pub unit: ServiceRef<UnitStruct>,
    pub lazy: Lazy<ServiceRef<UnitStruct>>,
    pub count: usize,
}

#[injectable]
pub struct Record(
    pub ServiceRef<UnitStruct>,
    pub Lazy<ServiceRef<UnitStruct>>,
    pub usize,
);
