use crate::traits::*;
use di::{inject, injectable, ServiceProvider, ServiceRef};

pub struct Container {
    provider: ServiceProvider,
}

// make Container injectable as Container
#[injectable]
impl Container {
    // note: 'ServiceProvider' is special. it will be cloned
    // rather than wrapped in ServiceRef, Rc, or Arc
    pub fn new(provider: ServiceProvider) -> Self {
        Self { provider }
    }

    pub fn foo(&self) -> ServiceRef<dyn Foo> {
        self.provider.get_required::<dyn Foo>()
    }
}

pub struct ScopedContainer {
    provider: ServiceProvider,
}

// make ScopedContainer injectable as ScopedContainer
#[injectable]
impl ScopedContainer {
    #[inject]
    pub fn init(provider: ServiceProvider) -> Self {
        Self {
            provider: provider.create_scope(),
        }
    }

    pub fn foo(&self) -> ServiceRef<dyn Foo> {
        self.provider.get_required::<dyn Foo>()
    }
}
