<!--
This file contains links that can be shared across pages. RustDoc links cannot currently
be used by mdBook directly. Links are stable on crates.io so we can centralize what is
required in this file, albeit manually.

REF: https://github.com/rust-lang/mdBook/issues/1356
REF: https://github.com/rust-lang/cargo/issues/739
REF: https://github.com/tag1consulting/goose/issues/320
-->

[`Ref`]: https://docs.rs/more-di/3.1.0/di/type.Ref.html
[`Type`]: https://docs.rs/more-di/3.1.0/di/struct.Type.html
[`ServiceCardinality`]: https://docs.rs/more-di/3.1.0/di/enum.ServiceCardinality.html
[`ServiceLifetime`]: https://docs.rs/more-di/3.1.0/di/enum.ServiceLifetime.html
[`ServiceDependency`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceDependency.html
[`ServiceDescriptor`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceDescriptor.html

[`ServiceCollection`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html
[`ServiceCollection::build_provider()`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html#method.build_provider
[`add`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html#method.add
[`try_add`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html#method.try_add
[`try_add_to_all`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html#method.try_add_to_all
[`try_add_all`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html#method.try_add_all
[`replace`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html#method.replace
[`try_replace`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceCollection.html#method.try_replace

[`ValidationError`]: https://docs.rs/more-di/3.1.0/di/struct.ValidationError.html
[`validate`]: https://docs.rs/more-di/3.1.0/di/fn.validate.html

[`ServiceProvider`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html
[`ScopedServiceProvider`]: https://docs.rs/more-di/3.1.0/di/struct.ScopedServiceProvider.html
[`create_scope`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.create_scope
[`get`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get
[`get_mut`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_mut
[`get_by_key`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_by_key
[`get_by_key_mut`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_by_key_mut
[`get_all`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_all
[`get_all_mut`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_all_mut
[`get_all_by_key`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_all_by_key
[`get_all_by_key_mut`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_all_by_key_mut
[`get_required`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_required
[`get_required_mut`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_required_mut
[`get_required_by_key`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_required_by_key
[`get_required_by_key_mut`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceProvider.html#method.get_required_by_key_mut

[`Injectable`]: https://docs.rs/more-di/3.1.0/di/trait.Injectable.html
[`InjectBuilder`]: https://docs.rs/more-di/3.1.0/di/struct.InjectBuilder.html
[`as_mut`]: https://docs.rs/more-di/3.1.0/di/struct.InjectBuilder.html#method.as_mut
[`with_key<TKey>`]: https://docs.rs/more-di/3.1.0/di/struct.InjectBuilder.html#method.with_key

[`ServiceDescriptorBuilder`]: https://docs.rs/more-di/3.1.0/di/struct.ServiceDescriptorBuilder.html
[`singleton`]: https://docs.rs/more-di/3.1.0/di/fn.singleton.html
[`singleton_as_self`]: https://docs.rs/more-di/3.1.0/di/fn.singleton_as_self.html
[`singleton_factory`]: https://docs.rs/more-di/3.1.0/di/fn.singleton_factory.html
[`singleton_with_key`]: https://docs.rs/more-di/3.1.0/di/fn.singleton_with_key.html
[`singleton_with_key_factory`]: https://docs.rs/more-di/3.1.0/di/fn.singleton_with_key_factory.html
[`scoped`]: https://docs.rs/more-di/3.1.0/di/fn.scoped.html
[`scoped_factory`]: https://docs.rs/more-di/3.1.0/di/fn.scoped_factory.html
[`scoped_with_key`]: https://docs.rs/more-di/3.1.0/di/fn.scoped_with_key.html
[`scoped_with_key_factory`]: https://docs.rs/more-di/3.1.0/di/fn.scoped_with_key_factory.html
[`transient`]: https://docs.rs/more-di/3.1.0/di/fn.transient.html
[`transient_factory`]: https://docs.rs/more-di/3.1.0/di/fn.transient_factory.html
[`transient_as_self`]: https://docs.rs/more-di/3.1.0/di/fn.transient_as_self.html
[`transient_with_key`]: https://docs.rs/more-di/3.1.0/di/fn.transient_with_key.html
[`transient_with_key_factory`]: https://docs.rs/more-di/3.1.0/di/fn.transient_with_key_factory.html
[`transient_with_key_as_self`]: https://docs.rs/more-di/3.1.0/di/fn.transient_with_key_as_self.html
[`existing`]: https://docs.rs/more-di/3.1.0/di/fn.existing.html
[`existing_as_self`]: https://docs.rs/more-di/3.1.0/di/fn.existing_as_self.html
[`existing_with_key`]: https://docs.rs/more-di/3.1.0/di/fn.existing_with_key.html
[`existing_with_key_as_self`]: https://docs.rs/more-di/3.1.0/di/fn.existing_with_key_as_self.html
[`exactly_one`]: https://docs.rs/more-di/3.1.0/di/fn.exactly_one.html
[`exactly_one_with_key`]: https://docs.rs/more-di/3.1.0/di/fn.exactly_one_with_key.html
[`zero_or_one`]: https://docs.rs/more-di/3.1.0/di/fn.zero_or_one.html
[`zero_or_one_with_key`]: https://docs.rs/more-di/3.1.0/di/fn.zero_or_one_with_key.html
[`zero_or_more`]: https://docs.rs/more-di/3.1.0/di/fn.zero_or_more.html
[`zero_or_more_with_key`]: https://docs.rs/more-di/3.1.0/di/fn.zero_or_more_with_key.html

[`lazy`]: https://docs.rs/more-di/3.1.0/di/lazy/index.html
[`Lazy`]: https://docs.rs/more-di/3.1.0/di/lazy/struct.Lazy.html
[`lazy::exactly_one`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.exactly_one.html
[`lazy::exactly_one_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.exactly_one_mut.html
[`lazy::exactly_one_with_key`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.exactly_one_with_key.html
[`lazy::exactly_one_with_key_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.exactly_one_with_key_mut.html
[`lazy::zero_or_one`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_one.html
[`lazy::zero_or_one_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_one_mut.html
[`lazy::zero_or_one_by_key`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_one_by_key.html
[`lazy::zero_or_one_by_key_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_one_by_key_mut.html
[`lazy::zero_or_more`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_more.html
[`lazy::zero_or_more_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_more_mut.html
[`lazy::zero_or_more_by_key`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_more_by_key.html
[`lazy::zero_or_more_by_key_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.zero_or_more_by_key_mut.html
[`lazy::missing`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.missing.html
[`lazy::missing_with_key`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.missing_with_key.html
[`lazy::empty`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.empty.html
[`lazy::empty_with_key`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.empty_with_key.html
[`lazy::init`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.init.html
[`lazy::init_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.init_mut.html
[`lazy::init_by_key`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.init_by_key.html
[`lazy::init_by_key_mut`]: https://docs.rs/more-di/3.1.0/di/lazy/fn.init_by_key_mut.html