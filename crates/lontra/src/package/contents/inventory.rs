use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use anyhow::{Result, anyhow};
use minijinja::Environment;

use super::engine;
use crate::jinja;

pub(super) trait Registry {
    fn register_command(&mut self, parser: engine::BoxedCommandBuilder);
    fn register_condition(&mut self, builder: engine::BoxedConditionBuilder);
    fn register_with_wrapper(&mut self, builder: engine::BoxedWithWrapperBuilder);
    fn register_render_helper(&mut self, render_helper: Box<dyn RenderHelper>);
}

pub(super) trait RenderHelper: Sync + Send {
    fn register_globals(&self, env: &mut Environment, ctx: &Arc<jinja::Context>);
}

#[derive(Default)]
struct RegistryImpl {
    commands: HashMap<String, engine::BoxedCommandBuilder>,
    commands_order: Vec<String>,
    conditions: HashMap<String, engine::BoxedConditionBuilder>,
    conditions_order: Vec<String>,
    with_wrappers: HashMap<String, engine::BoxedWithWrapperBuilder>,
    with_wrappers_order: Vec<String>,
    render_helpers: Vec<Box<dyn RenderHelper>>,
}

impl Registry for RegistryImpl {
    fn register_command(&mut self, parser: engine::BoxedCommandBuilder) {
        let name = parser.name();
        let former = self.commands.insert(parser.name(), parser);
        if let Some(former) = former {
            panic!("parser name conflict: {:?}", former.name());
        }
        self.commands_order.push(name);
    }
    fn register_condition(&mut self, builder: engine::BoxedConditionBuilder) {
        let name = builder.name();
        let former = self.conditions.insert(builder.name(), builder);
        if let Some(former) = former {
            panic!("ConditionBuilder name conflict: {:?}", former.name());
        }
        self.conditions_order.push(name);
    }
    fn register_with_wrapper(&mut self, builder: engine::BoxedWithWrapperBuilder) {
        let name = builder.name();
        let former = self.with_wrappers.insert(builder.name(), builder);
        if let Some(former) = former {
            panic!("WithWrapperBuilder name conflict: {:?}", former.name());
        }
        self.with_wrappers_order.push(name);
    }
    fn register_render_helper(&mut self, render_helper: Box<dyn RenderHelper>) {
        self.render_helpers.push(render_helper);
    }
}

fn register_all(registry: &mut dyn Registry) {
    // MANIFEST.
    super::subdir::register(registry);
    super::prefix::register(registry);
    super::dirs_prefix::register(registry);
    super::tags::register(registry);
    // Files.
    super::symlink::register(registry);
    super::symlink_tree::register(registry);
    super::mkdir::register(registry);
    super::copy::register(registry);
    super::output_file::register(registry);
    super::cat_glob::register(registry);
    super::set_contents::register(registry);
    super::importer::register(registry);
    super::render::register(registry);
    // Downloads.
    super::fetch::register(registry);
    super::git_clone::register(registry);
    super::remote_source::register(registry);
    // Exec.
    super::exec::register(registry);
    super::which::register(registry);
    // Control.
    super::file_tests::register(registry);
    super::is_command::register(registry);
    super::is_os::register(registry);
    super::once::register(registry);
    super::return_::register(registry);
    // Deprecation.
    super::deprecated::register(registry);
}

fn registry() -> &'static RegistryImpl {
    static INSTANCE: OnceLock<RegistryImpl> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let mut registry = RegistryImpl::default();
        register_all(&mut registry);
        registry
    })
}

pub(super) fn commands() -> impl Iterator<Item = &'static engine::BoxedCommandBuilder> {
    registry().commands_order.iter().map(|name| {
        registry()
            .commands
            .get(name)
            .expect("parsers_order must match parsers")
    })
}

pub(super) fn command(name: &str) -> Result<&dyn engine::CommandBuilder> {
    registry()
        .commands
        .get(name)
        .ok_or_else(|| anyhow!("unknown command {name}"))
        .map(AsRef::as_ref)
}

pub(super) fn conditions() -> impl Iterator<Item = &'static engine::BoxedConditionBuilder> {
    registry().conditions_order.iter().map(|name| {
        registry()
            .conditions
            .get(name)
            .expect("conditions_order must match conditions")
    })
}

pub(super) fn condition(name: &str) -> Result<&dyn engine::ConditionBuilder> {
    registry()
        .conditions
        .get(name)
        .ok_or_else(|| anyhow!("unknown condition {name}"))
        .map(AsRef::as_ref)
}

pub(super) fn with_wrappers() -> impl Iterator<Item = &'static engine::BoxedWithWrapperBuilder> {
    registry().with_wrappers_order.iter().map(|name| {
        registry()
            .with_wrappers
            .get(name)
            .expect("conditions_order must match conditions")
    })
}

pub(super) fn with_wrapper(name: &str) -> Result<&dyn engine::WithWrapperBuilder> {
    registry()
        .with_wrappers
        .get(name)
        .ok_or_else(|| anyhow!("unknown with wrapper {name}"))
        .map(AsRef::as_ref)
}

pub(super) fn register_render_globals(env: &mut Environment, ctx: &Arc<jinja::Context>) {
    for rh in &registry().render_helpers {
        rh.register_globals(env, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsers_index() {
        assert_eq!(registry().commands.len(), registry().commands_order.len());
        assert_eq!(commands().count(), registry().commands.len());
    }

    #[test]
    fn test_conditions_index() {
        assert_eq!(
            registry().conditions.len(),
            registry().conditions_order.len()
        );
        assert_eq!(conditions().count(), registry().conditions.len());
    }

    #[test]
    fn test_with_wrappers_index() {
        assert_eq!(
            registry().with_wrappers.len(),
            registry().with_wrappers_order.len()
        );
        assert_eq!(with_wrappers().count(), registry().with_wrappers.len());
    }
}
