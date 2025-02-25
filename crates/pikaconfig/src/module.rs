use anyhow::{Context, Result};

use registry::Registry;

#[derive(Default)]
pub struct Rules {
    pub force_update: bool,
    pub force_reinstall: bool,
    pub keep_going: bool,
    pub user_deps: bool,
}

impl Rules {
    pub(crate) fn wrap_keep_going<F>(&self, f: F) -> Result<()>
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
impl_for_part!((<T: Module + ?Sized>), Box<T>, self, (self.as_ref()));
impl_for_part!((<T: Module + ?Sized>), &T, self, (*self));
impl_for_part!(
    (<T0: Module, T1: Module>), (T0, T1),
    self, (self.0) (self.1)
);
impl_for_part!(
    (<T0: Module, T1: Module, T2: Module>), (T0, T1, T2),
    self, (self.0) (self.1) (self.2)
);

macro_rules! impl_wrap {
    ($t:ident, ($($wrap:tt)*), ($sel:ident .$($module:tt)*)) => {
        impl<T: Module> Module for $t<T> {
            fn pre_uninstall(&$sel, rules: &Rules) -> Result<()> {
                $($wrap)*("pre_uninstall", rules, || $sel.$($module)*.pre_uninstall(rules))
            }
            fn pre_install(&$sel, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
                $($wrap)*("pre_install", rules, || $sel.$($module)*.pre_install(rules, registry))
            }
            fn install(&$sel, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
                $($wrap)*("install", rules, || $sel.$($module)*.install(rules, registry))
            }
            fn post_install(&$sel, rules: &Rules, registry: &mut dyn Registry) -> Result<()> {
                $($wrap)*("post_install", rules, || $sel.$($module)*.post_install(rules, registry))
            }
            fn system_install(&$sel, rules: &Rules) -> Result<()> {
                $($wrap)*("system_install", rules, || $sel.$($module)*.system_install(rules))
            }
        }
    };
}

struct WrappedModule<T: Module> {
    module: T,
    error_context: String,
}

impl<T: Module> WrappedModule<T> {
    fn wrap<F>(&self, method: &str, _rules: &Rules, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        f().with_context(|| format!("failed {method} in {:?}", self.error_context))
    }
}

impl_wrap!(WrappedModule, (self.wrap), (self.module));

pub(crate) fn wrap<T: Module + 'static>(module: T, error_context: String) -> ModuleBox {
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

pub(crate) fn wrap_keep_going<T>(modules: Vec<T>) -> ModuleBox
where
    T: Module + 'static,
{
    Box::new(WrappedKeepGoing { modules })
}

struct WrappedUserDeps<T>(T);

impl<T: Module> WrappedUserDeps<T> {
    fn wrap<F>(&self, _method: &str, rules: &Rules, f: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        if rules.user_deps { f() } else { Ok(()) }
    }
}

impl_wrap!(WrappedUserDeps, (self.wrap), (self.0));

pub(crate) fn wrap_user_deps<T>(module: T) -> ModuleBox
where
    T: Module + 'static,
{
    Box::new(WrappedUserDeps(module))
}

pub(crate) struct Dummy;

impl Module for Dummy {}

pub(crate) fn dummy_box() -> ModuleBox {
    Box::new(Dummy)
}
