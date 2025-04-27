use anyhow::Result;

use crate::module::Rules;

pub(super) trait Installer {
    fn install(&self, rules: &Rules) -> Result<()>;
}

impl<T: Installer> Installer for Vec<T> {
    fn install(&self, rules: &Rules) -> Result<()> {
        for installer in self {
            installer.install(rules)?;
        }
        Ok(())
    }
}

impl<T: Installer + ?Sized> Installer for &T {
    fn install(&self, rules: &Rules) -> Result<()> {
        T::install(self, rules)
    }
}

impl<T: Installer + ?Sized> Installer for Box<T> {
    fn install(&self, rules: &Rules) -> Result<()> {
        T::install(self, rules)
    }
}
