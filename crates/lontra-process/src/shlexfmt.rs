//! Shell quoting helpers used for logging and human consumption.
//! The helpers in this library don't return errors and may not be shell-safe
//! or accurate. Use shlex directly instead.

use std::borrow::Cow;

use shlex::Quoter;

pub fn quote(in_str: &str) -> Cow<'_, str> {
    Quoter::new().allow_nul(true).quote(in_str).unwrap()
}

pub fn join<'a, I: IntoIterator<Item = &'a str>>(words: I) -> String {
    Quoter::new().allow_nul(true).join(words).unwrap()
}
