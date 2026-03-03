use crate::{
    validate, Ref, ServiceCardinality, ServiceDescriptor, ServiceDescriptorBuilder, ServiceLifetime, ServiceProvider,
    Type, ValidationError,
};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Formatter, Result as FormatResult, Write};
use std::iter::{DoubleEndedIterator, ExactSizeIterator};
use std::ops::Index;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;

#[cfg(feature = "fmt")]
use colored::Colorize;

#[cfg(feature = "fmt")]
use std::fmt::Display;

/// Represents a service collection.
#[derive(Default)]
pub struct ServiceCollection {
    items: Vec<ServiceDescriptor>,
}

impl ServiceCollection {
    /// Creates and returns a new instance of the service collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of elements in the collection.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Removes all elements from the collection.
    pub fn clear(&mut self) {
        self.items.clear()
    }

    /// Removes and returns the element at position index within the collection.
    ///
    /// # Argument
    ///
    /// * `index` - The index of the element to remove
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> ServiceDescriptor {
        self.items.remove(index)
    }

    /// Adds a service using the specified service descriptor.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [`ServiceDescriptor`](crate::ServiceDescriptor) to register
    pub fn add<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        self.items.push(descriptor.into());
        self
    }

    /// Adds a service using the specified service descriptor if the service has not already been registered.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [`ServiceDescriptor`](crate::ServiceDescriptor) to register
    pub fn try_add<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        let new_item = descriptor.into();
        let service_type = new_item.service_type();

        for item in &self.items {
            if item.service_type() == service_type {
                return self;
            }
        }

        self.items.push(new_item);
        self
    }

    /// Adds a service using the specified service descriptor if the service with same service and
    /// implementation type has not already been registered.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The [`ServiceDescriptor`](crate::ServiceDescriptor) to register
    pub fn try_add_to_all<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        let new_item = descriptor.into();
        let service_type = new_item.service_type();
        let implementation_type = new_item.implementation_type();

        if service_type == implementation_type {
            return self;
        }

        for item in &self.items {
            if item.service_type() == service_type && item.implementation_type() == implementation_type {
                return self;
            }
        }

        self.items.push(new_item);
        self
    }

    /// Adds the specified service descriptors if each of the services are not already registered
    /// with the same service and implementation type.
    ///
    /// # Arguments
    ///
    /// * `descriptors` - The [`ServiceDescriptor`](crate::ServiceDescriptor) sequence to register
    pub fn try_add_all(&mut self, descriptors: impl IntoIterator<Item = ServiceDescriptor>) -> &mut Self {
        for descriptor in descriptors {
            self.try_add_to_all(descriptor);
        }
        self
    }

    /// Removes the first service descriptor with the same service type and adds the replacement.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The replacement [`ServiceDescriptor`](crate::ServiceDescriptor)
    pub fn replace<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        let new_item = descriptor.into();
        let service_type = new_item.service_type();

        for i in 0..self.items.len() {
            if self.items[i].service_type() == service_type {
                self.items.remove(i);
                break;
            }
        }

        self.items.push(new_item);
        self
    }

    /// Adds or replaces a service with the specified descriptor if the service has not already been registered.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - The replacement [`ServiceDescriptor`](crate::ServiceDescriptor)
    pub fn try_replace<T: Into<ServiceDescriptor>>(&mut self, descriptor: T) -> &mut Self {
        self.try_add(descriptor)
    }

    /// Removes all specified descriptors of the specified type.
    pub fn remove_all<T: Any + ?Sized>(&mut self) -> &mut Self {
        let service_type = Type::of::<T>();

        for i in (0..self.items.len()).rev() {
            if self.items[i].service_type() == service_type {
                self.items.remove(i);
            }
        }

        self
    }

    /// Builds and returns a new [`ServiceProvider`](crate::ServiceProvider).
    pub fn build_provider(&self) -> Result<ServiceProvider, ValidationError> {
        validate(self)?;

        let mut services = HashMap::with_capacity(self.items.len());

        for item in &self.items {
            let key = item.service_type().clone();
            let descriptors = services.entry(key).or_insert_with(Vec::new);

            // note: dependencies are only interesting for validation. after a ServiceProvider
            // is created, no further validation occurs. prevent copying unnecessary memory
            // and allow it to potentially be freed if the ServiceCollection is dropped.
            descriptors.push(item.clone_with(false));
        }

        for values in services.values_mut() {
            values.shrink_to_fit();
        }

        services.shrink_to_fit();
        Ok(ServiceProvider::new(services))
    }

    /// Gets a read-only iterator for the collection
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ServiceDescriptor> + DoubleEndedIterator {
        self.items.iter()
    }

    /// Decorates an existing service descriptor with a new one that wraps the original.
    ///
    /// # Arguments
    ///
    /// * `activate` - The function that will be called to decorate the resolved service instance
    ///
    /// # Remarks
    ///
    /// This function will only decorate the last registered [ServiceDescriptor] for the specified service type. If
    /// there are multiple, the others are ignored. If you need decorate all services of a particular service type,
    /// consider using [Self::decorate_all] instead. If the service to be decorated is not registered, this function
    /// does nothing. The decorator [ServiceDescriptor] is created with the same lifetime as the original. The
    /// implementation type of the decorator is determined by the generic parameter `TImpl`. If the original and
    /// decorator implementation types are the same, the original, decorated [ServiceDescriptor] is not replaced to
    /// prevent infinite recursion.
    ///
    /// # Example
    ///
    /// ```
    /// use di::{injectable, Injectable, ServiceCollection, Ref};
    ///
    /// trait Counter {
    ///     fn count(&self) -> usize;
    /// }
    ///
    /// #[injectable(Counter)]
    /// struct SingleCount;
    ///
    /// impl Counter for SingleCount {
    ///     fn count(&self) -> usize {
    ///         1
    ///     }
    /// }
    ///
    /// struct DoubleCount(Ref<dyn Counter>);
    ///
    /// impl Counter for DoubleCount {
    ///     fn count(&self) -> usize {
    ///         self.0.count() * 2
    ///     }
    /// }
    ///
    /// let provider = ServiceCollection::new()
    ///     .add(SingleCount::transient())
    ///     .decorate::<dyn Counter, DoubleCount>(|_, decorated| Ref::new(DoubleCount(decorated)))
    ///     .build_provider()
    ///     .unwrap();
    /// let counter = provider.get_required::<dyn Counter>();
    ///
    /// assert_eq!(counter.count(), 2);
    /// ```
    pub fn decorate<TSvc: Any + ?Sized, TImpl>(
        &mut self,
        activate: impl Fn(&ServiceProvider, Ref<TSvc>) -> Ref<TSvc> + 'static,
    ) -> &mut Self {
        let service_type = Type::of::<TSvc>();

        for item in self.items.iter_mut().rev() {
            if item.service_type() != service_type {
                continue;
            }

            let impl_type = Type::of::<TImpl>();

            if item.implementation_type() == impl_type {
                return self;
            }

            let original = item.clone();
            let builder = ServiceDescriptorBuilder::<TSvc, TImpl>::new(original.lifetime(), impl_type);

            *item = builder.from(move |sp| {
                let decorated = original.get(sp).downcast_ref::<Ref<TSvc>>().unwrap().clone();
                activate(sp, decorated)
            });

            break;
        }

        self
    }

    /// Decorates all existing service descriptors with a new one that wraps the original.
    ///
    /// # Arguments
    ///
    /// * `activate` - The function that will be called to decorate the resolved service instance
    ///
    /// # Remarks
    ///
    /// This function decorates all registered [ServiceDescriptor] for the specified service type. If there are none,
    /// this function does nothing. The decorator [ServiceDescriptor] is created with the same lifetime as the original.
    /// If the original, decorated [ServiceDescriptor] is the same the decorator type, it is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use di::{injectable, Injectable, ServiceCollection, Ref};
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// trait Feature {
    ///     fn show(&self);
    /// }
    ///
    /// #[injectable(Feature)]
    /// struct Feature1;
    ///
    /// impl Feature for Feature1 {
    ///     fn show(&self) {
    ///     }
    /// }
    ///
    /// #[injectable(Feature)]
    /// struct Feature2;
    ///
    /// impl Feature for Feature2 {
    ///     fn show(&self) {
    ///     }
    /// }
    ///
    /// #[injectable]
    /// struct Tracker(AtomicUsize);
    ///
    /// impl Tracker {
    ///     fn track(&self) {
    ///         self.0.fetch_add(1, Ordering::Relaxed);
    ///     }
    ///
    ///     fn count(&self) -> usize {
    ///         self.0.load(Ordering::Relaxed)
    ///     }
    /// }
    ///
    /// struct FeatureTracker {
    ///     feature: Ref<dyn Feature>,
    ///     tracker: Ref<Tracker>,
    /// };
    ///
    /// impl Feature for FeatureTracker {
    ///     fn show(&self) {
    ///         self.tracker.track();
    ///         self.feature.show();
    ///     }
    /// }
    ///
    /// let provider = ServiceCollection::new()
    ///     .add(Tracker::singleton())
    ///     .try_add_to_all(Feature1::transient())
    ///     .try_add_to_all(Feature2::transient())
    ///     .decorate_all::<dyn Feature, FeatureTracker>(|sp, decorated| {
    ///         Ref::new(FeatureTracker { feature: decorated, tracker: sp.get_required::<Tracker>() })
    ///     })
    ///     .build_provider()
    ///     .unwrap();
    /// let features = provider.get_all::<dyn Feature>();
    /// let tracker = provider.get_required::<Tracker>();
    ///
    /// for feature in features {
    ///     feature.show();
    /// }
    ///
    /// assert_eq!(tracker.count(), 2);
    /// ```
    pub fn decorate_all<TSvc: Any + ?Sized, TImpl>(
        &mut self,
        activate: impl Fn(&ServiceProvider, Ref<TSvc>) -> Ref<TSvc> + 'static,
    ) -> &mut Self {
        let service_type = Type::of::<TSvc>();
        let func = Ref::new(activate);

        for item in self.items.iter_mut() {
            let impl_type = Type::of::<TImpl>();

            if item.service_type() != service_type || item.implementation_type() == impl_type {
                continue;
            }

            let original = item.clone();
            let activate = func.clone();
            let builder = ServiceDescriptorBuilder::<TSvc, TImpl>::new(original.lifetime(), impl_type);

            *item = builder.from(move |sp| {
                let decorated = original.get(sp).downcast_ref::<Ref<TSvc>>().unwrap().clone();
                (activate)(sp, decorated)
            });
        }

        self
    }
}

