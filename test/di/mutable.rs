// this file tests code generation for #[injectable] using
// mutable dependencies. if the project compiles, then code
// generation completed successfully

#![allow(dead_code)]

use di::{injectable, lazy::Lazy, Ref, RefMut};
use std::sync::Mutex;

#[injectable]
pub struct MutDep(usize);

#[injectable]
pub struct MutStruct {
    pub dep: RefMut<MutDep>,
}

#[injectable]
pub struct MutTupleStruct(pub RefMut<MutDep>);

#[injectable]
pub struct MutTupleGeneric<T: 'static>(pub RefMut<T>);

#[injectable]
pub struct MutStructRef {
    pub dep: Ref<Mutex<MutDep>>,
}

pub struct MutStructImpl {
    dep: RefMut<MutDep>,
}

#[injectable]
impl MutStructImpl {
    fn new(dep: RefMut<MutDep>) -> Self {
        Self { dep }
    }
}

pub struct MutStructImplRef {
    dep: Ref<Mutex<MutDep>>,
}

#[injectable]
impl MutStructImplRef {
    fn new(dep: Ref<Mutex<MutDep>>) -> Self {
        Self { dep }
    }
}

#[injectable]
pub struct MutStructVec {
    pub vec: Vec<RefMut<MutDep>>,
}

pub struct MutStructIter {
    pub vec: Vec<RefMut<MutDep>>,
}

#[injectable]
impl MutStructIter {
    pub fn new(deps: impl Iterator<Item = RefMut<MutDep>>) -> Self {
        Self {
            vec: deps.collect(),
        }
    }
}

pub struct MutStructIterRef {
    pub vec: Vec<Ref<Mutex<MutDep>>>,
}

#[injectable]
impl MutStructIterRef {
    pub fn new(deps: impl Iterator<Item = Ref<Mutex<MutDep>>>) -> Self {
        Self {
            vec: deps.collect(),
        }
    }
}

#[injectable]
pub struct MutStructLazy {
    pub dep: Lazy<RefMut<MutDep>>,
}

pub struct MutStructLazyImpl {
    dep: Lazy<RefMut<MutDep>>,
}

#[injectable]
impl MutStructLazyImpl {
    fn new(dep: Lazy<RefMut<MutDep>>) -> Self {
        Self { dep }
    }
}