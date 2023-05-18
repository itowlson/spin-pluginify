use std::path::Path;
use std::process::Command;

pub struct Spin {
    bin: String
}

impl Spin {
    pub fn new() -> Self {
        let bin = std::env::var("SPIN_BIN_PATH").unwrap_or("spin".to_string());
        Self { bin: bin.into() }
    }

    pub fn command(&self) -> Command {
        Command::new(&self.bin)
    }

    pub fn plugin_install_file(&self, file: impl AsRef<Path>) -> Result<(), std::io::Error> {
        self.command()
            .arg("plugin")
            .arg("install")
            .arg("--file")
            .arg(file.as_ref())
            .arg("--yes")
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn plugin_run(&self, name: &str) -> Result<(), std::io::Error> {
        self.command()
            .arg(name)
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn plugin_uninstall(&self, name: &str) -> Result<(), std::io::Error> {
        self.command()
            .arg("plugin")
            .arg("uninstall")
            .arg(name)
            .spawn()?
            .wait()?;

        Ok(())
    }
}
