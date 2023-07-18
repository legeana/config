use logos::Logos;
use thiserror::Error;

use crate::quote;

#[derive(Error, Clone, Debug, Default, PartialEq)]
pub enum LexerError {
    #[error(transparent)]
    UnquoteError(#[from] quote::UnquoteError),
    #[default]
    #[error("invalid token")]
    InvalidToken,
}

impl LexerError {
    pub fn with_location(self, lex: &logos::Lexer<Token>) -> LocationError {
        let (start, end) = lex.extras.span_location(&lex.span());
        match self {
            Self::InvalidToken => LocationError {
                source: self,
                location: LocationRange::new_single(start),
            },
            _ => LocationError {
                source: self,
                location: LocationRange::new_pair(start, end),
            },
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
#[error("{source:#} at {location}")]
pub struct LocationError {
    #[source]
    source: LexerError,
    location: LocationRange,
}

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(error = LexerError)]
#[logos(extras = LineTracker)]
#[logos(skip r"#.*")] // comments
pub enum Token {
    #[token("\\\n", |lex| lex.extras.record_line(lex.span().end))]
    #[regex(r#"[ \t]+"#)]
    Space,
    #[token("\n", |lex| lex.extras.record_line(lex.span().end))]
    Newline,
    #[regex(r#"'([^'\n\\]|\\[^\n])*'"#, |lex| quote::unquote(lex.slice()))]
    #[regex(r#""([^"\n\\]|\\[^\n])*""#, |lex| quote::unquote(lex.slice()))]
    #[regex(r#"[^\s'"]+"#, |lex| lex.slice().to_owned())]
    Literal(String),
    #[token("if")]
    If,
    #[token("{")]
    Begin,
    #[token("}")]
    End,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Space => write!(f, "Token::Space"),
            Token::Newline => write!(f, "Token::Newline"),
            Token::Literal(s) => write!(f, "Token::Literal({s:?})"),
            Token::If => write!(f, "Token::If"),
            Token::Begin => write!(f, "Token::Begin"),
            Token::End => write!(f, "Token::End"),
        }
    }
}

pub struct LineTracker {
    line_starts: Vec<usize>, // byte index
}

impl LineTracker {
    fn line_index(&self, pos: usize) -> usize {
        assert!(!self.line_starts.is_empty());
        // Binary search?
        for (line_index, start) in self.line_starts.iter().enumerate().rev() {
            if pos >= *start {
                return line_index;
            }
        }
        assert!(pos >= *self.line_starts.last().expect("must not be empty"));
        self.line_starts.len() - 1
    }
    fn char_location(&self, pos: usize) -> Location {
        let line_index = self.line_index(pos);
        assert!(line_index < self.line_starts.len());
        let line_start = self.line_starts[line_index];
        assert!(line_start <= pos);
        let column = pos - line_start + 1;
        Location {
            index: pos,
            line_number: line_index + 1,
            column,
        }
    }
    fn span_location(&self, span: &logos::Span) -> (Location, Location) {
        (self.char_location(span.start), self.char_location(span.end))
    }
    fn record_line(&mut self, line_start: usize) {
        self.line_starts.push(line_start);
    }
}

impl Default for LineTracker {
    fn default() -> Self {
        Self {
            // First line always starts at 0.
            line_starts: vec![0],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Location {
    pub index: usize,
    pub line_number: usize,
    pub column: usize,
}

impl Location {
    // Used only in tests.
    #[allow(dead_code)]
    pub fn new_p_l_c(index: usize, line_number: usize, column: usize) -> Self {
        Self {
            index,
            line_number,
            column,
        }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {} column {}", self.line_number, self.column)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct LocationRange {
    start: Location,
    end: Option<Location>,
}

impl LocationRange {
    fn new_pair(start: Location, end: Location) -> Self {
        LocationRange {
            start,
            end: Some(end),
        }
    }
    fn new_single(start: Location) -> Self {
        LocationRange { start, end: None }
    }
}

impl std::fmt::Display for LocationRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.start)?;
        match self.end {
            Some(end) => write!(f, " .. {end}"),
            None => Ok(()),
        }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct LalrpopLexer<'input> {
    lexer: logos::Lexer<'input, Token>,
}

impl<'input> LalrpopLexer<'input> {
    pub fn new(source: &'input str) -> Self {
        Self {
            lexer: Token::lexer(source),
        }
    }
}

impl<'input> Iterator for LalrpopLexer<'input> {
    type Item = Spanned<Token, Location, LocationError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(Ok(tok)) => {
                let span = self.lexer.span();
                let (start, end) = self.lexer.extras.span_location(&span);
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
    fn test_no_space_between_literals() {
        let mut lex = Token::lexer(
            r#"
            "hello""world"
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("hello".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("world".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_no_newline_in_literal() {
        let mut lex = Token::lexer(
            r#"
            "hello
            world"#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        let err = lex.next().expect("error").expect_err("");
        assert_eq!(err, LexerError::InvalidToken);
    }

    #[test]
    fn test_comments() {
        let mut lex = Token::lexer(
            r#"
            # comment 1
            command one
            # comment 2
            command two
        "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        // Skipped comment 1.
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("command".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("one".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        // Skipped comment 2.
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("command".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("two".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_multiline_string() {
        let mut lex = Token::lexer("\"\n\"");
        assert_eq!(lex.next(), Some(Err(LexerError::InvalidToken)));
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
        let err = lex.next().expect("error").expect_err("InvalidToken");
        assert_eq!(err, LexerError::InvalidToken);
        let loc_err = err.with_location(&lex);
        assert_eq!(
            loc_err.location,
            LocationRange::new_single(Location::new_p_l_c(13, 2, 13))
        );
        assert_eq!(loc_err.to_string(), "invalid token at line 2 column 13");
    }

    #[test]
    fn test_if_block() {
        let mut lex = Token::lexer(
            r#"
            if test {
                command hello world
            }
            "#,
        );
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::If)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("test".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Begin)));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("command".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("hello".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::Literal("world".into()))));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), Some(Ok(Token::End)));
        assert_eq!(lex.next(), Some(Ok(Token::Newline)));
        assert_eq!(lex.next(), Some(Ok(Token::Space)));
        assert_eq!(lex.next(), None);
    }
}
