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

impl Installer for Box<dyn Installer> {
    fn install(&self) -> Result<()> {
        self.as_ref().install()
    }
}