impl<'a> IntoIterator for &'a ServiceCollection {
    type Item = &'a ServiceDescriptor;
    type IntoIter = Iter<'a, ServiceDescriptor>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a mut ServiceCollection {
    type Item = &'a mut ServiceDescriptor;
    type IntoIter = IterMut<'a, ServiceDescriptor>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut()
    }
}

impl IntoIterator for ServiceCollection {
    type Item = ServiceDescriptor;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl Index<usize> for ServiceCollection {
    type Output = ServiceDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl std::fmt::Debug for ServiceCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        print(self, TextRenderer, f)
    }
}

#[cfg(feature = "fmt")]
impl Display for ServiceCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        print(self, TerminalRenderer, f)
    }
}

trait Renderer {
    fn write(&mut self, ch: char, f: &mut Formatter<'_>) -> FormatResult {
        f.write_char(ch)
    }

    fn write_str<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn service<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn implementation<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn keyword<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn info<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn warn<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn error<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn accent<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }
}

#[derive(Default)]
struct TextRenderer;

impl Renderer for TextRenderer {}

#[cfg(feature = "fmt")]
#[derive(Default)]
struct TerminalRenderer;

#[cfg(feature = "fmt")]
impl Renderer for TerminalRenderer {
    fn write(&mut self, ch: char, f: &mut Formatter<'_>) -> FormatResult {
        f.write_char(ch)
    }

