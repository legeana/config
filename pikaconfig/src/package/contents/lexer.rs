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
    #[error("literals without separator")]
    LiteralsWithoutSeparator,
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
    EndOfInput,
    #[token("\\\n", |lex| lex.extras.record_line(lex.span().end))]
    #[regex(r#"[ \t]+"#)]
    Space,
    #[token("\n", |lex| lex.extras.record_line(lex.span().end))]
    #[token("\r\n", |lex| lex.extras.record_line(lex.span().end))]
    Newline,
    #[regex(r#"'([^'\n\\]|\\[^\n])*'"#, |lex| quote::unquote(lex.slice()))]
    SingleQuotedLiteral(String),
    #[regex(r#""([^"\n\\]|\\[^\n])*""#, |lex| quote::unquote(lex.slice()))]
    DoubleQuotedLiteral(String),
    #[regex(r#"[^\s'"]+"#, |lex| lex.slice().to_owned())]
    UnquotedLiteral(String),
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("{")]
    Begin,
    #[token("}")]
    End,
    #[token("=")]
    Assign,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::EndOfInput => write!(f, "Token::EndOfInput"),
            Token::Space => write!(f, "Token::Space"),
            Token::Newline => write!(f, "Token::Newline"),
            Token::SingleQuotedLiteral(s) => write!(f, "Token::SingleQuotedLiteral({s:?})"),
            Token::DoubleQuotedLiteral(s) => write!(f, "Token::DoubleQuotedLiteral({s:?})"),
            Token::UnquotedLiteral(s) => write!(f, "Token::UnquotedLiteral({s:?})"),
            Token::If => write!(f, "Token::If"),
            Token::Else => write!(f, "Token::Else"),
            Token::Begin => write!(f, "Token::Begin"),
            Token::End => write!(f, "Token::End"),
            Token::Assign => write!(f, "Token::Assign"),
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
    prev_literal: bool,
    stop_iteration: bool,
    lexer: logos::Lexer<'input, Token>,
}

impl<'input> LalrpopLexer<'input> {
    pub fn new(source: &'input str) -> Self {
        Self {
            prev_literal: false,
            stop_iteration: false,
            lexer: Token::lexer(source),
        }
    }
    /// Handles literal separation and EndOfInput.
    fn next_logos_tok(&mut self) -> Option<Result<Token, LexerError>> {
        if self.stop_iteration {
            return None;
        }
        loop {
            let next = self.lexer.next();
            match next {
                Some(Ok(
                    Token::SingleQuotedLiteral(_)
                    | Token::DoubleQuotedLiteral(_)
                    | Token::UnquotedLiteral(_),
                )) => {
                    if self.prev_literal {
                        self.terminate();
                        return Some(Err(LexerError::LiteralsWithoutSeparator));
                    }
                    self.prev_literal = true;
                    return next;
                }
                Some(Ok(Token::Space)) => {
                    self.prev_literal = false;
                    continue; // Skip spaces.
                }
                Some(Ok(_)) => {
                    self.prev_literal = false;
                    return next;
                }
                Some(Err(_)) => {
                    self.terminate();
                    return next;
                }
                None => {
                    self.terminate();
                    return Some(Ok(Token::EndOfInput));
                }
            }
        }
    }
    fn terminate(&mut self) {
        self.stop_iteration = true;
    }
}

impl<'input> Iterator for LalrpopLexer<'input> {
    type Item = Spanned<Token, Location, LocationError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_logos_tok() {
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

    struct TestLexer<'input>(LalrpopLexer<'input>);

    impl<'input> TestLexer<'input> {
        fn new(source: &'input str) -> Self {
            Self(LalrpopLexer::new(source))
        }
    }

    impl<'input> Iterator for TestLexer<'input> {
        type Item = Result<Token, LocationError>;

        fn next(&mut self) -> Option<Result<Token, LocationError>> {
            match self.0.next() {
                Some(Ok((_, tok, _))) => Some(Ok(tok)),
                Some(Err(err)) => Some(Err(err)),
                None => None,
            }
        }
    }

    macro_rules! assert_token {
        ($item:expr, $tok:expr) => {{
            let result = $item;
            assert_eq!(result, Some(Ok($tok)));
            result.unwrap().unwrap()
        }};
    }

    macro_rules! assert_err {
        ($item:expr, $err:expr) => {{
            let result = $item;
            assert_eq!(result, Some(Err($err)));
            result.unwrap().unwrap_err()
        }};
    }

    macro_rules! assert_eoi {
        ($lex:ident) => {
            assert_eq!($lex.next(), Some(Ok(Token::EndOfInput)));
            assert_eq!($lex.next(), None);
        };
    }

