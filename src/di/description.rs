use cfg_if::cfg_if;

mod descriptor;
mod lifetime;

pub use descriptor::ServiceDescriptor;
pub use lifetime::ServiceLifetime;

cfg_if! {
    if #[cfg(feature = "builder")] {
        mod builder;

        pub use builder::ServiceDescriptorBuilder;
    }
}