    fn write_str<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        f.write_str(text.as_ref())
    }

    fn keyword<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        text.as_ref().truecolor(75, 154, 214).fmt(f)
    }

    fn service<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        text.as_ref().truecolor(158, 211, 163).fmt(f)
    }

    fn implementation<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        text.as_ref().truecolor(78, 201, 176).fmt(f)
    }

    fn info<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        text.as_ref().truecolor(118, 118, 118).fmt(f)
    }

    fn warn<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        text.as_ref().truecolor(220, 220, 170).fmt(f)
    }

    fn error<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        text.as_ref().truecolor(231, 72, 86).fmt(f)
    }

    fn accent<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> FormatResult {
        text.as_ref().truecolor(218, 112, 179).fmt(f)
    }
}

enum PrintItem<'a> {
    One(&'a ServiceDescriptor),
    Many((&'a Type, &'a str, &'a Vec<&'a ServiceDescriptor>)),
    Warning((&'a Type, &'a str)),
    Error((&'a Type, &'a str)),
}

struct PrintContext<'a> {
    scope: ServiceLifetime,
    visited: Vec<&'a ServiceDescriptor>,
    lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>,
}

impl<'a> PrintContext<'a> {
    fn new(lookup: &'a HashMap<&'a Type, Vec<&'a ServiceDescriptor>>) -> Self {
        Self {
            scope: ServiceLifetime::Transient,
            visited: Vec::new(),
            lookup,
        }
    }

