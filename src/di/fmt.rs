mod context;
mod item;
mod renderer;

#[cfg(feature = "fmt")]
pub mod terminal;

pub mod text;

pub use context::Context;
pub use item::Item;
pub use renderer::Renderer;

use crate::{
    ServiceCardinality::{self, *},
    ServiceCollection, ServiceDescriptor,
    ServiceLifetime::*,
    Type,
};
use std::{
    collections::HashMap,
    fmt::{Formatter, Result},
};

pub fn write(services: &ServiceCollection, mut renderer: impl Renderer, f: &mut Formatter<'_>) -> Result {
    let count = services.len();

    if count == 0 {
        return Ok(());
    }

    let last = count - 1;
    let mut branches = Vec::<char>::new();
    let mut lookup = HashMap::with_capacity(count);

    for item in services {
        let key = item.service_type();
        let descriptors = lookup.entry(key).or_insert_with(Vec::new);
        descriptors.push(item);
    }

    let mut context = Context::new(&lookup);

    branches.push('│');
    branches.push(' ');

    for (index, descriptor) in services.iter().enumerate() {
        if index == last {
            renderer.write('└', f)?;
            branches[0] = ' ';
        } else if index == 0 {
            renderer.write('┌', f)?;
        } else {
            renderer.write('├', f)?;
        }

        renderer.write(' ', f)?;
        context.reset(descriptor);
        write_item(
            Item::One(descriptor),
            ExactlyOne,
            &mut context,
            0,
            &mut branches,
            f,
            &mut renderer,
        )?;

        if index != last {
            renderer.write_str("│\n", f)?;
        }
    }

    Ok(())
}

fn write_item(
    item: Item,
    cardinality: ServiceCardinality,
    context: &mut Context,
    depth: usize,
    branches: &mut Vec<char>,
    formatter: &mut Formatter,
    renderer: &mut impl Renderer,
) -> Result {
    match item {
        Item::One(sd) => {
            append_service(sd.service_type(), cardinality, renderer, formatter)?;

            if context.is_invalid_lifetime(sd) {
                renderer.error(
                    format!("⧗ {} [{:?}]", sd.implementation_type().name(), sd.lifetime()),
                    formatter,
                )?;
            } else {
                append_implementation(sd, renderer, formatter)?;
            }
        }
        Item::Many((ty, impl_count, _)) => {
            append_service(ty, cardinality, renderer, formatter)?;
            renderer.write_str(impl_count, formatter)?;
        }
        Item::Warning((sd, msg)) => {
            append_service(sd, cardinality, renderer, formatter)?;
            renderer.warn(msg, formatter)?;
        }
        Item::Error((sd, msg)) => {
            append_service(sd, cardinality, renderer, formatter)?;
            renderer.error(msg, formatter)?;
        }
    }

    renderer.write('\n', formatter)?;

    match item {
        Item::One(child) => traverse_dependencies(child, context, depth, branches, formatter, renderer),
        Item::Many((_, _, children)) => traverse_services(children, context, depth, branches, formatter, renderer),
        _ => Ok(()),
    }
}

fn append_service(
    ty: &Type,
    cardinality: ServiceCardinality,
    renderer: &mut impl Renderer,
    f: &mut Formatter,
) -> Result {
    let (type_, key) = Type::deconstruct(ty);

    if type_.starts_with("dyn") {
        renderer.keyword("dyn", f)?;
        renderer.write(' ', f)?;
        renderer.service(&type_[(type_.char_indices().nth(4).unwrap().0)..], f)?;
    } else {
        renderer.implementation(type_, f)?;
    }

    if cardinality == ServiceCardinality::ZeroOrMore {
        renderer.accent("*", f)?;
    } else if cardinality == ServiceCardinality::ZeroOrOne {
        renderer.accent("?", f)?;
    }

    if let Some(name) = key {
        renderer.write(' ', f)?;
        renderer.info("[⚿ ", f)?;
        renderer.info(name, f)?;
        renderer.info("]", f)?;
    }

    renderer.write_str(" → ", f)
}

fn append_implementation(item: &ServiceDescriptor, renderer: &mut impl Renderer, f: &mut Formatter) -> Result {
    renderer.implementation(item.implementation_type().name(), f)?;
    renderer.write(' ', f)?;

    match item.lifetime() {
        Scoped => renderer.info("[Scoped]", f),
        Singleton => renderer.info("[Singleton]", f),
        Transient => renderer.info("[Transient]", f),
    }
}

fn indent(branches: &mut Vec<char>, formatter: &mut Formatter, renderer: &mut impl Renderer, last: bool) -> Result {
    for branch in &*branches {
        renderer.write(*branch, formatter)?;
    }

    if last {
        renderer.write('└', formatter)?;
    } else {
        renderer.write('├', formatter)?;
    }

    renderer.write(' ', formatter)?;

    if last {
        branches.push(' ');
    } else {
        branches.push('│');
    }

    branches.push(' ');
    Ok(())
}

fn unindent(branches: &mut Vec<char>) {
    branches.pop();
    branches.pop();
}

fn traverse_dependencies(
    descriptor: &ServiceDescriptor,
    context: &mut Context,
    depth: usize,
    branches: &mut Vec<char>,
    formatter: &mut Formatter,
    renderer: &mut impl Renderer,
) -> Result {
    for (index, dependency) in descriptor.dependencies().iter().enumerate() {
        let type_ = dependency.injected_type();
        let cardinality = dependency.cardinality();
        let last = index == descriptor.dependencies().len() - 1;

        indent(branches, formatter, renderer, last)?;

        if let Some(children) = context.lookup(type_) {
            if cardinality == ZeroOrMore {
                write_item(
                    Item::Many((type_, &format!("Count: {}", children.len()), children)),
                    cardinality,
                    context,
                    depth + 1,
                    branches,
                    formatter,
                    renderer,
                )?;
            } else {
                for child in children {
                    let msg;
                    let item = if context.is_circular_ref(child) {
                        msg = format!("♺ {}", child.service_type().name());
                        Item::Error((child.service_type(), &msg))
                    } else {
                        Item::One(child)
                    };

                    context.enter(child);
                    write_item(item, cardinality, context, depth + 1, branches, formatter, renderer)?;
                    context.exit();
                }
            }
        } else {
            let item = match cardinality {
                ExactlyOne => Item::Error((type_, "‼ Missing")),
                ZeroOrOne => Item::Warning((type_, "▲ Missing")),
                ZeroOrMore => Item::Warning((type_, "▲ Count: 0")),
            };

            write_item(item, cardinality, context, depth + 1, branches, formatter, renderer)?;
        }

        unindent(branches);
    }

    Ok(())
}

fn traverse_services(
    descriptors: &[&ServiceDescriptor],
    context: &mut Context,
    depth: usize,
    branches: &mut Vec<char>,
    formatter: &mut Formatter,
    renderer: &mut impl Renderer,
) -> Result {
    for (index, descriptor) in descriptors.iter().enumerate() {
        let last = index == descriptors.len() - 1;

        indent(branches, formatter, renderer, last)?;
        write_item(
            Item::One(descriptor),
            ExactlyOne,
            context,
            depth + 1,
            branches,
            formatter,
            renderer,
        )?;
        unindent(branches);
    }

    Ok(())
}
