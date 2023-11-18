# Extensibility

Using dependency injection in your own application is certainly useful; however, the strength of the `more-di` crate really starts to shine when you enable DI composition across different crates. It effectively enables a DI ecosystem that crate library authors can elect to make required or opt into as a conditional feature.

## Configuration

There are few requirements to make it possible to interleave DI into a library. This will typically be configured in `Cargo.toml`.

```toml
[package]
name = "logger"
version = "1.0.0"
description = "An example logger"

[lib]
name = "logger"

[features]
di = ["more-di"]
async = ["more-di/async"] # our 'async' feature actives the 'more-di/async' feature

[dependencies.more-di]
version = "3.0"
default-features = false
features = ["inject"]
optional = true           # only bring di when requested
```

The next part is to create a trait that can apply the extensions. It is not a hard requirement, but this typically takes the form of:

- `pub trait <Feature>ServiceExtensions`
- Defined in the `crate::ext` module

The library module would then configure the extensions as optional.

#### lib.rs

```rust
#[cfg(feature = "di")]
mod di_ext;

#[cfg(feature = "di")]
pub mod ext {
    use super::*;    
    pub use di_ext::*;
}
```

The extensions will then look something like the following and apply to `ServiceCollection`.

#### di_ext.rs

```rust
use di::*;

// define extensions that can be applied to ServiceCollection
// note: remember to flow a mutable reference to make it easy
// to compose with other extensions
pub trait CustomServiceExtensions {
    fn add_custom_services(&mut self) -> &mut Self;
}

impl CustomServiceExtensions for ServiceCollection {
    fn add_custom_services(&mut self) -> &mut Self {
        // add custom services
        self.try_add(transient::<dyn Service, DefaultImpl>())
    }
}
```

## Example

Let's consider several composable library crates for logging.

### Logger Crate

This crate would include the core abstractions common to all extenders.

```rust
// map to DI type when enabled
#[cfg(feature = "di")]
pub type Ref<T> = di::Ref<T>;

// default to Rc<T>
#[cfg(not(feature = "di"))]
pub type Ref<T> = std::rc::Rc<T>;

pub trait Logger {
    fn log(&self, text: &str);
}

pub trait LoggerSource {
    fn log(&self, text: &str);
}

// #[injectable(Logger)] only when the "di" feature is enabled
#[cfg_attr(feature = "di", di::injectable(Logger))]
pub struct DefaultLogger {
    loggers: Vec<Ref<dyn LoggerSource>>,
}

impl DefaultLogger {
    pub fn new(loggers: impl Iterator<Item = Ref<dyn LoggerSource>>) -> Self {
        Self {
            loggers: loggers.collect(),
        }
    }
}

impl Logger for DefaultLogger {
    fn log(&self, text: &str) {
        for logger in self.loggers {
            logger.log(text)
        }
    }
}

#[cfg(feature = "di")]
pub mod ext {
    use di::*;

    pub trait LoggerServiceExtensions {
        fn add_logging(&mut self) -> &mut Self;
    }

    impl LoggerServiceExtensions for ServiceCollection {
        fn add_logging(&mut self) -> &mut Self {
            self.try_add(DefaultLogger::singleton())
        }
    }
}
```

## Console Logger Crate

This crate would provide a logger which writes to the console.

```rust
#[cfg_attr(feature = "di", di::injectable(LoggerSource))]
pub struct ConsoleLogger;

impl LoggerSource for ConsoleLogger {
    fn log(&self, text: &str) {
        println!("{}", text)
    }
}

#[cfg(feature = "di")]
pub mod ext {
    use di::*;

    pub trait ConsoleLoggerServiceExtensions {
        fn add_console_logging(&mut self) -> &mut Self;
    }

    impl ConsoleLoggerServiceExtensions for ServiceCollection {
        fn add_console_logging(&mut self) -> &mut Self {
            self.try_add(ConsoleLogger::transient())
        }
    }
}
```

## File Logger Crate

This crate would provide a logger which writes to a file.

```rust
use std::fs::File;

pub struct FileLogger {
    file: File,
}

impl FileLogger {
    pub fn new<S: AsRef<str>>(filename: S) -> Self {
        Self {
            file: File::create(Path::new(s.as_ref())).unwrap(),
        }
    }
}

impl LoggerSource for FileLogger {
    fn log(&self, text: &str) {
        self.file.write_all(text.as_bytes()).ok()
    }
}

#[cfg(feature = "di")]
pub mod ext {
    use di::*;

    pub trait FileLoggerServiceExtensions {
        fn add_file_logging<S: AsRef<str>>(&mut self, filename: S) -> &mut Self;
    }

    impl FileLoggerServiceExtensions for ServiceCollection {
        fn add_file_logging<S: AsRef<str>>(&mut self, filename: S) -> &mut Self {
            let path = filename.as_ref().clone();
            self.try_add(transient::<dyn LoggerSource, FileLogger>()
                         .from(move |_| Ref::new(FileLogger::new(path))))
        }
    }
}
```

## Putting It All Together

We can now put it all together in an application. Our configuration will look something like:

```toml
[package]
name = "myapp"
version = "1.0.0"
description = "An example application"

[[bin]]
name = "myapp"
path = "main.rs"

[dependencies]
more-di = "3.0"
logger = { "1.0.0", features = ["di"] }
console-logger = { "1.0.0", features = ["di"] }
file-logger = { "1.0.0", features = ["di"] }
```

#### main.rs

```rust
use di::*;
use logger::{*, ext::*};
use console_logger::{*, ext::*};
use file_logger::{*, ext::*};

fn main() {
    let provider = ServiceCollection::new()
        .add_logging()
        .add_console_logging()
        .add_file_logging("example.log")
        .build_provider()
        .unwrap();

    let logger = provider.get_required::<dyn Logger>();

    logger.log("Hello world!");
}
```

## Dependency Injected Enabled Crates

The following are crates which provide DI extensions:

- [more-options](https://crates.io/crates/more-options)