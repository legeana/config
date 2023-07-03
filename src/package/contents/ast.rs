// TODO: remove when ast is used
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq)]
pub struct Manifest {
    pub location: PathBuf,
    pub statements: Vec<Statement>,
}

impl Manifest {
    pub fn parse<P: AsRef<Path>>(location: P, input: &str) -> Result<Manifest> {
        let parser = super::ast_parser::ManifestParser::new();
        match parser.parse(location.as_ref(), input) {
            Ok(manifest) => Ok(manifest),
            Err(err) => Err(anyhow!("failed to parse Manifest: {err:?}")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Command(Command),
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse() {
        assert_eq!(
            Manifest::parse("", "").ok(),
            Some(Manifest {
                location: "".into(),
                statements: Vec::new(),
            })
        );
        assert_eq!(
            Manifest::parse("some-path", "").unwrap(),
            Manifest {
                location: "some-path".into(),
                statements: Vec::new(),
            }
        );
        assert_eq!(
            // TODO: use \n instead of ;
            Manifest::parse("", "prefix path;").unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![
                    Statement::Command(Command {
                        name: "prefix".to_owned(),
                        args: vec!["path".to_owned()],
                    }),
                ],
            }
        );
        assert_eq!(
            // TODO: use \n instead of ;
            Manifest::parse("", r#"
                symlink "some/path" and 'another';
                another_command;
            "#).unwrap(),
            Manifest {
                location: "".into(),
                statements: vec![
                    Statement::Command(Command {
                        name: "symlink".to_owned(),
                        args: vec![
                            "some/path".to_owned(),
                            "and".to_owned(),
                            "another".to_owned(),
                        ],
                    }),
                    Statement::Command(Command {
                        name: "another_command".to_owned(),
                        args: Vec::new(),
                    }),
                ],
            }
        );
    }
}
