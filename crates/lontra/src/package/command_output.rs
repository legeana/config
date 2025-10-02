use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use lontra_process::Command;
use lontra_process::cmd;
use regex::Regex;
use regex::RegexSet;
use serde::Deserialize;

use crate::package::satisficer::Satisficer;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub(super) enum CommandOutput {
    CommandRegex {
        command: Vec<PathBuf>,
        regex: String,
    },
    CommandAllRegexes {
        command: Vec<PathBuf>,
        all_regexes: Vec<String>,
    },
    CommandAnyRegex {
        command: Vec<PathBuf>,
        any_regex: Vec<String>,
    },
    BashRegex {
        bash: String,
        regex: String,
    },
    BashAllRegexes {
        bash: String,
        all_regexes: Vec<String>,
    },
    BashAnyRegex {
        bash: String,
        any_regex: Vec<String>,
    },
}

fn command_output<I, S>(command: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::from_argv(command)?.output()
}

fn bash_output(bash: &str) -> Result<String> {
    cmd!(["bash", "-c", bash]).output()
}

#[derive(Debug)]
enum GenericRegex {
    Single(Regex),
    All(Vec<Regex>),
    Any(RegexSet),
}

impl GenericRegex {
    fn new_single(pattern: impl AsRef<str> + Debug) -> Result<Self> {
        let r = Regex::new(pattern.as_ref())
            .with_context(|| format!("failed to compile regex from {pattern:?}"))?;
        Ok(Self::Single(r))
    }
    fn new_all<I, S>(patterns: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str> + Debug,
    {
        let r: Vec<_> = patterns
            .into_iter()
            .map(|p| {
                Regex::new(p.as_ref())
                    .with_context(|| format!("failed to compile regex from {p:?}"))
            })
            .collect::<Result<_>>()?;
        Ok(Self::All(r))
    }
    fn new_any<I, S>(patterns: I) -> Result<Self>
    where
        I: IntoIterator<Item = S> + Debug,
        S: AsRef<str>,
    {
        let r = RegexSet::new(patterns).context("failed to compile regex set")?;
        Ok(Self::Any(r))
    }
    fn is_match(&self, haystack: &str) -> bool {
        match self {
            Self::Single(r) => r.is_match(haystack),
            Self::All(r) => r.iter().all(|r| r.is_match(haystack)),
            Self::Any(r) => r.is_match(haystack),
        }
    }
}

impl Satisficer for CommandOutput {
    fn is_satisfied(&self) -> Result<bool> {
        match self {
            Self::CommandRegex { command, regex } => {
                let r = GenericRegex::new_single(regex)?;
                let output = command_output(command)?;
                Ok(r.is_match(&output))
            }
            Self::CommandAllRegexes {
                command,
                all_regexes,
            } => {
                let r = GenericRegex::new_all(all_regexes)?;
                let output = command_output(command)?;
                Ok(r.is_match(&output))
            }
            Self::CommandAnyRegex { command, any_regex } => {
                let r = GenericRegex::new_any(any_regex)?;
                let output = command_output(command)?;
                Ok(r.is_match(&output))
            }
            Self::BashRegex { bash, regex } => {
                let r = GenericRegex::new_single(regex)?;
                let output = bash_output(bash)?;
                Ok(r.is_match(&output))
            }
            Self::BashAllRegexes { bash, all_regexes } => {
                let r = GenericRegex::new_all(all_regexes)?;
                let output = bash_output(bash)?;
                Ok(r.is_match(&output))
            }
            Self::BashAnyRegex { bash, any_regex } => {
                let r = GenericRegex::new_any(any_regex)?;
                let output = bash_output(bash)?;
                Ok(r.is_match(&output))
            }
        }
    }
}
