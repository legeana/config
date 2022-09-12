use std::{io::Write, path::PathBuf};

use crate::package::configuration::file_util::make_local_state;
use crate::package::configuration::parser;
use crate::package::configuration::util::multiple_args;
use crate::package::configuration::Configuration;
use crate::registry::Registry;

use anyhow::{anyhow, Context};
use glob::glob as glob_iter;

pub struct CatGlobIntoParser;

const COMMAND: &str = "cat_glob_into";

// TODO: use std::path::MAIN_SEPARATOR_STR when available
// See https://github.com/rust-lang/rust/issues/94071.
const PATH_SEP: &str = "/";

struct CatGlobIntoInstaller {
    dst: PathBuf,
}

struct CatGlobIntoHook {
    globs: Vec<String>,
    dst: PathBuf,
}

impl super::FileInstaller for CatGlobIntoInstaller {
    fn install(&self, registry: &mut dyn Registry) -> anyhow::Result<()> {
        make_local_state(registry, &self.dst)?;
        Ok(())
    }
}

impl super::Hook for CatGlobIntoHook {
    fn execute(&self) -> anyhow::Result<()> {
        let out_file = std::fs::File::create(&self.dst)
            .with_context(|| format!("unable to create {}", self.dst.display()))?;
        let mut out = std::io::BufWriter::new(out_file);
        for glob in self.globs.iter() {
            for entry in glob_iter(glob).with_context(|| format!("failed to glob {}", glob))? {
                let path =
                    entry.with_context(|| format!("failed to iterate over glob {}", glob))?;
                let inp_file = std::fs::File::open(&path)
                    .with_context(|| format!("failed to open {}", path.display()))?;
                let mut inp = std::io::BufReader::new(inp_file);
                std::io::copy(&mut inp, &mut out).with_context(|| {
                    format!(
                        "failed to copy from {} to {}",
                        path.display(),
                        self.dst.display()
                    )
                })?;
            }
        }
        out.flush()
            .with_context(|| format!("failed to flush {}", self.dst.display()))?;
        Ok(())
    }
}

impl parser::Parser for CatGlobIntoParser {
    fn name(&self) -> &'static str {
        COMMAND
    }
    fn help(&self) -> &'static str {
        "cat_glob_into <filename> <glob1> [<glob2> ...]
           create filename in local storage by concatenating globs"
    }
    fn parse(
        &self,
        state: &mut parser::State,
        configuration: &mut Configuration,
        args: &[&str],
    ) -> parser::Result<()> {
        let (fname, globs) = multiple_args(COMMAND, args, 1)?;
        assert!(fname.len() == 1);
        let filename = fname[0];
        let current_prefix = state.prefix.current.to_str().ok_or_else(|| {
            anyhow!(
                "failed to represent current prefix {:?} as a string",
                &state.prefix.current
            )
        })?;
        let glob_prefix = current_prefix.to_owned() + PATH_SEP;
        let concatenated_globs: Vec<String> =
            globs.iter().map(|g| glob_prefix.clone() + g).collect();
        configuration.files.push(Box::new(CatGlobIntoInstaller {
            dst: state.prefix.current.join(filename),
        }));
        configuration.post_hooks.push(Box::new(CatGlobIntoHook {
            globs: concatenated_globs,
            dst: state.prefix.current.join(filename),
        }));
        Ok(())
    }
}
