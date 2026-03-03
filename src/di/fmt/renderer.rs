use std::fmt::{Formatter, Result};

pub trait Renderer {
    fn write(&mut self, ch: char, f: &mut Formatter<'_>) -> Result;

    fn write_str<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;

    fn service<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;

    fn implementation<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;

    fn keyword<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;

    fn info<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;

    fn warn<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;

    fn error<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;

    fn accent<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result;
}
