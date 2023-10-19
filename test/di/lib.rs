#![cfg(test)]

// since this is a test crate, the test configuration needs to be
// specified in order to expand macros
//
// RUSTFLAGS='--cfg test' cargo expand

mod containers;
mod mutable;
mod scenarios;
mod structs;
mod traits;
