#![allow(dead_code)]

use crate::traits::*;
use di::*;

#[injectable]
struct A;

#[injectable]
struct B {
    a: ServiceRef<A>,
}

#[injectable]
struct C {
    b: ServiceRef<B>,
}

#[injectable]
struct D {
    a: ServiceRef<A>,
    c: Option<ServiceRef<C>>,
    cat: Option<ServiceRef<CatInTheHat>>, // use: missing + optional
    thing: ServiceRef<Thing1>,            // use: missing + required
    things: Vec<ServiceRef<dyn Thing>>,
}

#[injectable]
struct E {
    f: ServiceRef<F>, // use: circular ref
}

#[injectable]
struct F {
    e: ServiceRef<E>, // use: circular ref
}

#[injectable(Thing)]
struct Thing3 {
    a: ServiceRef<A>,
}

// use: invalid lifetime
impl Thing for Thing3 {}

impl ToString for Thing3 {
    fn to_string(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

trait Logger {}

#[injectable]
struct G {
    loggers: Vec<ServiceRef<dyn Logger>>, // use: empty list
}

#[test]
fn debug_should_format_service_collection() {
    // arrange
    let mut services = ServiceCollection::new();

    services
        .add(A::singleton())
        .add(B::singleton())
        .add(C::transient())
        .add(D::singleton())
        .add(E::transient())
        .add(F::transient())
        .add(G::transient())
        .add(Thing2::transient())
        .add(Thing3::scoped());

    // act
    let output = format!("{:?}", services);

    // assert
    assert_eq!(
        output,
        "┌ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n\
         │\n\
         ├ more_di_tests::format::B → more_di_tests::format::B [Singleton]\n\
         │ └ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n\
         │\n\
         ├ more_di_tests::format::C → more_di_tests::format::C [Transient]\n\
         │ └ more_di_tests::format::B → more_di_tests::format::B [Singleton]\n\
         │   └ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n\
         │\n\
         ├ more_di_tests::format::D → more_di_tests::format::D [Singleton]\n\
         │ ├ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n\
         │ ├ more_di_tests::format::C? → more_di_tests::format::C [Transient]\n\
         │ │ └ more_di_tests::format::B → more_di_tests::format::B [Singleton]\n\
         │ │   └ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n\
         │ ├ more_di_tests::traits::CatInTheHat? → ▲ Missing\n\
         │ ├ more_di_tests::traits::Thing1 → ‼ Missing\n\
         │ └ dyn more_di_tests::traits::Thing* → Count: 2\n\
         │   ├ dyn more_di_tests::traits::Thing → more_di_tests::traits::Thing2 [Transient]\n\
         │   └ dyn more_di_tests::traits::Thing → ⧗ more_di_tests::format::Thing3 [Scoped]\n\
         │     └ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n\
         │\n\
         ├ more_di_tests::format::E → more_di_tests::format::E [Transient]\n\
         │ └ more_di_tests::format::F → more_di_tests::format::F [Transient]\n\
         │   └ more_di_tests::format::E → ♺ more_di_tests::format::E\n\
         │\n\
         ├ more_di_tests::format::F → more_di_tests::format::F [Transient]\n\
         │ └ more_di_tests::format::E → more_di_tests::format::E [Transient]\n\
         │   └ more_di_tests::format::F → ♺ more_di_tests::format::F\n\
         │\n\
         ├ more_di_tests::format::G → more_di_tests::format::G [Transient]\n\
         │ └ dyn more_di_tests::format::Logger* → ▲ Count: 0\n\
         │\n\
         ├ dyn more_di_tests::traits::Thing → more_di_tests::traits::Thing2 [Transient]\n\
         │\n\
         └ dyn more_di_tests::traits::Thing → more_di_tests::format::Thing3 [Scoped]\n  \
           └ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n"
    );
}