    fn reset(&mut self, descriptor: &'a ServiceDescriptor) {
        self.scope = descriptor.lifetime();
        self.visited.clear();
        self.visited.push(descriptor);
    }

    fn lookup(&self, key: &Type) -> Option<&'a Vec<&'a ServiceDescriptor>> {
        self.lookup.get(key)
    }

    fn enter(&mut self, descriptor: &'a ServiceDescriptor) {
        if self.scope != ServiceLifetime::Singleton && descriptor.lifetime() == ServiceLifetime::Singleton {
            self.scope = ServiceLifetime::Singleton;
        }

        self.visited.push(descriptor);
    }

    fn exit(&mut self) {
        self.visited.pop();

        for item in self.visited.iter().rev() {
            self.scope = item.lifetime();

            if self.scope == ServiceLifetime::Singleton {
                return;
            }
        }

        self.scope = self.visited.last().map_or(ServiceLifetime::Transient, |s| s.lifetime());
    }

    fn is_circular_ref(&self, descriptor: &ServiceDescriptor) -> bool {
        for item in self.visited.iter().rev() {
            if item.service_type() == descriptor.service_type() {
                return true;
            }
        }

        false
    }

    fn is_invalid_lifetime(&self, descriptor: &ServiceDescriptor) -> bool {
        self.scope == ServiceLifetime::Singleton && descriptor.lifetime() == ServiceLifetime::Scoped
    }
}

