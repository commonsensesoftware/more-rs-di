#![allow(dead_code)]

use crate::traits::*;
use di::*;

#[injectable]
struct A;

#[injectable]
struct B {
    a: Ref<A>,
}

#[injectable]
struct C {
    b: Ref<B>,
}

#[injectable]
struct D {
    a: Ref<A>,
    c: Option<Ref<C>>,
    cat: Option<Ref<CatInTheHat>>, // use: missing + optional
    thing: Ref<Thing1>,            // use: missing + required
    things: Vec<Ref<dyn Thing>>,
}

#[injectable]
struct E {
    f: Ref<F>, // use: circular ref
}

#[injectable]
struct F {
    e: Ref<E>, // use: circular ref
}

#[injectable(Thing)]
struct Thing3 {
    a: Ref<A>,
}

// use: invalid lifetime
impl Thing for Thing3 {}

impl ToString for Thing3 {
    fn to_string(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

#[cfg_attr(feature = "async", maybe_impl::traits(Send, Sync))]
trait Logger {}

#[injectable]
struct G {
    loggers: Vec<Ref<dyn Logger>>, // use: empty list
}

fn new_service_collection() -> ServiceCollection {
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

    services
}

#[rustfmt::skip]
const TEXT_PLAIN: &str =
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
   └ more_di_tests::format::A → more_di_tests::format::A [Singleton]\n";

#[rustfmt::skip]
const TEXT_TERMINAL: &str =
"┌ [38;2;78;201;176mmore_di_tests::format::A[0m → [38;2;78;201;176mmore_di_tests::format::A[0m [38;2;118;118;118m[Singleton][0m\n\
 │\n\
 ├ [38;2;78;201;176mmore_di_tests::format::B[0m → [38;2;78;201;176mmore_di_tests::format::B[0m [38;2;118;118;118m[Singleton][0m\n\
 │ └ [38;2;78;201;176mmore_di_tests::format::A[0m → [38;2;78;201;176mmore_di_tests::format::A[0m [38;2;118;118;118m[Singleton][0m\n\
 │\n\
 ├ [38;2;78;201;176mmore_di_tests::format::C[0m → [38;2;78;201;176mmore_di_tests::format::C[0m [38;2;118;118;118m[Transient][0m\n\
 │ └ [38;2;78;201;176mmore_di_tests::format::B[0m → [38;2;78;201;176mmore_di_tests::format::B[0m [38;2;118;118;118m[Singleton][0m\n\
 │   └ [38;2;78;201;176mmore_di_tests::format::A[0m → [38;2;78;201;176mmore_di_tests::format::A[0m [38;2;118;118;118m[Singleton][0m\n\
 │\n\
 ├ [38;2;78;201;176mmore_di_tests::format::D[0m → [38;2;78;201;176mmore_di_tests::format::D[0m [38;2;118;118;118m[Singleton][0m\n\
 │ ├ [38;2;78;201;176mmore_di_tests::format::A[0m → [38;2;78;201;176mmore_di_tests::format::A[0m [38;2;118;118;118m[Singleton][0m\n\
 │ ├ [38;2;78;201;176mmore_di_tests::format::C[0m[38;2;218;112;179m?[0m → [38;2;78;201;176mmore_di_tests::format::C[0m [38;2;118;118;118m[Transient][0m\n\
 │ │ └ [38;2;78;201;176mmore_di_tests::format::B[0m → [38;2;78;201;176mmore_di_tests::format::B[0m [38;2;118;118;118m[Singleton][0m\n\
 │ │   └ [38;2;78;201;176mmore_di_tests::format::A[0m → [38;2;78;201;176mmore_di_tests::format::A[0m [38;2;118;118;118m[Singleton][0m\n\
 │ ├ [38;2;78;201;176mmore_di_tests::traits::CatInTheHat[0m[38;2;218;112;179m?[0m → [38;2;220;220;170m▲ Missing[0m\n\
 │ ├ [38;2;78;201;176mmore_di_tests::traits::Thing1[0m → [38;2;231;72;86m‼ Missing[0m\n\
 │ └ [38;2;75;154;214mdyn[0m [38;2;158;211;163mmore_di_tests::traits::Thing[0m[38;2;218;112;179m*[0m → Count: 2\n\
 │   ├ [38;2;75;154;214mdyn[0m [38;2;158;211;163mmore_di_tests::traits::Thing[0m → [38;2;78;201;176mmore_di_tests::traits::Thing2[0m [38;2;118;118;118m[Transient][0m\n\
 │   └ [38;2;75;154;214mdyn[0m [38;2;158;211;163mmore_di_tests::traits::Thing[0m → [38;2;231;72;86m⧗ more_di_tests::format::Thing3 [Scoped][0m\n\
 │     └ [38;2;78;201;176mmore_di_tests::format::A[0m → [38;2;78;201;176mmore_di_tests::format::A[0m [38;2;118;118;118m[Singleton][0m\n\
 │\n\
 ├ [38;2;78;201;176mmore_di_tests::format::E[0m → [38;2;78;201;176mmore_di_tests::format::E[0m [38;2;118;118;118m[Transient][0m\n\
 │ └ [38;2;78;201;176mmore_di_tests::format::F[0m → [38;2;78;201;176mmore_di_tests::format::F[0m [38;2;118;118;118m[Transient][0m\n\
 │   └ [38;2;78;201;176mmore_di_tests::format::E[0m → [38;2;231;72;86m♺ more_di_tests::format::E[0m\n\
 │\n\
 ├ [38;2;78;201;176mmore_di_tests::format::F[0m → [38;2;78;201;176mmore_di_tests::format::F[0m [38;2;118;118;118m[Transient][0m\n\
 │ └ [38;2;78;201;176mmore_di_tests::format::E[0m → [38;2;78;201;176mmore_di_tests::format::E[0m [38;2;118;118;118m[Transient][0m\n\
 │   └ [38;2;78;201;176mmore_di_tests::format::F[0m → [38;2;231;72;86m♺ more_di_tests::format::F[0m\n\
 │\n\
 ├ [38;2;78;201;176mmore_di_tests::format::G[0m → [38;2;78;201;176mmore_di_tests::format::G[0m [38;2;118;118;118m[Transient][0m\n\
 │ └ [38;2;75;154;214mdyn[0m [38;2;158;211;163mmore_di_tests::format::Logger[0m[38;2;218;112;179m*[0m → [38;2;220;220;170m▲ Count: 0[0m\n\
 │\n\
 ├ [38;2;75;154;214mdyn[0m [38;2;158;211;163mmore_di_tests::traits::Thing[0m → [38;2;78;201;176mmore_di_tests::traits::Thing2[0m [38;2;118;118;118m[Transient][0m\n\
 │\n\
 └ [38;2;75;154;214mdyn[0m [38;2;158;211;163mmore_di_tests::traits::Thing[0m → [38;2;78;201;176mmore_di_tests::format::Thing3[0m [38;2;118;118;118m[Scoped][0m\n  \
   └ [38;2;78;201;176mmore_di_tests::format::A[0m → [38;2;78;201;176mmore_di_tests::format::A[0m [38;2;118;118;118m[Singleton][0m\n";

#[test]
fn debug_should_format_service_collection() {
    // arrange
    let services = new_service_collection();

    // act
    let output = format!("{services:?}");

    // assert
    assert_eq!(output, TEXT_PLAIN);
}

#[test]
fn display_should_format_service_collection() {
    // arrange
    let services = new_service_collection();

    // act
    let output = format!("{services}");

    // assert
    assert_eq!(output, TEXT_PLAIN);
}

#[test]
fn alt_display_should_format_service_collection() {
    // arrange
    let services = new_service_collection();

    // act
    let output = format!("{services:#}");

    // assert
    assert_eq!(output, TEXT_TERMINAL);
}
