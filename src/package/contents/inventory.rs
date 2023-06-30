use once_cell::sync::OnceCell;
use tera::Tera;

use super::builder;

pub trait Registry {
    fn register_parser(&mut self, parser: Box<dyn builder::Parser>);
    fn register_render_helper(&mut self, render_helper: Box<dyn RenderHelper>);
}

pub trait RenderHelper: Sync + Send {
    fn register_render_helper(&self, tera: &mut Tera);
}

#[derive(Default)]
struct RegistryImpl {
    commands: Vec<Box<dyn builder::Parser>>,
    render_helpers: Vec<Box<dyn RenderHelper>>,
}

impl Registry for RegistryImpl {
    fn register_parser(&mut self, parser: Box<dyn builder::Parser>) {
        self.commands.push(parser);
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
    // Exec.
    super::exec::register(registry);
    // Control.
    super::if_command::register(registry);
    super::if_missing::register(registry);
    super::if_os::register(registry);
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

pub fn parsers() -> impl Iterator<Item = &'static Box<dyn builder::Parser>> {
    registry().commands.iter()
}

pub fn register_render_helpers(tera: &mut Tera) {
    for rh in &registry().render_helpers {
        rh.register_render_helper(tera);
    }
}
