use anyhow::{Context, Result};

use crate::registry::Registry;

#[derive(Default)]
pub struct Rules {
    pub allow_package_install_failures: bool,
    pub force_download: bool,
}

pub trait Module {
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        let _ = rules;
        let _ = registry;
        Ok(())
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        let _ = rules;
        let _ = registry;
        Ok(())
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        let _ = rules;
        let _ = registry;
        Ok(())
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        let _ = rules;
        Ok(())
    }
}

pub type ModuleBox = Box<dyn Module>;

impl<T: Module> Module for Vec<T> {
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        for module in self {
            module.pre_install(rules, registry)?;
        }
        Ok(())
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        for module in self {
            module.install(rules, registry)?;
        }
        Ok(())
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        for module in self {
            module.post_install(rules, registry)?;
        }
        Ok(())
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        for module in self {
            module.system_install(rules)?;
        }
        Ok(())
    }
}

impl Module for ModuleBox {
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.as_ref().pre_install(rules, registry)
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.as_ref().install(rules, registry)
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.as_ref().post_install(rules, registry)
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        self.as_ref().system_install(rules)
    }
}

struct WrappedModule<T: Module> {
    module: T,
    error_context: String,
}

impl<T: Module> Module for WrappedModule<T> {
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.module.pre_install(rules, registry)
            .with_context(|| format!("failed pre_install in {:?}", self.error_context))
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.module.install(rules, registry)
            .with_context(|| format!("failed install in {:?}", self.error_context))
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.module.post_install(rules, registry)
            .with_context(|| format!("failed post_install in {:?}", self.error_context))
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        self.module.system_install(rules)
            .with_context(|| format!("failed system_install in {:?}", self.error_context))
    }
}

pub fn wrap<T: Module + 'static>(module: T, error_context: String) -> ModuleBox {
    Box::new(WrappedModule {
        error_context,
        module,
    })
}
