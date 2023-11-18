// this file tests code generation for #[injectable] using
// keyed dependencies. if the project compiles, then code
// generation completed successfully

#![allow(dead_code)]

use di::{injectable, lazy::Lazy, KeyedRef, KeyedRefMut};
use std::sync::Mutex;

pub mod key {
    pub struct Key1;
    pub struct Key2;
    pub struct Key3;
}

#[injectable]
pub struct KeyedDep(usize);

#[injectable]
pub struct KeyedStruct {
    pub dep: KeyedRef<key::Key1, KeyedDep>,
}

#[injectable]
pub struct KeyedTupleStruct(pub KeyedRef<key::Key1, KeyedDep>);

#[injectable]
pub struct KeyedTupleGeneric<T: 'static>(pub KeyedRef<key::Key1, T>);

#[injectable]
pub struct KeyedStructRef {
    pub dep: KeyedRef<key::Key1, Mutex<KeyedDep>>,
}

pub struct KeyedStructImpl {
    dep: KeyedRef<key::Key1, KeyedDep>,
}

#[injectable]
impl KeyedStructImpl {
    fn new(dep: KeyedRef<key::Key1, KeyedDep>) -> Self {
        Self { dep }
    }
}

pub struct KeyedStructImplRef {
    dep: KeyedRef<key::Key1, Mutex<KeyedDep>>,
}

#[injectable]
impl KeyedStructImplRef {
    fn new(dep: KeyedRef<key::Key1, Mutex<KeyedDep>>) -> Self {
        Self { dep }
    }
}

#[injectable]
pub struct KeyedStructVec {
    pub vec: Vec<KeyedRef<key::Key1, KeyedDep>>,
}

pub struct KeyedStructIter {
    pub vec: Vec<KeyedRef<key::Key1, KeyedDep>>,
}

#[injectable]
impl KeyedStructIter {
    pub fn new(deps: impl Iterator<Item = KeyedRef<key::Key1, KeyedDep>>) -> Self {
        Self {
            vec: deps.collect(),
        }
    }
}

pub struct KeyedStructIterRef {
    pub vec: Vec<KeyedRef<key::Key1, Mutex<KeyedDep>>>,
}

#[injectable]
impl KeyedStructIterRef {
    pub fn new(deps: impl Iterator<Item = KeyedRef<key::Key1, Mutex<KeyedDep>>>) -> Self {
        Self {
            vec: deps.collect(),
        }
    }
}

#[injectable]
pub struct KeyedStructLazy {
    pub dep: Lazy<KeyedRef<key::Key2, KeyedDep>>,
}

pub struct KeyedStructLazyImpl {
    dep: Lazy<KeyedRef<key::Key2, KeyedDep>>,
}

#[injectable]
impl KeyedStructLazyImpl {
    fn new(dep: Lazy<KeyedRef<key::Key2, KeyedDep>>) -> Self {
        Self { dep }
    }
}

#[injectable]
pub struct KeyedStructMut {
    pub dep: KeyedRefMut<key::Key3, KeyedDep>
}