fn print<R: Renderer>(services: &ServiceCollection, mut renderer: R, f: &mut Formatter<'_>) -> FormatResult {
    let count = services.items.len();

    if count == 0 {
        return Ok(());
    }

    let last = count - 1;
    let mut branches = Vec::<char>::new();
    let mut lookup = HashMap::with_capacity(count);

    for item in &services.items {
        let key = item.service_type();
        let descriptors = lookup.entry(key).or_insert_with(Vec::new);
        descriptors.push(item);
    }

    let mut context = PrintContext::new(&lookup);

    branches.push('│');
    branches.push(' ');

    for (index, descriptor) in services.items.iter().enumerate() {
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

        print_item(
            PrintItem::One(descriptor),
            ServiceCardinality::ExactlyOne,
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

fn print_item<R: Renderer>(
    item: PrintItem,
    cardinality: ServiceCardinality,
    context: &mut PrintContext,
    depth: usize,
    branches: &mut Vec<char>,
    formatter: &mut Formatter,
    renderer: &mut R,
) -> FormatResult {
    match item {
        PrintItem::One(sd) => {
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
        PrintItem::Many((ty, impl_count, _)) => {
            append_service(ty, cardinality, renderer, formatter)?;
            renderer.write_str(impl_count, formatter)?;
        }
        PrintItem::Warning((sd, msg)) => {
            append_service(sd, cardinality, renderer, formatter)?;
            renderer.warn(msg, formatter)?;
        }
        PrintItem::Error((sd, msg)) => {
            append_service(sd, cardinality, renderer, formatter)?;
            renderer.error(msg, formatter)?;
        }
    }

    renderer.write('\n', formatter)?;

    match item {
        PrintItem::One(child) => traverse_dependencies(child, context, depth, branches, formatter, renderer),
        PrintItem::Many((_, _, children)) => traverse_services(children, context, depth, branches, formatter, renderer),
        _ => Ok(()),
    }
}

fn append_service<R: Renderer>(
    ty: &Type,
    cardinality: ServiceCardinality,
    renderer: &mut R,
    f: &mut Formatter,
) -> FormatResult {
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

fn append_implementation<R: Renderer>(item: &ServiceDescriptor, renderer: &mut R, f: &mut Formatter) -> FormatResult {
    renderer.implementation(item.implementation_type().name(), f)?;
    renderer.write(' ', f)?;

    match item.lifetime() {
        ServiceLifetime::Scoped => renderer.info("[Scoped]", f),
        ServiceLifetime::Singleton => renderer.info("[Singleton]", f),
        ServiceLifetime::Transient => renderer.info("[Transient]", f),
    }
}

fn indent<R: Renderer>(
    branches: &mut Vec<char>,
    formatter: &mut Formatter,
    renderer: &mut R,
    last: bool,
) -> FormatResult {
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

fn traverse_dependencies<R: Renderer>(
    descriptor: &ServiceDescriptor,
    context: &mut PrintContext,
    depth: usize,
    branches: &mut Vec<char>,
    formatter: &mut Formatter,
    renderer: &mut R,
) -> FormatResult {
    for (index, dependency) in descriptor.dependencies().iter().enumerate() {
        let type_ = dependency.injected_type();
        let cardinality = dependency.cardinality();
        let last = index == descriptor.dependencies().len() - 1;

        indent(branches, formatter, renderer, last)?;

        if let Some(children) = context.lookup(type_) {
            if cardinality == ServiceCardinality::ZeroOrMore {
                print_item(
                    PrintItem::Many((type_, &format!("Count: {}", children.len()), children)),
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
                        PrintItem::Error((child.service_type(), &msg))
                    } else {
                        PrintItem::One(child)
                    };

                    context.enter(child);
                    print_item(item, cardinality, context, depth + 1, branches, formatter, renderer)?;
                    context.exit();
                }
            }
        } else {
            let item = match cardinality {
                ServiceCardinality::ExactlyOne => PrintItem::Error((type_, "‼ Missing")),
                ServiceCardinality::ZeroOrOne => PrintItem::Warning((type_, "▲ Missing")),
                ServiceCardinality::ZeroOrMore => PrintItem::Warning((type_, "▲ Count: 0")),
            };

            print_item(item, cardinality, context, depth + 1, branches, formatter, renderer)?;
        }

        unindent(branches);
    }

    Ok(())
}

fn traverse_services<R: Renderer>(
    descriptors: &Vec<&ServiceDescriptor>,
    context: &mut PrintContext,
    depth: usize,
    branches: &mut Vec<char>,
    formatter: &mut Formatter,
    renderer: &mut R,
) -> FormatResult {
    for (index, descriptor) in descriptors.iter().enumerate() {
        let last = index == descriptors.len() - 1;

        indent(branches, formatter, renderer, last)?;
        print_item(
            PrintItem::One(descriptor),
            ServiceCardinality::ExactlyOne,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{existing, existing_as_self, singleton, singleton_as_self, test::*, transient};
    use std::fs::remove_file;
    use std::path::{Path, PathBuf};

    #[test]
    fn is_empty_should_return_true_when_empty() {
        // arrange
        let collection = ServiceCollection::default();

        // act
        let empty = collection.is_empty();

        // assert
        assert!(empty);
    }

    #[test]
    fn length_should_return_zero_when_empty() {
        // arrange
        let collection = ServiceCollection::default();

        // act
        let length = collection.len();

        // assert
        assert_eq!(length, 0);
    }

    #[test]
    fn is_empty_should_return_false_when_not_empty() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        let not_empty = !collection.is_empty();

        // assert
        assert!(not_empty);
    }

    #[test]
    fn length_should_return_count_when_not_empty() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        let length = collection.len();

        // assert
        assert_eq!(length, 1);
    }

    #[test]
    fn clear_should_remove_all_elements() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        collection.clear();

        // assert
        assert!(collection.is_empty());
    }

    #[test]
    fn try_add_should_do_nothing_when_service_is_registered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection.add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // act
        collection
            .try_add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // assert
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn try_add_to_all_should_add_descriptor_when_implementation_is_unregistered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection.add(existing::<dyn TestService, TestServiceImpl>(Box::new(
            TestServiceImpl::default(),
        )));

        collection.try_add_to_all(
            singleton::<dyn OtherTestService, OtherTestServiceImpl>()
                .from(|sp| Ref::new(OtherTestServiceImpl::new(sp.get_required::<dyn TestService>()))),
        );

        // act
        let count = collection.len();

        // assert
        assert_eq!(count, 2);
    }

    #[test]
    fn try_add_to_all_should_not_add_descriptor_when_implementation_is_registered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection.add(existing::<dyn TestService, TestServiceImpl>(Box::new(
            TestServiceImpl::default(),
        )));

        collection.try_add_to_all(
            transient::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())),
        );

        // act
        let count = collection.len();

        // assert
        assert_eq!(count, 1);
    }

    #[test]
    fn try_add_all_should_only_add_descriptors_for_unregistered_implementations() {
        // arrange
        let descriptors = vec![
            existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default())),
            transient::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())),
        ];
        let mut collection = ServiceCollection::new();

        collection.try_add_all(descriptors.into_iter());

        // act
        let count = collection.len();

        // assert
        assert_eq!(count, 1);
    }

    #[test]
    fn replace_should_replace_first_registered_service() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())))
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // act
        collection
            .replace(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // assert
        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn remove_all_should_remove_registered_services() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())))
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl::default())));

        // act
        collection.remove_all::<dyn TestService>();

        // assert
        assert!(collection.is_empty());
    }

    #[test]
    fn try_replace_should_do_nothing_when_service_is_registered() {
        // arrange
        let mut collection = ServiceCollection::new();

        collection
            .add(singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl { value: 1 })));

        // act
        collection.try_replace(
            singleton::<dyn TestService, TestServiceImpl>().from(|_| Ref::new(TestServiceImpl { value: 2 })),
        );

        // assert
        let value = collection
            .build_provider()
            .unwrap()
            .get_required::<dyn TestService>()
            .value();
        assert_eq!(value, 1);
    }

    #[test]
    fn remove_should_remove_element_at_index() {
        // arrange
        let descriptor = existing::<dyn TestService, TestServiceImpl>(Box::new(TestServiceImpl::default()));
        let mut collection = ServiceCollection::new();

        collection.add(descriptor);

        // act
        let _ = collection.remove(0);

        // assert
        assert!(collection.is_empty());
    }

    #[test]
    fn service_collection_should_drop_existing_as_service() {
        // arrange
        let file = new_temp_file("drop1");

        // act
        {
            let mut services = ServiceCollection::new();
            services.add(existing_as_self(Droppable::new(file.clone())));
        }

        // assert
        let dropped = !file.exists();
        remove_file(&file).ok();
        assert!(dropped);
    }

    #[test]
    fn service_collection_should_not_drop_service_if_never_instantiated() {
        // arrange
        let file = new_temp_file("drop4");
        let mut services = ServiceCollection::new();

        // act
        {
            services
                .add(existing::<Path, PathBuf>(file.clone().into_boxed_path()))
                .add(singleton_as_self().from(|sp| Ref::new(Droppable::new(sp.get_required::<Path>().to_path_buf()))));
        }

        // assert
        let not_dropped = file.exists();
        remove_file(&file).ok();
        assert!(not_dropped);
    }
}
