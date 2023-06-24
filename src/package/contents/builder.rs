use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tera::Tera;
use thiserror::Error;

use crate::module::Module;

#[derive(Error, Debug)]
pub enum Error {
    #[error("builder {builder}: unsupported command {command}")]
    UnsupportedCommand { builder: String, command: String },
}

pub struct State {
    pub enabled: bool,
    pub prefix: PathBuf,
}

impl State {
    pub fn new() -> Self {
        Self {
            enabled: true,
            prefix: dirs::home_dir().expect("failed to determine home dir"),
        }
    }
    pub fn dst_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.prefix.join(path)
    }
}

/// Parser transforms a statement into a Builder.
/// This should be purely syntactical.
pub trait Parser {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn parse(&self, workdir: &Path, args: &[&str]) -> Result<Box<dyn Builder>>;
    /// [Optional] Register Handlebars helper.
    fn register_render_helper(&self, tera: &mut Tera) -> Result<()> {
        let _ = tera;
        Ok(())
    }
}

/// Builder is creates a Module or modifies State.
pub trait Builder: std::fmt::Debug {
    fn build(&self, state: &mut State) -> Result<Option<Box<dyn Module>>>;
}

fn parsers() -> Vec<Box<dyn Parser>> {
    let result: Vec<Vec<Box<dyn Parser>>> = vec![
        // MANIFEST.
        super::subdir::commands(),
        super::prefix::commands(),
        super::dirs_prefix::commands(),
        super::xdg_prefix::commands(),
        super::tags::commands(),
        // Files.
        super::symlink::commands(),
        super::symlink_tree::commands(),
        super::mkdir::commands(),
        super::copy::commands(),
        super::output_file::commands(),
        super::cat_glob::commands(),
        super::set_contents::commands(),
        super::importer::commands(),
        super::render::commands(),
        // Downloads.
        super::fetch::commands(),
        super::git_clone::commands(),
        // Exec.
        super::exec::commands(),
        // Control.
        super::if_executable::commands(),
        super::if_missing::commands(),
        super::if_os::commands(),
        // Deprecation.
        super::deprecated::commands(),
    ];
    result.into_iter().flatten().collect()
}

pub fn parse(workdir: &Path, args: &[&str]) -> Result<Box<dyn Builder>> {
    let mut matched: Vec<(String, Box<dyn Builder>)> = Vec::new();
    for parser in parsers() {
        match parser.parse(workdir, args) {
            Ok(builder) => matched.push((parser.name(), builder)),
            Err(err) => {
                match err.downcast_ref::<Error>() {
                    Some(Error::UnsupportedCommand {
                        builder: _,
                        command: _,
                    }) => {
                        // Try another builder.
                        continue;
                    }
                    _ => {
                        return Err(err);
                    }
                }
            }
        }
    }
    match matched.len() {
        0 => Err(anyhow!("unsupported command {:?}", args)),
        1 => Ok(matched.pop().unwrap().1),
        _ => Err(anyhow!(
            "{:?} matched multiple parsers: {:?}",
            args,
            matched.iter().map(|(parser, _)| parser).collect::<Vec<_>>(),
        )),
    }
}

pub fn register_render_helpers(tera: &mut Tera) -> Result<()> {
    for parser in parsers() {
        parser
            .register_render_helper(tera)
            .with_context(|| format!("failed to register {} helper", parser.name()))?;
    }
    Ok(())
}

pub fn help() -> String {
    let mut help = String::new();
    for parser in parsers() {
        help.push_str(parser.help().trim_end());
        help.push('\n');
    }
    help
}
