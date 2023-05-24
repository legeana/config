use anyhow::Result;

use crate::registry::Registry;

pub struct Rules {
    pub allow_package_install_failures: bool,
}

pub trait Module {
    #[allow(unused_variables)]
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn system_install(&self, rules: &Rules) -> Result<()> {
        Ok(())
    }
}

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

impl Module for Box<dyn Module> {
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
