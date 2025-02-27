// pub: required by Logos derive macro.
#![allow(unreachable_pub)]

use logos::Logos;
use thiserror::Error;

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
    #[regex(r#"[^\s'"!={}()#][^\s'"!={}()]*"#, |lex| lex.slice().to_owned())]
    UnquotedLiteral(String),
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("{")]
    Begin,
    #[token("}")]
    End,
    #[token("$(")]
    SubstitutionBegin,
    #[token(")")]
    SubstitutionEnd,
    #[token("=")]
    Assign,
    #[token("!")]
    Not,
    #[token("with")]
    With,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfInput => write!(f, "Token::EndOfInput"),
            Self::Space => write!(f, "Token::Space"),
            Self::Newline => write!(f, "Token::Newline"),
            Self::SingleQuotedLiteral(s) => write!(f, "Token::SingleQuotedLiteral({s:?})"),
            Self::DoubleQuotedLiteral(s) => write!(f, "Token::DoubleQuotedLiteral({s:?})"),
            Self::UnquotedLiteral(s) => write!(f, "Token::UnquotedLiteral({s:?})"),
            Self::If => write!(f, "Token::If"),
            Self::Else => write!(f, "Token::Else"),
            Self::Begin => write!(f, "Token::Begin"),
            Self::End => write!(f, "Token::End"),
            Self::SubstitutionBegin => write!(f, "Token::SubstitutionBegin"),
            Self::SubstitutionEnd => write!(f, "Token::SubstitutionEnd"),
            Self::Assign => write!(f, "Token::Assign"),
            Self::Not => write!(f, "Token::Not"),
            Self::With => write!(f, "Token::With"),
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
    #[cfg(test)]
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
        Self {
            start,
            end: Some(end),
        }
    }
    fn new_single(start: Location) -> Self {
        Self { start, end: None }
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

impl Iterator for LalrpopLexer<'_> {
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

    impl Iterator for TestLexer<'_> {
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
            special!tokens=are{separators}when unquoted
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
        assert_token!(lex.next(), Token::UnquotedLiteral("special".into()));
        assert_token!(lex.next(), Token::Not);
        assert_token!(lex.next(), Token::UnquotedLiteral("tokens".into()));
        assert_token!(lex.next(), Token::Assign);
        assert_token!(lex.next(), Token::UnquotedLiteral("are".into()));
        assert_token!(lex.next(), Token::Begin);
        assert_token!(lex.next(), Token::UnquotedLiteral("separators".into()));
        assert_token!(lex.next(), Token::End);
        assert_token!(lex.next(), Token::UnquotedLiteral("when".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("unquoted".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_escaped_newline() {
        let mut lex = TestLexer::new(
            r"
            simple command \
                multiple args
        ",
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
            command three # comment 3
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
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("three".into()));
        assert_token!(lex.next(), Token::Newline);
        // Skipped comment 3.
        assert_eoi!(lex);
    }

    #[test]
    fn test_comment_inside_literal() {
        let mut lex = TestLexer::new(
            r#"
            #comment_1
            literal#comment_2
            literal# tail
        "#,
        );
        assert_token!(lex.next(), Token::Newline);
        // Skipped comment_1.
        assert_token!(lex.next(), Token::Newline);
        assert_token!(
            lex.next(),
            Token::UnquotedLiteral("literal#comment_2".into())
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("literal#".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("tail".into()));
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
    fn test_if_not_block() {
        let mut lex = TestLexer::new(
            r#"
            if ! test {
                command hello world
            }
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::If);
        assert_token!(lex.next(), Token::Not);
        assert_token!(lex.next(), Token::UnquotedLiteral("test".into()));
        assert_token!(lex.next(), Token::Begin);
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("hello".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("world".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::End);
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_if_block_where_not_is_a_part_of_the_command() {
        let mut lex = TestLexer::new(
            r#"
            if !test {
                command hello world
            }
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::If);
        assert_token!(lex.next(), Token::Not);
        assert_token!(lex.next(), Token::UnquotedLiteral("test".into()));
        assert_token!(lex.next(), Token::Begin);
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("hello".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("world".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::End);
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_if_block_where_not_is_a_part_of_the_quoted_command() {
        let mut lex = TestLexer::new(
            r#"
            if "!test" {
                command hello world
            }
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::If);
        assert_token!(lex.next(), Token::DoubleQuotedLiteral("!test".into()));
        assert_token!(lex.next(), Token::Begin);
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("command".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("hello".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("world".into()));
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

    #[test]
    fn test_with_statement() {
        let mut lex = TestLexer::new(
            r#"
            with wrapper {
                statement one
                statement two
            }
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::With);
        assert_token!(lex.next(), Token::UnquotedLiteral("wrapper".into()));
        assert_token!(lex.next(), Token::Begin);
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("statement".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("one".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("statement".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("two".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::End);
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_substitution() {
        let mut lex = TestLexer::new(
            r#"
            arg1 $(arg2 arg3) arg4
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::UnquotedLiteral("arg1".into()));
        assert_token!(lex.next(), Token::SubstitutionBegin);
        assert_token!(lex.next(), Token::UnquotedLiteral("arg2".into()));
        assert_token!(lex.next(), Token::UnquotedLiteral("arg3".into()));
        assert_token!(lex.next(), Token::SubstitutionEnd);
        assert_token!(lex.next(), Token::UnquotedLiteral("arg4".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_substitution_in_single_quoted() {
        let mut lex = TestLexer::new(
            r#"
            '$(arg)'
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::SingleQuotedLiteral("$(arg)".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }

    #[test]
    fn test_substitution_in_double_quoted() {
        let mut lex = TestLexer::new(
            r#"
            "$(arg)"
            "#,
        );
        assert_token!(lex.next(), Token::Newline);
        assert_token!(lex.next(), Token::DoubleQuotedLiteral("$(arg)".into()));
        assert_token!(lex.next(), Token::Newline);
        assert_eoi!(lex);
    }
}
