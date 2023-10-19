use di::{injectable, lazy::Lazy, ServiceRefMut};
use std::{rc::Rc, sync::Mutex};

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
pub struct MutStructRc {
    pub dep: Rc<Mutex<MutDep>>,
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

pub struct MutStructImplRc {
    dep: Rc<Mutex<MutDep>>,
}

#[injectable]
impl MutStructImplRc {
    fn new(dep: Rc<Mutex<MutDep>>) -> Self {
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

pub struct MutStructIterRc {
    pub vec: Vec<Rc<Mutex<MutDep>>>,
}

#[injectable]
impl MutStructIterRc {
    pub fn new(deps: impl Iterator<Item = Rc<Mutex<MutDep>>>) -> Self {
        Self {
            vec: deps.collect(),
        }
    }
}
