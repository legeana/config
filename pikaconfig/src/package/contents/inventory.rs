use std::collections::HashMap;

use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use tera::Tera;

use super::engine;

pub trait Registry {
    fn register_command(&mut self, parser: engine::CommandBuilderBox);
    fn register_condition(&mut self, builder: engine::ConditionBuilderBox);
    fn register_with_wrapper(&mut self, builder: engine::WithWrapperBuilderBox);
    fn register_render_helper(&mut self, render_helper: Box<dyn RenderHelper>);
}

pub trait RenderHelper: Sync + Send {
    fn register_render_helper(&self, tera: &mut Tera);
}

#[derive(Default)]
struct RegistryImpl {
    commands: HashMap<String, engine::CommandBuilderBox>,
    commands_order: Vec<String>,
    conditions: HashMap<String, engine::ConditionBuilderBox>,
    conditions_order: Vec<String>,
    with_wrappers: HashMap<String, engine::WithWrapperBuilderBox>,
    with_wrappers_order: Vec<String>,
    render_helpers: Vec<Box<dyn RenderHelper>>,
}

impl Registry for RegistryImpl {
    fn register_command(&mut self, parser: engine::CommandBuilderBox) {
        let name = parser.name();
        let former = self.commands.insert(parser.name(), parser);
        if let Some(former) = former {
            panic!("parser name conflict: {:?}", former.name());
        }
        self.commands_order.push(name);
    }
    fn register_condition(&mut self, builder: engine::ConditionBuilderBox) {
        let name = builder.name();
        let former = self.conditions.insert(builder.name(), builder);
        if let Some(former) = former {
            panic!("ConditionBuilder name conflict: {:?}", former.name());
        }
        self.conditions_order.push(name);
    }
    fn register_with_wrapper(&mut self, builder: engine::WithWrapperBuilderBox) {
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
    // Control.
    super::file_tests::register(registry);
    super::literal::register(registry);
    super::is_command::register(registry);
    super::is_os::register(registry);
    super::once::register(registry);
    super::return_::register(registry);
    // Deprecation.
    super::deprecated::register(registry);
}

fn registry() -> &'static RegistryImpl {
    static INSTANCE: OnceCell<RegistryImpl> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let mut registry = RegistryImpl::default();
        register_all(&mut registry);
        registry
    })
}

pub fn commands() -> impl Iterator<Item = &'static engine::CommandBuilderBox> {
    registry().commands_order.iter().map(|name| {
        registry()
            .commands
            .get(name)
            .expect("parsers_order must match parsers")
    })
}

pub fn command(name: &str) -> Result<&dyn engine::CommandBuilder> {
    registry()
        .commands
        .get(name)
        .ok_or_else(|| anyhow!("unknown command {name}"))
        .map(|p| p.as_ref())
}

pub fn conditions() -> impl Iterator<Item = &'static engine::ConditionBuilderBox> {
    registry().conditions_order.iter().map(|name| {
        registry()
            .conditions
            .get(name)
            .expect("conditions_order must match conditions")
    })
}

pub fn condition(name: &str) -> Result<&dyn engine::ConditionBuilder> {
    registry()
        .conditions
        .get(name)
        .ok_or_else(|| anyhow!("unknown condition {name}"))
        .map(|p| p.as_ref())
}

pub fn with_wrappers() -> impl Iterator<Item = &'static engine::WithWrapperBuilderBox> {
    registry().with_wrappers_order.iter().map(|name| {
        registry()
            .with_wrappers
            .get(name)
            .expect("conditions_order must match conditions")
    })
}

pub fn with_wrapper(name: &str) -> Result<&dyn engine::WithWrapperBuilder> {
    registry()
        .with_wrappers
        .get(name)
        .ok_or_else(|| anyhow!("unknown with wrapper {name}"))
        .map(|p| p.as_ref())
}

pub fn register_render_helpers(tera: &mut Tera) {
    for rh in &registry().render_helpers {
        rh.register_render_helper(tera);
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
