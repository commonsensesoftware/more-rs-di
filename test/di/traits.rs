use di::{inject, injectable, lazy::Lazy, KeyedRef, Ref};
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
    bar: Ref<dyn Bar>,
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
    pub fn create(bar: Ref<dyn Bar>) -> Self {
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
    bar: Lazy<Ref<dyn Bar>>,
}

impl Foo for OneLazyFoo {
    fn echo(&self) -> &str {
        self.bar.value().echo()
    }
}

#[injectable(Foo)]
impl OneLazyFoo {
    pub fn new(bar: Lazy<Ref<dyn Bar>>) -> Self {
        Self { bar }
    }
}

pub struct MaybeLazyFoo {
    bar: Lazy<Option<Ref<dyn Bar>>>,
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
    pub fn new(bar: Lazy<Option<Ref<dyn Bar>>>) -> Self {
        Self { bar }
    }
}

pub struct ManyLazyFoo {
    bars: Lazy<Vec<Ref<dyn Bar>>>,
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
    pub fn new(bars: Lazy<Vec<Ref<dyn Bar>>>) -> Self {
        Self { bars }
    }
}

pub mod key {
    pub struct Thing1;
    pub struct Thing2;
}

pub trait Thing: ToString {}

#[derive(Default)]
pub struct Thing1 {}

#[injectable(Thing)]
impl Thing1 {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct Thing2 {}

#[injectable(Thing)]
impl Thing2 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Thing for Thing1 {}

impl ToString for Thing1 {
    fn to_string(&self) -> String {
        String::from(std::any::type_name::<Self>())
    }
}

impl Thing for Thing2 {}

impl ToString for Thing2 {
    fn to_string(&self) -> String {
        String::from(std::any::type_name::<Self>())
    }
}

pub struct CatInTheHat {
    pub thing1: Ref<dyn Thing>,
    pub thing2: Ref<dyn Thing>,
}

#[injectable]
impl CatInTheHat {
    pub fn new(
        thing1: KeyedRef<key::Thing1, dyn Thing>,
        thing2: KeyedRef<key::Thing2, dyn Thing>,
    ) -> Self {
        Self {
            thing1: thing1.into(),
            thing2: thing2.into(),
        }
    }
}

pub struct Thingies {
    items: Vec<Ref<dyn Thing>>,
}

#[injectable]
impl Thingies {
    pub fn new(items: impl Iterator<Item = Ref<dyn Thing>>) -> Self {
        Self {
            items: items.collect(),
        }
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }
}

#[injectable(Foo)]
pub struct FooToo;

impl Foo for FooToo {
    fn echo(&self) -> &str {
        "Success!"
    }
}

#[injectable(Foo)]
pub struct FooTwo(Ref<dyn Bar>);

impl Foo for FooTwo {
    fn echo(&self) -> &str {
        self.0.echo()
    }
}

#[injectable]
pub struct MoreThingies {
    things: Vec<Ref<dyn Thing>>,
}

impl MoreThingies {
    pub fn count(&self) -> usize {
        self.things.len()
    }
}

pub trait Service1 {}

pub trait Service2 {}

#[injectable]
pub struct MultiService;

impl Service1 for MultiService {}

impl Service2 for MultiService {}
