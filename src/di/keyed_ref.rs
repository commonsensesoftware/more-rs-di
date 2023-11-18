use crate::Ref;
use std::{any::Any, borrow::Borrow, marker::PhantomData, ops::Deref, sync::Mutex};

/// Represents a holder for a keyed service.
#[derive(Clone)]
pub struct KeyedRef<TKey, TSvc: Any + ?Sized> {
    service: Ref<TSvc>,
    _key: PhantomData<TKey>,
}

/// Represents a holder for a keyed, mutable service.
pub type KeyedRefMut<TKey, TSvc> = KeyedRef<TKey, Mutex<TSvc>>;

impl<TKey, TSvc: Any + ?Sized> KeyedRef<TKey, TSvc> {
    /// Initializes a new holder for the specified keyed service.
    ///
    /// * `service` - the keyed service reference the holder is for
    pub fn new(service: Ref<TSvc>) -> Self {
        Self {
            service,
            _key: PhantomData,
        }
    }
}

impl<TKey, TSvc: Any + ?Sized> Into<Ref<TSvc>> for KeyedRef<TKey, TSvc> {
    fn into(self) -> Ref<TSvc> {
        self.service
    }
}

impl<TKey, TSvc: Any + ?Sized> AsRef<TSvc> for KeyedRef<TKey, TSvc> {
    fn as_ref(&self) -> &TSvc {
        self.service.as_ref()
    }
}

impl<TKey, TSvc: Any + ?Sized> Borrow<TSvc> for KeyedRef<TKey, TSvc> {
    fn borrow(&self) -> &TSvc {
        self.service.borrow()
    }
}

impl<TKey, TSvc: Any + ?Sized> Deref for KeyedRef<TKey, TSvc> {
    type Target = TSvc;

    fn deref(&self) -> &Self::Target {
        self.service.deref()
    }
}
