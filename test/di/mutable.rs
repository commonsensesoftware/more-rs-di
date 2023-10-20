// this file tests code generation for #[injectable] using
// mutable dependencies. if the project compiles, then code
// generation completed successfully

#![allow(dead_code)]

use di::{injectable, lazy::Lazy, ServiceRef, ServiceRefMut};
use std::sync::Mutex;

#[injectable]
pub struct MutDep(usize);

#[injectable]
pub struct MutStruct {
    pub dep: ServiceRefMut<MutDep>,
}

#[injectable]
pub struct MutTupleStruct(pub ServiceRefMut<MutDep>);

#[injectable]
pub struct MutTupleGeneric<T: 'static>(pub ServiceRefMut<T>);

#[injectable]
pub struct MutStructRef {
    pub dep: ServiceRef<Mutex<MutDep>>,
}

pub struct MutStructImpl {
    dep: ServiceRefMut<MutDep>,
}

#[injectable]
impl MutStructImpl {
    fn new(dep: ServiceRefMut<MutDep>) -> Self {
        Self { dep }
    }
}

pub struct MutStructImplRef {
    dep: ServiceRef<Mutex<MutDep>>,
}

#[injectable]
impl MutStructImplRef {
    fn new(dep: ServiceRef<Mutex<MutDep>>) -> Self {
        Self { dep }
    }
}

#[injectable]
pub struct MutStructVec {
    pub vec: Vec<ServiceRefMut<MutDep>>,
}

pub struct MutStructIter {
    pub vec: Vec<ServiceRefMut<MutDep>>,
}

#[injectable]
impl MutStructIter {
    pub fn new(deps: impl Iterator<Item = ServiceRefMut<MutDep>>) -> Self {
        Self {
            vec: deps.collect(),
        }
    }
}

pub struct MutStructIterRef {
    pub vec: Vec<ServiceRef<Mutex<MutDep>>>,
}

#[injectable]
impl MutStructIterRef {
    pub fn new(deps: impl Iterator<Item = ServiceRef<Mutex<MutDep>>>) -> Self {
        Self {
            vec: deps.collect(),
        }
    }
}

#[injectable]
pub struct MutStructLazy {
    pub dep: Lazy<ServiceRefMut<MutDep>>,
}

pub struct MutStructLazyImpl {
    dep: Lazy<ServiceRefMut<MutDep>>,
}

#[injectable]
impl MutStructLazyImpl {
    fn new(dep: Lazy<ServiceRefMut<MutDep>>) -> Self {
        Self { dep }
    }
}