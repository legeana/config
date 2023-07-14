use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use super::lexer;

#[derive(Debug, PartialEq)]
pub struct Manifest {
    pub location: PathBuf,
    pub statements: Vec<Statement>,
}

impl Manifest {
    pub fn parse(location: impl AsRef<Path>, input: impl AsRef<str>) -> Result<Manifest> {
        let location = location.as_ref();
        let lex = lexer::LalrpopLexer::new(input.as_ref());
        let parser = super::ast_parser::ManifestParser::new();
        match parser.parse(location, lex) {
            Ok(manifest) => Ok(manifest),
            Err(err) => Err(anyhow!("failed to parse {location:?}: {err}")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Command(Command),
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub location: lexer::Location,
    pub name: String,
    pub args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_manifest() {
        assert_eq!(
            Manifest::parse("", "").unwrap(),
            Manifest {
                location: "".into(),
                statements: Vec::new(),
            }
        );
    }

    #[test]
    fn test_manifest_location() {
        assert_eq!(
            Manifest::parse("some-path", "").unwrap(),
            Manifest {
                location: "some-path".into(),
                statements: Vec::new(),
            }
        );
    }

    #[test]
    fn test_manifest_empty_line() {
        assert_eq!(
            Manifest::parse("", "\n").unwrap(),
            Manifest {
                location: "".into(),
                statements: Vec::new(),
            }
        );
    }

    #[test]
    fn test_manifest_without_trailing_newline() {
        assert_eq!(
            Manifest::parse("", "prefix path").unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::Command(Command {
                    location: lexer::Location::new_p_l_c(0, 1, 1),
                    name: "prefix".to_owned(),
                    args: vec!["path".to_owned()],
                }),],
            }
        );
    }

    #[test]
    fn test_manifest_single_line() {
        assert_eq!(
            Manifest::parse("", "prefix path\n").unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::Command(Command {
                    location: lexer::Location::new_p_l_c(0, 1, 1),
                    name: "prefix".to_owned(),
                    args: vec!["path".to_owned()],
                }),],
            }
        );
    }

    #[test]
    fn test_manifest_multiple_lines() {
        assert_eq!(
            Manifest::parse("", "prefix path\nanother line\n").unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![
                    Statement::Command(Command {
                        location: lexer::Location::new_p_l_c(0, 1, 1),
                        name: "prefix".to_owned(),
                        args: vec!["path".to_owned()],
                    }),
                    Statement::Command(Command {
                        location: lexer::Location::new_p_l_c(12, 2, 1),
                        name: "another".to_owned(),
                        args: vec!["line".to_owned()],
                    }),
                ],
            }
        );
    }

    #[test]
    fn test_manifest_empty_lines_between() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                command one

                command two
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![
                    Statement::Command(Command {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        name: "command".into(),
                        args: vec!["one".into()]
                    }),
                    Statement::Command(Command {
                        location: lexer::Location::new_p_l_c(46, 4, 17),
                        name: "command".into(),
                        args: vec!["two".into()]
                    }),
                ],
            }
        );
    }

    #[test]
    fn test_multiple_quoted_statements() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                symlink "some/path" and 'another'
                another_command
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![
                    Statement::Command(Command {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        name: "symlink".to_owned(),
                        args: vec![
                            "some/path".to_owned(),
                            "and".to_owned(),
                            "another".to_owned(),
                        ],
                    }),
                    Statement::Command(Command {
                        location: lexer::Location::new_p_l_c(67, 3, 17),
                        name: "another_command".to_owned(),
                        args: Vec::new(),
                    }),
                ],
            }
        );
    }
}