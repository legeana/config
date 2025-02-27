use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use super::args::{Argument, Arguments};
use super::lexer;

#[derive(Debug, PartialEq)]
pub(super) struct Manifest {
    pub location: PathBuf,
    pub statements: Vec<Statement>,
}

impl Manifest {
    pub(super) fn parse(location: impl AsRef<Path>, input: impl AsRef<str>) -> Result<Self> {
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
pub(super) enum Statement {
    Command(Invocation),
    IfStatement(IfStatement),
    CommandAssignment(CommandAssignment),
    ValueAssignment(ValueAssignment),
    WithStatement(WithStatement),
}

#[derive(Debug, PartialEq)]
pub(super) struct Invocation {
    pub location: lexer::Location,
    pub name: String,
    pub args: Arguments,
}

impl std::fmt::Display for Invocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.location, shlexfmt::quote(&self.name))?;
        for arg in &self.args.0 {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum Condition {
    Command(Invocation),
    Not(Box<Condition>),
}

#[derive(Debug, PartialEq)]
pub(super) struct IfClause {
    pub location: lexer::Location,
    pub condition: Condition,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub(super) struct IfStatement {
    pub location: lexer::Location,
    pub if_clause: IfClause,
    pub else_if_clauses: Vec<IfClause>,
    pub else_statements: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub(super) struct CommandAssignment {
    pub location: lexer::Location,
    pub var: String,
    pub command: Invocation,
}

#[derive(Debug, PartialEq)]
pub(super) struct ValueAssignment {
    pub location: lexer::Location,
    pub var: String,
    pub value: Argument,
}

#[derive(Debug, PartialEq)]
pub(super) struct WithStatement {
    pub wrapper: Invocation,
    pub statements: Vec<Statement>,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::super::args::args;
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
                statements: vec![Statement::Command(Invocation {
                    location: lexer::Location::new_p_l_c(0, 1, 1),
                    name: "prefix".to_owned(),
                    args: args![~"path"],
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
                statements: vec![Statement::Command(Invocation {
                    location: lexer::Location::new_p_l_c(0, 1, 1),
                    name: "prefix".to_owned(),
                    args: args![~"path"],
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
                    Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(0, 1, 1),
                        name: "prefix".to_owned(),
                        args: args![~"path"],
                    }),
                    Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(12, 2, 1),
                        name: "another".to_owned(),
                        args: args![~"line"],
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
                    Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        name: "command".into(),
                        args: args![~"one"],
                    }),
                    Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(46, 4, 17),
                        name: "command".into(),
                        args: args![~"two"],
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
                    Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        name: "symlink".to_owned(),
                        args: args![@"some/path", ~"and", "another",],
                    }),
                    Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(67, 3, 17),
                        name: "another_command".to_owned(),
                        args: args![],
                    }),
                ],
            }
        );
    }

    #[test]
    fn test_if_statement_no_args() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if cond {
                    statement
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(20, 2, 20),
                            name: "cond".to_owned(),
                            args: args![],
                        }),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(47, 3, 21),
                            name: "statement".to_owned(),
                            args: args![],
                        })],
                    },
                    else_if_clauses: Vec::new(),
                    else_statements: Vec::new(),
                }),],
            }
        );
    }

    #[test]
    fn test_if_statement_with_args() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if cond "with" args {
                    statement
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(20, 2, 20),
                            name: "cond".to_owned(),
                            args: args![@"with", ~"args"],
                        }),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(59, 3, 21),
                            name: "statement".to_owned(),
                            args: args![],
                        })],
                    },
                    else_if_clauses: Vec::new(),
                    else_statements: Vec::new(),
                }),],
            }
        );
    }

    #[test]
    fn test_if_statement_with_else() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if cond {
                    statement
                } else {
                    alternative statement
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(20, 2, 20),
                            name: "cond".to_owned(),
                            args: args![],
                        }),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(47, 3, 21),
                            name: "statement".to_owned(),
                            args: args![],
                        })],
                    },
                    else_if_clauses: Vec::new(),
                    else_statements: vec![Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(102, 5, 21),
                        name: "alternative".to_owned(),
                        args: args![~"statement"],
                    })],
                }),],
            }
        );
    }

    #[test]
    fn test_nested_if_statement() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if cond one {
                    if cond two {
                        statement
                    }
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(20, 2, 20),
                            name: "cond".to_owned(),
                            args: args![~"one"],
                        }),
                        statements: vec![Statement::IfStatement(IfStatement {
                            location: lexer::Location::new_p_l_c(51, 3, 21),
                            if_clause: IfClause {
                                location: lexer::Location::new_p_l_c(51, 3, 21),
                                condition: Condition::Command(Invocation {
                                    location: lexer::Location::new_p_l_c(54, 3, 24),
                                    name: "cond".to_owned(),
                                    args: args![~"two"],
                                }),
                                statements: vec![Statement::Command(Invocation {
                                    location: lexer::Location::new_p_l_c(89, 4, 25),
                                    name: "statement".to_owned(),
                                    args: args![],
                                }),],
                            },
                            else_if_clauses: Vec::new(),
                            else_statements: Vec::new(),
                        })],
                    },
                    else_if_clauses: Vec::new(),
                    else_statements: Vec::new(),
                }),],
            }
        );
    }

    #[test]
    fn test_chained_if_else_statement() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if cond one {
                    statement one
                } else if cond two {
                    statement two
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(20, 2, 20),
                            name: "cond".to_owned(),
                            args: args![~"one"],
                        }),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(51, 3, 21),
                            name: "statement".to_owned(),
                            args: args![~"one"],
                        }),],
                    },
                    else_if_clauses: vec![IfClause {
                        location: lexer::Location::new_p_l_c(88, 4, 24),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(91, 4, 27),
                            name: "cond".to_owned(),
                            args: args![~"two"],
                        }),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(122, 5, 21),
                            name: "statement".to_owned(),
                            args: args![~"two"],
                        }),],
                    },],
                    else_statements: Vec::new(),
                }),],
            }
        );
    }

    #[test]
    fn test_chained_if_else_statement_with_else() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if cond one {
                    statement one
                } else if cond two {
                    statement two
                } else {
                    statement "else"
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(20, 2, 20),
                            name: "cond".to_owned(),
                            args: args![~"one"],
                        }),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(51, 3, 21),
                            name: "statement".to_owned(),
                            args: args![~"one"],
                        }),],
                    },
                    else_if_clauses: vec![IfClause {
                        location: lexer::Location::new_p_l_c(88, 4, 24),
                        condition: Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(91, 4, 27),
                            name: "cond".to_owned(),
                            args: args![~"two"],
                        }),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(122, 5, 21),
                            name: "statement".to_owned(),
                            args: args![~"two"],
                        }),],
                    },],
                    else_statements: vec![Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(181, 7, 21),
                        name: "statement".to_owned(),
                        args: args![@"else"],
                    })],
                }),],
            }
        );
    }

    #[test]
    fn test_if_not_statement() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if ! cond {
                    statement
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Not(Box::new(Condition::Command(Invocation {
                            location: lexer::Location::new_p_l_c(22, 2, 22),
                            name: "cond".to_owned(),
                            args: args![],
                        }))),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(49, 3, 21),
                            name: "statement".to_owned(),
                            args: args![],
                        }),],
                    },
                    else_if_clauses: vec![],
                    else_statements: vec![],
                })],
            }
        );
    }

    #[test]
    fn test_if_not_not_not_statement() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                if ! ! ! cond {
                    statement
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::IfStatement(IfStatement {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    if_clause: IfClause {
                        location: lexer::Location::new_p_l_c(17, 2, 17),
                        condition: Condition::Not(Box::new(Condition::Not(Box::new(
                            Condition::Not(Box::new(Condition::Command(Invocation {
                                location: lexer::Location::new_p_l_c(26, 2, 26),
                                name: "cond".to_owned(),
                                args: args![],
                            })))
                        )))),
                        statements: vec![Statement::Command(Invocation {
                            location: lexer::Location::new_p_l_c(53, 3, 21),
                            name: "statement".to_owned(),
                            args: args![],
                        }),],
                    },
                    else_if_clauses: vec![],
                    else_statements: vec![],
                })],
            }
        );
    }

    #[test]
    fn test_command_assignment() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                my_var = $(command arg)
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::CommandAssignment(CommandAssignment {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    var: "my_var".into(),
                    command: Invocation {
                        location: lexer::Location::new_p_l_c(28, 2, 28),
                        name: "command".into(),
                        args: args![~"arg"],
                    }
                })],
            }
        );
    }

    #[test]
    fn test_value_assignment() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                my_var = value
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::ValueAssignment(ValueAssignment {
                    location: lexer::Location::new_p_l_c(17, 2, 17),
                    var: "my_var".into(),
                    value: Argument::VarsAndHome("value".to_owned()),
                })],
            }
        );
    }

    #[test]
    fn test_with_statement() {
        assert_eq!(
            Manifest::parse(
                "",
                r#"
                with command one {
                    wrapped command
                }
            "#
            )
            .unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![Statement::WithStatement(WithStatement {
                    wrapper: Invocation {
                        location: lexer::Location::new_p_l_c(22, 2, 22),
                        name: "command".into(),
                        args: args![~"one"],
                    },
                    statements: vec![Statement::Command(Invocation {
                        location: lexer::Location::new_p_l_c(56, 3, 21),
                        name: "wrapped".into(),
                        args: args![~"command"],
                    }),],
                })],
            }
        );
    }
}
