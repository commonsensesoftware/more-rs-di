use crate::{Ref, KeyedRef};
use std::any::type_name;
use std::env;
use std::fs::{remove_file, File};
use std::path::PathBuf;

#[cfg(feature = "async")]
use std::sync::Mutex;

pub(crate) fn new_temp_file(name: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(name);
    path.set_extension("tmp");
    File::create(&path).ok();
    path
}

pub(crate) trait TestService {
    fn value(&self) -> usize;
}

pub(crate) trait OtherTestService {}

pub(crate) trait AnotherTestService {}

#[derive(Default)]
pub(crate) struct TestServiceImpl {
    pub value: usize,
}

#[derive(Default)]
pub(crate) struct TestService2Impl {
    pub value: usize,
}

pub(crate) struct OtherTestServiceImpl {
    _service: Ref<dyn TestService>,
}

impl TestService for TestServiceImpl {
    fn value(&self) -> usize {
        self.value
    }
}

impl TestService for TestService2Impl {
    fn value(&self) -> usize {
        self.value
    }
}

impl OtherTestServiceImpl {
    pub fn new(service: Ref<dyn TestService>) -> Self {
        Self { _service: service }
    }
}

impl OtherTestService for OtherTestServiceImpl {}

pub(crate) struct AnotherTestServiceImpl {
    _service: Ref<dyn OtherTestService>,
}

impl AnotherTestServiceImpl {
    pub fn new(service: Ref<dyn OtherTestService>) -> Self {
        Self { _service: service }
    }
}

impl AnotherTestService for AnotherTestServiceImpl {}

pub(crate) struct Droppable {
    file: PathBuf,
}

impl Droppable {
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }
}

impl Drop for Droppable {
    fn drop(&mut self) {
        remove_file(&self.file).ok();
    }
}

#[cfg(feature = "async")]
#[derive(Default)]
pub(crate) struct TestAsyncServiceImpl {
    value: Mutex<usize>,
}

#[cfg(feature = "async")]
impl TestService for TestAsyncServiceImpl {
    fn value(&self) -> usize {
        let mut value = self.value.lock().unwrap();
        *value += 1;
        *value
    }
}

pub(crate) struct TestOptionalDepImpl {
    _service: Option<Ref<dyn TestService>>,
}

impl TestOptionalDepImpl {
    pub fn new(service: Option<Ref<dyn TestService>>) -> Self {
        Self { _service: service }
    }
}

impl OtherTestService for TestOptionalDepImpl {}

pub(crate) struct TestCircularDepImpl {
    _service: Ref<dyn TestService>,
}

impl TestCircularDepImpl {
    pub fn new(service: Ref<dyn TestService>) -> Self {
        Self { _service: service }
    }
}

impl TestService for TestCircularDepImpl {
    fn value(&self) -> usize {
        42
    }
}

pub(crate) struct TestAllKindOfProblems {
    _other: Ref<dyn OtherTestService>,
    _another: Ref<dyn AnotherTestService>,
}

impl TestAllKindOfProblems {
    pub fn new(
        other: Ref<dyn OtherTestService>,
        another: Ref<dyn AnotherTestService>,
    ) -> Self {
        Self {
            _other: other,
            _another: another,
        }
    }
}

impl TestService for TestAllKindOfProblems {
    fn value(&self) -> usize {
        42
    }
}

pub(crate) trait ServiceA {}

pub(crate) trait ServiceB {}

pub(crate) trait ServiceC {}

pub(crate) trait ServiceM {}

pub(crate) trait ServiceY {}

pub(crate) trait ServiceX {}

pub(crate) trait ServiceZ {}

pub(crate) struct ServiceAImpl {
    _m: Ref<dyn ServiceM>,
    _b: Ref<dyn ServiceB>,
}

impl ServiceAImpl {
    pub(crate) fn new(_m: Ref<dyn ServiceM>, _b: Ref<dyn ServiceB>) -> Self {
        Self { _m, _b }
    }
}

impl ServiceA for ServiceAImpl {}

pub(crate) struct ServiceBImpl {
    _m: Ref<dyn ServiceM>,
}

impl ServiceBImpl {
    pub(crate) fn new(_m: Ref<dyn ServiceM>) -> Self {
        Self { _m }
    }
}

impl ServiceB for ServiceBImpl {}

pub(crate) struct ServiceCImpl {
    _m: Ref<dyn ServiceM>,
}

impl ServiceCImpl {
    pub(crate) fn new(_m: Ref<dyn ServiceM>) -> Self {
        Self { _m }
    }
}
impl ServiceC for ServiceCImpl {}

pub(crate) struct ServiceCWithCircleRefToXImpl {
    _m: Ref<dyn ServiceM>,
    _x: Ref<dyn ServiceX>,
}

impl ServiceCWithCircleRefToXImpl {
    pub(crate) fn new(_m: Ref<dyn ServiceM>, _x: Ref<dyn ServiceX>) -> Self {
        Self { _m, _x }
    }
}

impl ServiceC for ServiceCWithCircleRefToXImpl {}

pub(crate) struct ServiceMImpl;

impl ServiceM for ServiceMImpl {}

pub(crate) struct ServiceYImpl {
    _m: Ref<dyn ServiceM>,
    _c: Ref<dyn ServiceC>,
}

impl ServiceYImpl {
    pub(crate) fn new(_m: Ref<dyn ServiceM>, _c: Ref<dyn ServiceC>) -> Self {
        Self { _m, _c }
    }
}

impl ServiceY for ServiceYImpl {}

pub(crate) struct ServiceXImpl {
    _m: Ref<dyn ServiceM>,
    _y: Ref<dyn ServiceY>,
}

impl ServiceX for ServiceXImpl {}

impl ServiceXImpl {
    pub(crate) fn new(_m: Ref<dyn ServiceM>, _y: Ref<dyn ServiceY>) -> Self {
        Self { _m, _y }
    }
}

pub(crate) struct ServiceZImpl {
    _m: Ref<dyn ServiceM>,
    _a: Ref<dyn ServiceA>,
    _x: Ref<dyn ServiceX>,
}

impl ServiceZImpl {
    pub(crate) fn new(
        _m: Ref<dyn ServiceM>,
        _a: Ref<dyn ServiceA>,
        _x: Ref<dyn ServiceX>,
    ) -> Self {
        Self { _m, _a, _x }
    }
}

impl ServiceZ for ServiceZImpl {}

pub(crate) trait Thing: ToString {}

#[derive(Default)]
pub(crate) struct Thing1;

#[derive(Default)]
pub(crate) struct Thing2;

#[derive(Default)]
pub(crate) struct Thing3;

impl Thing for Thing1 {}

impl ToString for Thing1 {
    fn to_string(&self) -> String {
        String::from(type_name::<Self>())
    }
}

impl Thing for Thing2 {}

impl ToString for Thing2 {
    fn to_string(&self) -> String {
        String::from(type_name::<Self>())
    }
}

impl Thing for Thing3 {}

impl ToString for Thing3 {
    fn to_string(&self) -> String {
        String::from(type_name::<Self>())
    }
}

pub(crate) mod key {
    pub(crate) struct Thingy;
    pub(crate) struct Thingies;
    pub(crate) struct Thing1;
    pub(crate) struct Thing2;
}

pub(crate) struct CatInTheHat {
}

impl CatInTheHat {
    pub fn new(
        _thing1: KeyedRef<key::Thing1, dyn Thing>,
        _thing2: Option<KeyedRef<key::Thing2, dyn Thing>>,
    ) -> Self {
        Self { }
    }
}
