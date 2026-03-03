use colored::Colorize;
use std::fmt::{Display, Formatter, Result, Write};

pub struct Renderer;

impl super::Renderer for Renderer {
    fn write(&mut self, ch: char, f: &mut Formatter<'_>) -> Result {
        f.write_char(ch)
    }

    fn write_str<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        f.write_str(text.as_ref())
    }

    fn keyword<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        text.as_ref().truecolor(75, 154, 214).fmt(f)
    }

    fn service<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        text.as_ref().truecolor(158, 211, 163).fmt(f)
    }

    fn implementation<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        text.as_ref().truecolor(78, 201, 176).fmt(f)
    }

    fn info<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        text.as_ref().truecolor(118, 118, 118).fmt(f)
    }

    fn warn<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        text.as_ref().truecolor(220, 220, 170).fmt(f)
    }

    fn error<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        text.as_ref().truecolor(231, 72, 86).fmt(f)
    }

    fn accent<T: AsRef<str>>(&mut self, text: T, f: &mut Formatter<'_>) -> Result {
        text.as_ref().truecolor(218, 112, 179).fmt(f)
    }
}
