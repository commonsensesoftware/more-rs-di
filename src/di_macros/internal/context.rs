use syn::TypePath;

pub struct CallSiteContext<'a> {
    pub type_: &'a TypePath,
    pub optional: bool,
    pub many: bool,
    pub lazy: bool,
    pub iterator: bool,
    pub mutable: bool,
}

impl<'a> CallSiteContext<'a> {
    pub fn optional_of_many(&self) -> bool {
        self.optional && self.many
    }
}

#[derive(Default)]
pub struct CallSiteContextBuilder<'a> {
    type_: Option<&'a TypePath>,
    optional: bool,
    many: bool,
    lazy: bool,
    iterator: bool,
    mutable: bool,
}

impl<'a> CallSiteContextBuilder<'a> {
    pub fn has_type(&mut self, value: &'a TypePath) {
        self.type_ = Some(value)
    }

    pub fn is_optional(&mut self) {
        self.optional = true
    }

    pub fn has_many(&mut self) {
        self.many = true
    }

    pub fn is_lazy(&mut self) {
        self.lazy = true
    }

    pub fn is_iterator(&mut self) {
        self.iterator = true;
        self.many = true
    }

    pub fn is_mutable(&mut self) {
        self.mutable = true
    }

    pub fn build(self) -> CallSiteContext<'a> {
        CallSiteContext {
            type_: self.type_.unwrap(),
            optional: self.optional,
            many: self.many,
            lazy: self.lazy,
            iterator: self.iterator,
            mutable: self.mutable,
        }
    }
}