use crate::{
    Ref, ServiceCardinality::*, ServiceDependency, ServiceDescriptor, ServiceDescriptorBuilder, ServiceLifetime::*,
    ServiceProvider, Type,
};
use std::any::Any;
use std::mem::MaybeUninit;
use std::sync::OnceLock;

type Sdb<TSvc, TImpl> = ServiceDescriptorBuilder<TSvc, TImpl>;

macro_rules! service_from_type {
    ($($traits:tt)+) => {
        #[inline(always)]
        fn no_op(_: &ServiceProvider) -> Ref<dyn $($traits)+> {
            Ref::new(MaybeUninit::<Box<dyn $($traits)+>>::uninit())
        }

        /// Initializes a new singleton [ServiceDescriptorBuilder].
        #[inline]
        pub fn singleton<TSvc: ?Sized + $($traits)+, TImpl>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
            Sdb::new(Singleton, Type::of::<TImpl>())
        }

        /// Initializes a new keyed singleton [ServiceDescriptorBuilder].
        #[inline]
        pub fn singleton_with_key<TKey, TSvc: ?Sized + $($traits)+, TImpl>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
            Sdb::keyed::<TKey>(Singleton, Type::of::<TImpl>())
        }

        /// Initializes a new singleton [ServiceDescriptorBuilder].
        ///
        /// # Remarks
        ///
        /// This function maps a concrete type to itself rather than a trait.
        #[inline]
        pub fn singleton_as_self<T: $($traits)+>() -> ServiceDescriptorBuilder<T, T> {
            Sdb::new(Singleton, Type::of::<T>())
        }

        /// Initializes a new scoped [ServiceDescriptorBuilder].
        #[inline]
        pub fn scoped<TSvc: ?Sized + $($traits)+, TImpl>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
            Sdb::new(Scoped, Type::of::<TImpl>())
        }

        /// Initializes a new scoped keyed [ServiceDescriptorBuilder].
        #[inline]
        pub fn scoped_with_key<TKey, TSvc: ?Sized + $($traits)+, TImpl>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
            Sdb::keyed::<TKey>(Scoped, Type::of::<TImpl>())
        }

        /// Initializes a new transient [ServiceDescriptorBuilder].
        #[inline]
        pub fn transient<TSvc: ?Sized + $($traits)+, TImpl>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
            Sdb::new(Transient, Type::of::<TImpl>())
        }

        /// Initializes a new keyed transient [ServiceDescriptorBuilder].
        #[inline]
        pub fn transient_with_key<TKey, TSvc: ?Sized + $($traits)+, TImpl>() -> ServiceDescriptorBuilder<TSvc, TImpl> {
            Sdb::keyed::<TKey>(Transient, Type::of::<TImpl>())
        }

        /// Initializes a new transient [ServiceDescriptorBuilder].
        ///
        /// # Remarks
        ///
        /// This function maps a concrete type to itself rather than a trait.
        #[inline]
        pub fn transient_as_self<T: $($traits)+>() -> ServiceDescriptorBuilder<T, T> {
            Sdb::new(Transient, Type::of::<T>())
        }

        /// Initializes a new transient keyed [ServiceDescriptorBuilder].
        ///
        /// # Remarks
        ///
        /// This function maps a concrete type to itself rather than a trait.
        #[inline]
        pub fn transient_with_key_as_self<TKey, TSvc: $($traits)+>() -> ServiceDescriptorBuilder<TSvc, TSvc> {
            Sdb::keyed::<TKey>(Transient, Type::of::<TSvc>())
        }

        /// Creates a new singleton [ServiceDescriptor] for an existing service instance.
        ///
        /// # Arguments
        ///
        /// * `instance` - The existing service instance
        ///
        /// # Remarks
        ///
        /// This function maps an existing instance to a trait.
        #[inline]
        pub fn existing<TSvc: ?Sized + $($traits)+, TImpl>(instance: Box<TSvc>) -> ServiceDescriptor {
            ServiceDescriptor::new(
                Singleton,
                Type::of::<TSvc>(),
                Type::of::<TImpl>(),
                Vec::new(),
                OnceLock::from(Ref::new(Ref::<TSvc>::from(instance)) as Ref<dyn $($traits)+>),
                Ref::new(no_op),
            )
        }

        /// Creates a new singleton [ServiceDescriptor] for an existing service instance.
        ///
        /// # Arguments
        ///
        /// * `instance` - The existing service instance
        ///
        /// # Remarks
        ///
        /// This function maps an existing instance to itself rather than a trait.
        #[inline]
        pub fn existing_as_self<T: $($traits)+>(instance: T) -> ServiceDescriptor {
            ServiceDescriptor::new(
                Singleton,
                Type::of::<T>(),
                Type::of::<T>(),
                Vec::new(),
                OnceLock::from(Ref::new(Ref::from(instance)) as Ref<dyn $($traits)+>),
                Ref::new(no_op),
            )
        }

        /// Creates a new singleton [ServiceDescriptor] for an existing service instance with a key.
        ///
        /// # Arguments
        ///
        /// * `instance` - The existing service instance
        ///
        /// # Remarks
        ///
        /// This function maps an existing instance to a trait.
        #[inline]
        pub fn existing_with_key<TKey, TSvc: ?Sized + $($traits)+, TImpl>(instance: Box<TSvc>) -> ServiceDescriptor {
            ServiceDescriptor::new(
                Singleton,
                Type::keyed::<TKey, TSvc>(),
                Type::of::<TImpl>(),
                Vec::new(),
                OnceLock::from(Ref::new(Ref::<TSvc>::from(instance)) as Ref<dyn $($traits)+>),
                Ref::new(no_op),
            )
        }

        /// Creates a new singleton [ServiceDescriptor] for an existing service instance with a key.
        ///
        /// # Arguments
        ///
        /// * `instance` - The existing service instance
        ///
        /// # Remarks
        ///
        /// This function maps an existing instance to itself rather than a trait.
        #[inline]
        pub fn existing_with_key_as_self<TKey, TSvc: $($traits)+>(instance: TSvc) -> ServiceDescriptor {
            ServiceDescriptor::new(
                Singleton,
                Type::keyed::<TKey, TSvc>(),
                Type::of::<TSvc>(),
                Vec::new(),
                OnceLock::from(Ref::new(Ref::from(instance)) as Ref<dyn $($traits)+>),
                Ref::new(no_op),
            )
        }
    };
}

