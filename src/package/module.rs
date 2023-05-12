use anyhow::Result;

use crate::registry::Registry;

pub trait Module {
    #[allow(unused_variables)]
    fn pre_install(&self, registry: &mut dyn Registry) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn post_install(&self, registry: &mut dyn Registry) -> Result<()> {
        Ok(())
    }
    fn system_install(&self) -> Result<()> {
        Ok(())
    }
}

impl<T: Module> Module for Vec<T> {
    fn pre_install(&self, registry: &mut dyn Registry) -> Result<()> {
        for module in self {
            module.pre_install(registry)?;
        }
        Ok(())
    }
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        for module in self {
            module.install(registry)?;
        }
        Ok(())
    }
    fn post_install(&self, registry: &mut dyn Registry) -> Result<()> {
        for module in self {
            module.post_install(registry)?;
        }
        Ok(())
    }
    fn system_install(&self) -> Result<()> {
        for module in self {
            module.system_install()?;
        }
        Ok(())
    }
}

impl Module for Box<dyn Module> {
    fn pre_install(&self, registry: &mut dyn Registry) -> Result<()> {
        self.as_ref().pre_install(registry)
    }
    fn install(&self, registry: &mut dyn Registry) -> Result<()> {
        self.as_ref().install(registry)
    }
    fn post_install(&self, registry: &mut dyn Registry) -> Result<()> {
        self.as_ref().post_install(registry)
    }
    fn system_install(&self) -> Result<()> {
        self.as_ref().system_install()
    }
}