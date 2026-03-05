use std::fmt::{Formatter, Result, Write};

pub struct Renderer;

impl super::Renderer for Renderer {
    fn write(&mut self, ch: char, f: &mut Formatter<'_>) -> Result {
        f.write_char(ch)
    }

    fn write_str<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn service<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn implementation<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn keyword<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn info<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn warn<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn error<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn accent<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }
}
