use di::{inject, injectable, ServiceRef};

pub trait Foo {
    fn echo(&self) -> &str;
}

pub trait Bar {
    fn echo(&self) -> &str;
}

pub struct BarImpl;

impl Bar for BarImpl {
    fn echo(&self) -> &str {
        "Success!"
    }
}

// make BarImpl injectable as Bar
#[injectable(Bar)]
impl BarImpl {
    // 'new' function used by convention
    // 'pub' not required, which makes it only callable via DI
    fn new() -> Self {
        Self {}
    }
}

pub struct FooImpl {
    bar: ServiceRef<dyn Bar>,
}

impl Foo for FooImpl {
    fn echo(&self) -> &str {
        self.bar.echo()
    }
}

// make FooImpl injectable as Foo
#[injectable(Foo)]
impl FooImpl {
    // identifies injection call site different from 'new'
    #[inject]
    pub fn create(bar: ServiceRef<dyn Bar>) -> Self {
        Self { bar }
    }
}
