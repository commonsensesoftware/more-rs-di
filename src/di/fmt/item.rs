use crate::{ServiceDescriptor, Type};

pub enum Item<'a> {
    One(&'a ServiceDescriptor),
    Many((&'a Type, &'a str, &'a Vec<&'a ServiceDescriptor>)),
    Warning((&'a Type, &'a str)),
    Error((&'a Type, &'a str)),
}