macro_rules! service_from_func {
    (($($traits:tt)+), ($($bounds:tt)+)) => {
        /// Initializes a new singleton [ServiceDescriptor].
        ///
        /// # Arguments
        ///
        /// * `factory` - The factory method used to create the service
        #[inline]
        pub fn singleton_factory<T: ?Sized + $($traits)+, F>(factory: F) -> ServiceDescriptor
        where
            F: Fn(&ServiceProvider) -> Ref<T> + $($bounds)+,
        {
            Sdb::<T, ()>::new(Singleton, Type::factory_of::<T>()).from(factory)
        }

        /// Initializes a new keyed singleton [ServiceDescriptor].
        ///
        /// # Arguments
        ///
        /// * `factory` - The factory method used to create the service
        #[inline]
        pub fn singleton_with_key_factory<TKey, TSvc: ?Sized + $($traits)+, F>(factory: F) -> ServiceDescriptor
        where
            F: Fn(&ServiceProvider) -> Ref<TSvc> + $($bounds)+,
        {
            Sdb::<TSvc, ()>::keyed::<TKey>(Singleton, Type::factory_of::<TSvc>()).from(factory)
        }

        /// Initializes a new scoped [ServiceDescriptor].
        ///
        /// # Arguments
        ///
        /// * `factory` - The factory method used to create the service
        #[inline]
        pub fn scoped_factory<T: ?Sized + $($traits)+, F>(factory: F) -> ServiceDescriptor
        where
            F: Fn(&ServiceProvider) -> Ref<T> + $($bounds)+,
        {
            Sdb::<T, ()>::new(Scoped, Type::factory_of::<T>()).from(factory)
        }

        /// Initializes a new keyed scoped [ServiceDescriptor].
        ///
        /// # Arguments
        ///
        /// * `factory` - The factory method used to create the service
        #[inline]
        pub fn scoped_with_key_factory<TKey, TSvc: ?Sized + $($traits)+, F>(factory: F) -> ServiceDescriptor
        where
            F: Fn(&ServiceProvider) -> Ref<TSvc> + $($bounds)+,
        {
            Sdb::<TSvc, ()>::keyed::<TKey>(Scoped, Type::factory_of::<TSvc>()).from(factory)
        }

        /// Initializes a new transient [ServiceDescriptor].
        ///
        /// # Arguments
        ///
        /// * `factory` - The factory method used to create the service
        #[inline]
        pub fn transient_factory<T: ?Sized + $($traits)+, F>(factory: F) -> ServiceDescriptor
        where
            F: Fn(&ServiceProvider) -> Ref<T> + $($bounds)+,
        {
            Sdb::<T, ()>::new(Transient, Type::factory_of::<T>()).from(factory)
        }

        /// Initializes a new keyed transient [ServiceDescriptor].
        ///
        /// # Arguments
        ///
        /// * `factory` - The factory method used to create the service
        #[inline]
        pub fn transient_with_key_factory<TKey, TSvc: ?Sized + $($traits)+, F>(factory: F) -> ServiceDescriptor
        where
            F: Fn(&ServiceProvider) -> Ref<TSvc> + $($bounds)+,
        {
            Sdb::<TSvc, ()>::keyed::<TKey>(Transient, Type::factory_of::<TSvc>()).from(factory)
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        service_from_type!(Any + Send + Sync);
        service_from_func!((Any + Send + Sync), (Send + Sync + 'static));
    } else {
        service_from_type!(Any);
        service_from_func!((Any), ('static));
    }
}

/// Creates a new [ServiceDependency] with a cardinality of exactly one (1:1).
#[inline]
pub fn exactly_one<T: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::of::<T>(), ExactlyOne)
}

/// Creates a new keyed [ServiceDependency] with a cardinality of exactly one (1:1).
#[inline]
pub fn exactly_one_with_key<TKey, TSvc: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::keyed::<TKey, TSvc>(), ExactlyOne)
}

/// Creates a new [ServiceDependency] with a cardinality of zero or one (0:1).
#[inline]
pub fn zero_or_one<T: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::of::<T>(), ZeroOrOne)
}

/// Creates a new keyed [ServiceDependency] with a cardinality of zero or one (0:1).
#[inline]
pub fn zero_or_one_with_key<TKey, TSvc: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::keyed::<TKey, TSvc>(), ZeroOrOne)
}

/// Creates a new [ServiceDependency] with a cardinality of zero or more (0:*).
#[inline]
pub fn zero_or_more<T: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::of::<T>(), ZeroOrMore)
}

/// Creates a new keyed [ServiceDependency] with a cardinality of zero or more (0:*).
#[inline]
pub fn zero_or_more_with_key<TKey, TSvc: Any + ?Sized>() -> ServiceDependency {
    ServiceDependency::new(Type::keyed::<TKey, TSvc>(), ZeroOrMore)
}
