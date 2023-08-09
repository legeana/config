use anyhow::{Context, Result};

use crate::registry::Registry;

#[derive(Default)]
pub struct Rules {
    pub force_download: bool,
    pub keep_going: bool,
}

impl Rules {
    pub fn wrap_keep_going<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        match f() {
            Ok(_) => Ok(()),
            Err(err) => {
                if self.keep_going {
                    log::error!("{err:?}");
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }
}

pub trait Module {
    /// Used for resource intensive operations to reduce the time without valid
    /// configuration between uninstall and install.
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        let _ = rules;
        Ok(())
    }
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

impl<T: Module> Module for [T] {
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        for module in self {
            module.pre_uninstall(rules)?;
        }
        Ok(())
    }
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

macro_rules! impl_for_part {
    (($($impl:tt)*), $t:ty, $sel:ident, $($part:tt)*) => {
        impl $($impl)* Module for $t {
            fn pre_uninstall(&$sel, rules: &Rules) -> Result<()> {
                $($part.pre_uninstall(rules)?;)*
                Ok(())
            }
            fn pre_install(&$sel, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
                $($part.pre_install(rules, registry)?;)*
                Ok(())
            }
            fn install(&$sel, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
                $($part.install(rules, registry)?;)*
                Ok(())
            }
            fn post_install(&$sel, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
                $($part.post_install(rules, registry)?;)*
                Ok(())
            }
            fn system_install(&$sel, rules: &Rules) -> Result<()> {
                $($part.system_install(rules)?;)*
                Ok(())
            }
        }
    };
}

impl_for_part!((<T: Module>), Vec<T>, self, (self.as_slice()));
impl_for_part!((), ModuleBox, self, (self.as_ref()));
impl_for_part!((<T0: Module, T1: Module>), (T0, T1), self, (self.0) (self.1));

struct WrappedModule<T: Module> {
    module: T,
    error_context: String,
}

impl<T: Module> Module for WrappedModule<T> {
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        self.module
            .pre_uninstall(rules)
            .with_context(|| format!("failed pre_uninstall in {:?}", self.error_context))
    }
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.module
            .pre_install(rules, registry)
            .with_context(|| format!("failed pre_install in {:?}", self.error_context))
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.module
            .install(rules, registry)
            .with_context(|| format!("failed install in {:?}", self.error_context))
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        self.module
            .post_install(rules, registry)
            .with_context(|| format!("failed post_install in {:?}", self.error_context))
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        self.module
            .system_install(rules)
            .with_context(|| format!("failed system_install in {:?}", self.error_context))
    }
}

pub fn wrap<T: Module + 'static>(module: T, error_context: String) -> ModuleBox {
    Box::new(WrappedModule {
        error_context,
        module,
    })
}

struct WrappedKeepGoing<T: Module> {
    modules: Vec<T>,
}

impl<T: Module> Module for WrappedKeepGoing<T> {
    fn pre_uninstall(&self, rules: &Rules) -> Result<()> {
        for m in &self.modules {
            rules.wrap_keep_going(|| m.pre_uninstall(rules))?;
        }
        Ok(())
    }
    fn pre_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        for m in &self.modules {
            rules.wrap_keep_going(|| m.pre_install(rules, registry))?;
        }
        Ok(())
    }
    fn install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        for m in &self.modules {
            rules.wrap_keep_going(|| m.install(rules, registry))?;
        }
        Ok(())
    }
    fn post_install(&self, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
        for m in &self.modules {
            rules.wrap_keep_going(|| m.post_install(rules, registry))?;
        }
        Ok(())
    }
    fn system_install(&self, rules: &Rules) -> Result<()> {
        for m in &self.modules {
            rules.wrap_keep_going(|| m.system_install(rules))?;
        }
        Ok(())
    }
}

pub fn wrap_keep_going<T>(modules: Vec<T>) -> ModuleBox
where
    T: Module + 'static,
{
    Box::new(WrappedKeepGoing { modules })
}

pub struct Dummy;

impl Module for Dummy {}

pub fn dummy_box() -> ModuleBox {
    Box::new(Dummy)
}
