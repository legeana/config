use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indoc::formatdoc;

use crate::module::{Module, ModuleBox, Rules};
use crate::registry::{FileType, Registry};

use super::args::Arguments;
use super::engine;
use super::inventory;
use super::local_state;

struct Once {
    pre_install_tag: PathBuf,
    install_tag: PathBuf,
    post_install_tag: PathBuf,
    module: ModuleBox,
}

fn wrap<F>(tag: &Path, f: F, force: bool, registry: &mut dyn Registry) -> Result<()>
where
    F: FnOnce(&mut dyn Registry) -> Result<()>,
{
    if !force
        && tag
            .try_exists()
            .with_context(|| format!("failed to check if {tag:?} exists"))?
    {
        return Ok(());
    }
    f(registry)?;
    match std::fs::create_dir(tag) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err).with_context(|| format!("failed to create {tag:?} directory")),
    }?;
    registry
        .register_state_file(tag, FileType::Directory)
        .with_context(|| format!("failed to register state file {tag:?}"))
}

impl Module for Once {
    // TODO: Cleanup tags on full uninstall, will need to distinguish between
    // full uninstall and pre_uninstall before install.
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        self.module.pre_uninstall(rules)
    }
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        wrap(
            &self.pre_install_tag,
            |registry| self.module.pre_install(rules, registry),
            rules.force_download,
            registry,
        )
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        wrap(
            &self.install_tag,
            |registry| self.module.install(rules, registry),
            rules.force_download,
            registry,
        )
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        wrap(
            &self.post_install_tag,
            |registry| self.module.post_install(rules, registry),
            rules.force_download,
            registry,
        )
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        self.module.system_install(rules)
    }
}

#[derive(Debug)]
struct OnceStatement {
    workdir: PathBuf,
    tag: String,
    statement: engine::StatementBox,
}

impl engine::Statement for OnceStatement {
    fn eval(&self, ctx: &mut engine::Context) -> Result<Option<ModuleBox>> {
        match self.statement.eval(ctx)? {
            Some(module) => {
                let tags = local_state::ephemeral_dir_state(&self.workdir, &self.tag)?;
                let pre_install_tag = tags.path().join("pre_install");
                let install_tag = tags.path().join("install");
                let post_install_tag = tags.path().join("post_install");
                Ok(Some(Box::new((
                    tags,
                    Once {
                        pre_install_tag,
                        install_tag,
                        post_install_tag,
                        module,
                    },
                ))))
            }
            None => Ok(None),
        }
    }
}

#[derive(Clone)]
struct OnceBuilder;

impl engine::WithWrapperBuilder for OnceBuilder {
    fn name(&self) -> String {
        "once".to_owned()
    }
    fn help(&self) -> String {
        formatdoc! {"
            with {command} <tag> {{ ... }}
                Executes command once, unless 'update' is used.
        ", command=self.name()}
    }
    fn build(
        &self,
        workdir: &Path,
        args: &Arguments,
        statement: engine::StatementBox,
    ) -> Result<engine::StatementBox> {
        let tag = args.expect_single_arg(self.name())?;
        Ok(Box::new(OnceStatement {
            workdir: workdir.to_owned(),
            tag: tag.expect_raw().context("tag")?.to_owned(),
            statement,
        }))
    }
}

pub fn register(registry: &mut dyn inventory::Registry) {
    registry.register_with_wrapper(Box::new(OnceBuilder));
}
