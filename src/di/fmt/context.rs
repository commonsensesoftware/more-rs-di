use crate::{
    ServiceDescriptor,
    ServiceLifetime::{self, *},
    Type,
};
use std::collections::HashMap;

pub struct Context<'a> {
    scope: ServiceLifetime,
    visited: Vec<&'a ServiceDescriptor>,
    lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>,
}

impl<'a> Context<'a> {
    pub fn new(lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>) -> Self {
        Self {
            scope: Transient,
            visited: Vec::new(),
            lookup,
        }
    }

    pub fn reset(&mut self, descriptor: &'a ServiceDescriptor) {
        self.scope = descriptor.lifetime();
        self.visited.clear();
        self.visited.push(descriptor);
    }

    pub fn lookup(&self, key: &Type) -> Option<&'a Vec<&'a ServiceDescriptor>> {
        self.lookup.get(key)
    }

    pub fn enter(&mut self, descriptor: &'a ServiceDescriptor) {
        if self.scope != Singleton && descriptor.lifetime() == Singleton {
            self.scope = Singleton;
        }

        self.visited.push(descriptor);
    }

    pub fn exit(&mut self) {
        self.visited.pop();

        for item in self.visited.iter().rev() {
            self.scope = item.lifetime();

            if self.scope == Singleton {
                return;
            }
        }

        self.scope = self.visited.last().map_or(Transient, |s| s.lifetime());
    }

    pub fn is_circular_ref(&self, descriptor: &ServiceDescriptor) -> bool {
        for item in self.visited.iter().rev() {
            if item.service_type() == descriptor.service_type() {
                return true;
            }
        }

        false
    }

    pub fn is_invalid_lifetime(&self, descriptor: &ServiceDescriptor) -> bool {
        self.scope == Singleton && descriptor.lifetime() == Scoped
    }
}
