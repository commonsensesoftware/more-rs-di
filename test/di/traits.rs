use di::{inject, injectable, ServiceRef};
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
