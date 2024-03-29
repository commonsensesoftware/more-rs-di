// this file tests code generation for #[injectable] using
// mutable dependencies. if the project compiles, then code
// generation completed successfully

#![allow(dead_code)]

use di::{injectable, lazy::Lazy, Ref, RefMut};
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "async")] {
        use std::sync::RwLock;
    } else {
        use std::cell::RefCell;
    }
}

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

cfg_if! {
    if #[cfg(feature = "async")] {
        #[injectable]
        pub struct MutStructRef {
            pub dep: Ref<RwLock<MutDep>>,
        }
    } else {
        #[injectable]
        pub struct MutStructRef {
            pub dep: Ref<RefCell<MutDep>>,
        }
    }
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

cfg_if! {
    if #[cfg(feature = "async")] {
        pub struct MutStructImplRef {
            dep: Ref<RwLock<MutDep>>,
        }

        #[injectable]
        impl MutStructImplRef {
            fn new(dep: Ref<RwLock<MutDep>>) -> Self {
                Self { dep }
            }
        }
    } else {
        pub struct MutStructImplRef {
            dep: Ref<RefCell<MutDep>>,
        }
        
        #[injectable]
        impl MutStructImplRef {
            fn new(dep: Ref<RefCell<MutDep>>) -> Self {
                Self { dep }
            }
        }
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

cfg_if! {
    if #[cfg(feature = "async")] {
        pub struct MutStructIterRef {
            pub vec: Vec<Ref<RwLock<MutDep>>>,
        }
        
        #[injectable]
        impl MutStructIterRef {
            pub fn new(deps: impl Iterator<Item = Ref<RwLock<MutDep>>>) -> Self {
                Self {
                    vec: deps.collect(),
                }
            }
        }
    } else {
        pub struct MutStructIterRef {
            pub vec: Vec<Ref<RefCell<MutDep>>>,
        }
        
        #[injectable]
        impl MutStructIterRef {
            pub fn new(deps: impl Iterator<Item = Ref<RefCell<MutDep>>>) -> Self {
                Self {
                    vec: deps.collect(),
                }
            }
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