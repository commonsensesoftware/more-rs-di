use di::{inject, injectable, ServiceRef};

pub struct Bar;

// make Bar injectable as Bar
// note: type can be omitted
#[injectable]
impl Bar {
    // 'new' function used by convention
    pub fn new() -> Self {
        Self {}
    }

    pub fn echo(&self) -> &str {
        "Success!"
    }
}

pub struct Foo {
    bar: ServiceRef<Bar>
}

// make Foo injectable as Foo
// note: type explicitly specified
#[injectable(Foo)]
impl Foo {
    // identifies injection call site different from 'new'
    #[inject]
    pub fn create(bar: ServiceRef<Bar>) -> Self {
        Self { bar }
    }

    pub fn echo(&self) -> &str {
        self.bar.echo()
    }
}