use crate::ServiceRef;
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

#[derive(Default)]
pub(crate) struct TestServiceImpl {
    pub value: usize,
}

#[derive(Default)]
pub(crate) struct TestService2Impl {
    pub value: usize,
}

pub(crate) struct OtherTestServiceImpl {
    _service: ServiceRef<dyn TestService>,
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
    pub fn new(service: ServiceRef<dyn TestService>) -> Self {
        Self { _service: service }
    }
}

impl OtherTestService for OtherTestServiceImpl {}

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