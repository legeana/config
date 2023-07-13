use anyhow::Result;
use thiserror::Error;

const ESCAPE: char = '\\';

struct Escaper {
    escape: char,
    special: Vec<char>,
}

impl Escaper {
    fn escape(&self, iter: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
        let special = self.special.clone();
        let escape = self.escape;
        iter.flat_map(move |c| {
            if special.contains(&c) {
                [Some(escape), Some(c)]
            } else {
                [Some(c), None]
            }
        })
        .flatten()
    }
    fn enquote(&self, quote: char, iter: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
        std::iter::once(quote)
            .chain(self.escape(iter))
            .chain(std::iter::once(quote))
    }
}

pub fn enquote_with(quote: char, text: &str) -> String {
    let esc = Escaper {
        escape: ESCAPE,
        special: vec![ESCAPE, quote],
    };
    esc.enquote(quote, text.chars()).collect()
}

pub fn enquote(text: &str) -> String {
    enquote_with('"', text)
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum UnquoteError {
    #[error("unexpected symbol {c:?} at position {pos}: {text:?}")]
    BeforeInit { pos: usize, c: char, text: String },
    #[error("unexpected symbol {c:?} past end quote {q:?} at position {pos}: {text:?}")]
    AfterTerm {
        q: char,
        pos: usize,
        c: char,
        text: String,
    },
    #[error("unexpected string without quotes: {0:?}")]
    NoQuotes(String),
    #[error("no terminating quote {q:?}: {text:?}")]
    NoTerminatingQuote { q: char, text: String },
}

#[derive(Debug, PartialEq)]
enum UnquoteState {
    Init,
    InQuote(char),
    Escape(char),
    Term(char),
    Error(UnquoteError),
    Success(String),
}

fn unquote_impl(text: &str) -> UnquoteState {
    type S = UnquoteState;
    let mut state = S::Init;
    let mut r = String::with_capacity(text.len());
    for (pos, c) in text.chars().enumerate() {
        state = match state {
            S::Init => match c {
                '\'' | '"' => S::InQuote(c),
                _ => S::Error(UnquoteError::BeforeInit {
                    pos,
                    c,
                    text: text.to_owned(),
                }),
            },
            S::InQuote(q) => match c {
                ESCAPE => S::Escape(q),
                _ if c == q => S::Term(q),
                _ => {
                    r.push(c);
                    S::InQuote(q)
                }
            },
            S::Escape(q) => {
                // We allow escaping anything, but only treat quote and ESCAPE specially.
                r.push(c);
                S::InQuote(q)
            }
            S::Term(q) => S::Error(UnquoteError::AfterTerm {
                q,
                pos,
                c,
                text: text.to_owned(),
            }),
            S::Error(e) => S::Error(e),
            S::Success(_) => panic!("unexpected state Success at position {pos}: {text:?}"),
        };
    }
    match state {
        S::Init => S::Error(UnquoteError::NoQuotes(text.to_owned())),
        S::InQuote(q) | S::Escape(q) => S::Error(UnquoteError::NoTerminatingQuote {
            q,
            text: text.to_owned(),
        }),
        S::Term(_) => S::Success(r),
        S::Error(e) => S::Error(e),
        S::Success(_) => panic!("there is no state transition to Success: {text:?}"),
    }
}

#[allow(dead_code)]
pub fn unquote<S: AsRef<str>>(text: S) -> Result<String, UnquoteError> {
    type S = UnquoteState;
    let text = text.as_ref();
    match unquote_impl(text) {
        S::Error(e) => Err(e),
        S::Success(result) => Ok(result),
        state => panic!("unexpected state {state:?}: {text:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enquote_with() {
        assert_eq!(enquote_with('\'', "hello world"), "'hello world'");
        assert_eq!(enquote_with('\'', "'hello'"), r#"'\'hello\''"#);
        assert_eq!(enquote_with('\'', r#"hello\ world"#), r#"'hello\\ world'"#);
    }

    #[test]
    fn test_enquote() {
        assert_eq!(enquote("hello"), r#""hello""#);
        assert_eq!(
            enquote(r#"test "hello" world"#),
            r#""test \"hello\" world""#
        );
        assert_eq!(enquote(r#"escaped\ symbol"#), r#""escaped\\ symbol""#);
    }

    #[test]
    fn test_unquote() {
        assert!(unquote("").is_err());
        assert_eq!(
            unquote(r#""hello world""#).ok(),
            Some("hello world".to_owned())
        );
        assert_eq!(
            unquote(r#"'hello world'"#).ok(),
            Some("hello world".to_owned())
        );
        assert_eq!(
            unquote(r#""hello\"world""#).ok(),
            Some(r#"hello"world"#.to_owned())
        );
        assert_eq!(
            unquote(r#""hello\'world""#).ok(),
            Some(r#"hello'world"#.to_owned())
        );
    }

    #[test]
    fn test_unquote_impl() {
        type S = UnquoteState;
        type E = UnquoteError;

        assert_eq!(
            unquote_impl(r#""hello"world"#),
            S::Error(E::AfterTerm {
                q: '"',
                pos: 7,
                c: 'w',
                text: r#""hello"world"#.to_owned()
            })
        );
        assert_eq!(unquote_impl(""), S::Error(E::NoQuotes("".to_owned())));
        assert_eq!(
            unquote_impl(r#"hello"#),
            S::Error(E::BeforeInit {
                pos: 0,
                c: 'h',
                text: "hello".to_owned()
            })
        );
        assert_eq!(
            unquote_impl(r#""hello"#),
            S::Error(E::NoTerminatingQuote {
                q: '"',
                text: r#""hello"#.to_owned()
            })
        );
    }
}