    #[test]
    fn test_crlf_newlines() {
        let mut lex = TestLexer::new("hello\r\nworld\r\n");
        assert_token!(lex.next(), Token::UnquotedLiteral("hello".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("world".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_unquoted() {
        let mut lex = TestLexer::new(
            r#"
            simple command
            another command
            do some/path
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("simple".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("another".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("do".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("some/path".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_escaped_newline() {
        let mut lex = TestLexer::new(
            r#"
            simple command \
                multiple args
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("simple".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("multiple".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("args".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_single_quoted_literals() {
        let mut lex = TestLexer::new(
            r#"
            'single-quoted'
            'single quoted'
            'single \'quoted\''
            '"single quoted"'
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::SingleQuotedLiteral("single-quoted".into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::SingleQuotedLiteral("single quoted".into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::SingleQuotedLiteral("single 'quoted'".into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::SingleQuotedLiteral(r#""single quoted""#.into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_double_quoted_literals() {
        let mut lex = TestLexer::new(
            r#"
            "double-quoted"
            "double quoted"
            "double \"quoted\""
            "'double quoted'"
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::DoubleQuotedLiteral("double-quoted".into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::DoubleQuotedLiteral(r#"double quoted"#.into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::DoubleQuotedLiteral(r#"double "quoted""#.into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::DoubleQuotedLiteral("'double quoted'".into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_quoted_literals() {
        let mut lex = TestLexer::new(
            r#"
            unquoted 'single quoted' "double quoted"
            unquoted\escaped 'single \'quoted\'' "double \"quoted\""
            "mixed 'quoted'" 'mixed "quoted"'
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("unquoted".into()));
        assert_token!(
            lex.next(),
            Token::SingleQuotedLiteral("single quoted".into())
        );
        assert_token!(
            lex.next(),
            Token::DoubleQuotedLiteral("double quoted".into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::UnquotedLiteral("unquoted\\escaped".into())
        );
        assert_token!(
            lex.next(),
            Token::SingleQuotedLiteral("single 'quoted'".into())
        );
        assert_token!(
            lex.next(),
            Token::DoubleQuotedLiteral(r#"double "quoted""#.into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::DoubleQuotedLiteral(r#"mixed 'quoted'"#.into())
        );
        assert_token!(
            lex.next(),
            Token::SingleQuotedLiteral(r#"mixed "quoted""#.into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_no_space_between_literals() {
        let mut lex = TestLexer::new(
            r#"
            "hello""world"
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::DoubleQuotedLiteral("hello".into()));
        assert_err!(
            lex.next(),
            LocationError {
                source: LexerError::LiteralsWithoutSeparator,
                location: LocationRange::new_pair(
                    Location::new_p_l_c(20, 2, 20),
                    Location::new_p_l_c(27, 2, 27),
                ),
            }
        );
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_no_newline_in_literal() {
        let mut lex = TestLexer::new(
            r#"
            "hello
            world"#,
        );
        assert_token!(lex.next(), Token::Newline);
        let err = lex.next().expect("error").expect_err("");
        assert_eq!(err.source, LexerError::InvalidToken);
    }

    #[test]
    fn test_comments() {
        let mut lex = TestLexer::new(
            r#"
            # comment 1
            command one
            # comment 2
            command two
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        // Skipped comment 1.
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("one".into()));
        assert_token!(lex.next(), Token::Newline);
        // Skipped comment 2.
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("two".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_multiline_string() {
        let mut lex = Token::lexer("\"\n\"");
        assert_eq!(lex.next(), Some(Err(LexerError::InvalidToken)));
    }

    #[test]
    fn test_error() {
        let mut lex = TestLexer::new(
            r#"
            "misquoted
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        let err = assert_err!(
            lex.next(),
            LocationError {
                source: LexerError::InvalidToken,
                location: LocationRange::new_single(Location::new_p_l_c(13, 2, 13)),
            }
        );
        assert_eq!(err.to_string(), "invalid token at line 2 column 13");
    }

    #[test]
    fn test_if_block() {
        let mut lex = TestLexer::new(
            r#"
            if test {
                command hello world
            } else {
                alternative command
            }
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::If);
        assert_token!(lex.next(), Token::UnquotedLiteral("test".into()));
        assert_token!(lex.next(), Token::Begin);
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("hello".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("world".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::End);
        assert_token!(lex.next(), Token::Else);
        assert_token!(lex.next(), Token::Begin);
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("alternative".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::End);
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_assignment() {
        let mut lex = TestLexer::new(
            r#"
            x = command arg
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("x".into()));
        assert_token!(lex.next(), Token::Assign);
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("arg".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }
}
