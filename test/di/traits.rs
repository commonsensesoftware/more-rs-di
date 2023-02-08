use di::{inject, injectable, lazy::Lazy, ServiceRef};
use std::fmt::Debug;

pub trait Foo {
    fn echo(&self) -> &str;
}

pub trait Bar {
    fn echo(&self) -> &str;
}

pub struct BarImpl;

impl Bar for BarImpl {
    fn echo(&self) -> &str {
        "Success!"
    }
}

// make BarImpl injectable as Bar
#[injectable(Bar)]
impl BarImpl {
    // 'new' function used by convention
    // 'pub' not required, which makes it only callable via DI
    fn new() -> Self {
        Self {}
    }
}

pub struct FooImpl {
    bar: ServiceRef<dyn Bar>,
}

impl Foo for FooImpl {
    fn echo(&self) -> &str {
        self.bar.echo()
    }
}

// make FooImpl injectable as Foo
#[injectable(Foo)]
impl FooImpl {
    // identifies injection call site different from 'new'
    #[inject]
    pub fn create(bar: ServiceRef<dyn Bar>) -> Self {
        Self { bar }
    }
}

pub trait Pair<TKey: Default + Debug, TValue: Default + Debug> {
    fn key(&self) -> &TKey;
    fn value(&self) -> &TValue;
}

pub struct PairImpl<TKey, TValue>
where
    TKey: Default + Debug + 'static,
    TValue: Default + Debug + 'static,
{
    key: TKey,
    value: TValue,
}

#[injectable(Pair<TKey, TValue>)]
impl<TKey, TValue> PairImpl<TKey, TValue>
where
    TKey: Default + Debug + 'static,
    TValue: Default + Debug + 'static,
{
    pub fn new() -> Self {
        Self {
            key: Default::default(),
            value: Default::default(),
        }
    }
}

impl<TKey, TValue> Pair<TKey, TValue> for PairImpl<TKey, TValue>
where
    TKey: Default + Debug,
    TValue: Default + Debug,
{
    fn key(&self) -> &TKey {
        &self.key
    }

    fn value(&self) -> &TValue {
        &self.value
    }
}

pub struct OneLazyFoo {
    bar: Lazy<ServiceRef<dyn Bar>>,
}

impl Foo for OneLazyFoo {
    fn echo(&self) -> &str {
        self.bar.value().echo()
    }
}

#[injectable(Foo)]
impl OneLazyFoo {
    pub fn new(bar: Lazy<ServiceRef<dyn Bar>>) -> Self {
        Self { bar }
    }
}

pub struct MaybeLazyFoo {
    bar: Lazy<Option<ServiceRef<dyn Bar>>>,
}

impl Foo for MaybeLazyFoo {
    fn echo(&self) -> &str {
        match self.bar.value() {
            Some(value) => value.echo(),
            _ => "",
        }
    }
}

#[injectable(Foo)]
impl MaybeLazyFoo {
    pub fn new(bar: Lazy<Option<ServiceRef<dyn Bar>>>) -> Self {
        Self { bar }
    }
}

pub struct ManyLazyFoo {
    bars: Lazy<Vec<ServiceRef<dyn Bar>>>,
}

impl Foo for ManyLazyFoo {
    fn echo(&self) -> &str {
        let value = self.bars.value();

        if value.is_empty() {
            ""
        } else {
            value[0].echo()
        }
    }
}

#[injectable(Foo)]
impl ManyLazyFoo {
    pub fn new(bars: Lazy<Vec<ServiceRef<dyn Bar>>>) -> Self {
        Self { bars }
    }
}
