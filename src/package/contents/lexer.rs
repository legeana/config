use logos::Logos;
use thiserror::Error;

use crate::quote;

#[derive(Error, Clone, Debug, Default, PartialEq)]
pub enum LexerError {
    #[error(transparent)]
    UnquoteError(#[from] quote::UnquoteError),
    #[default]
    #[error("invalid token")]
    InvalidLiteral,
}

impl LexerError {
    pub fn with_location(self, lex: &logos::Lexer<Token>) -> LocationError {
        LocationError {
            source: self,
            location: lex.extras.location(&lex.span()),
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
#[error("{source:#} at {location}")]
pub struct LocationError {
    #[source]
    source: LexerError,
    location: Location,
}

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(error = LexerError)]
#[logos(extras = LineTracker)]
pub enum Token {
    #[token("\\\n", |lex| lex.extras.update_line(&lex.span()))]
    #[regex(r#"[ \t]+"#)]
    Space,
    #[token("\n", |lex| lex.extras.update_line(&lex.span()))]
    Newline,
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| quote::unquote(lex.slice()))]
    #[regex(r#""([^"\\]|\\.)*""#, |lex| quote::unquote(lex.slice()))]
    #[regex(r#"\S+"#, |lex| lex.slice().to_owned())]
    Literal(String),
}

#[derive(Default)]
pub struct LineTracker {
    line_start: usize,  // starts at 0
    line_number: usize, // starts at 0
}

impl LineTracker {
    pub fn location(&self, span: &logos::Span) -> Location {
        Location {
            index: span.start,
            line_number: self.line_number,
            column: span.start - self.line_start,
        }
    }
    fn update_line(&mut self, span: &logos::Span) {
        self.line_number += 1;
        self.line_start = span.end;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    index: usize,
    line_number: usize,
    column: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {} column {}",
            self.line_number + 1,
            self.column + 1
        )
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct LalrpopLexer<'input> {
    lexer: logos::Lexer<'input, Token>,
}

impl<'input> LalrpopLexer<'input> {
    #[allow(dead_code)]
    pub fn new(source: &'input str) -> Self {
        Self {
            lexer: Token::lexer(source),
        }
    }
}

impl<'input> Iterator for LalrpopLexer<'input> {
    type Item = Spanned<Token, usize, LocationError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(Ok(tok)) => {
                let logos::Span { start, end } = self.lexer.span();
                Some(Ok((start, tok, end)))
            }
            Some(Err(err)) => {
                let err = err.with_location(&self.lexer);
                Some(Err(err))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unquoted() {
        let mut lex = Token::lexer(
            r#"
            simple command
            another command
            do some/path
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("simple".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("command".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("another".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("command".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("do".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("some/path".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_escaped_newline() {
        let mut lex = Token::lexer(
            r#"
            simple command \
                multiple args
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("simple".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("command".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.slice(), "\\\n");
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("multiple".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("args".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_single_quoted_literals() {
        let mut lex = Token::lexer(
            r#"
            'single-quoted'
            'single quoted'
            'single \'quoted\''
            '"single quoted"'
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        let next = lex.next();
        println!("{:?} #{}#", lex.next(), lex.slice());
        assert_eq!(next, Some(Ok(Token::Literal("single-quoted".into()))));
    }

    #[test]
    fn test_double_quoted_literals() {
        let mut lex = Token::lexer(
            r#"
            "double-quoted"
            "double quoted"
            "double \"quoted\""
            "'double quoted'"
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("double-quoted".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal(r#"double quoted"#.into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal(r#"double "quoted""#.into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal("'double quoted'".into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_quoted_literals() {
        let mut lex = Token::lexer(
            r#"
            unquoted 'single quoted' "double quoted"
            unquoted\escaped 'single \'quoted\'' "double \"quoted\""
            "mixed 'quoted'" 'mixed "quoted"'
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("unquoted".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("single quoted".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("double quoted".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal("unquoted\\escaped".into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal("single 'quoted'".into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal(r#"double "quoted""#.into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal(r#"mixed 'quoted'"#.into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::Literal(r#"mixed "quoted""#.into())))
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_error() {
        let mut lex = Token::lexer(
            r#"
            "misquoted
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        let err = lex.next().expect("error").expect_err("InvalidLiteral");
        assert_eq!(err, LexerError::InvalidLiteral);
        let loc_err = err.with_location(&lex);
        assert_eq!(
            loc_err.location,
            Location {
                index: 13,
                line_number: 1,
                column: 12
            }
        );
        assert_eq!(loc_err.to_string(), "invalid token at line 2 column 13");
    }
}
