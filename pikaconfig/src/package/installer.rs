use anyhow::Result;

pub trait Installer {
    fn install(&self) -> Result<()>;
}

impl<T: Installer> Installer for Vec<T> {
    fn install(&self) -> Result<()> {
        for installer in self.iter() {
            installer.install()?;
        }
        Ok(())
    }
}

impl<T: Installer + ?Sized> Installer for &T {
    fn install(&self) -> Result<()> {
        T::install(self)
    }
}

impl<T: Installer + ?Sized> Installer for Box<T> {
    fn install(&self) -> Result<()> {
        T::install(self)
    }
}
