use crate::ServiceRef;
use std::{any::Any, borrow::Borrow, marker::PhantomData, ops::Deref, sync::Mutex};

/// Represents a holder for a keyed service.
#[derive(Clone)]
pub struct KeyedServiceRef<TKey, TSvc: Any + ?Sized> {
    service: ServiceRef<TSvc>,
    _key: PhantomData<TKey>,
}

/// Represents a holder for a keyed, mutable service.
pub type KeyedServiceRefMut<TKey, TSvc> = KeyedServiceRef<TKey, Mutex<TSvc>>;

impl<TKey, TSvc: Any + ?Sized> KeyedServiceRef<TKey, TSvc> {
    /// Initializes a new holder for the specified keyed service.
    ///
    /// * `service` - the keyed service reference the holder is for
    pub fn new(service: ServiceRef<TSvc>) -> Self {
        Self {
            service,
            _key: PhantomData,
        }
    }
}

impl<TKey, TSvc: Any + ?Sized> Into<ServiceRef<TSvc>> for KeyedServiceRef<TKey, TSvc> {
    fn into(self) -> ServiceRef<TSvc> {
        self.service
    }
}

impl<TKey, TSvc: Any + ?Sized> AsRef<TSvc> for KeyedServiceRef<TKey, TSvc> {
    fn as_ref(&self) -> &TSvc {
        self.service.as_ref()
    }
}

impl<TKey, TSvc: Any + ?Sized> Borrow<TSvc> for KeyedServiceRef<TKey, TSvc> {
    fn borrow(&self) -> &TSvc {
        self.service.borrow()
    }
}

impl<TKey, TSvc: Any + ?Sized> Deref for KeyedServiceRef<TKey, TSvc> {
    type Target = TSvc;

    fn deref(&self) -> &Self::Target {
        self.service.deref()
    }
}
