use std::env::var;
use std::env::VarError;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

/// Environment variable name for the Spin binary path.
pub const BIN_PATH: &str = "SPIN_BIN_PATH";

pub struct Spin {
    bin: PathBuf,
}

impl From<PathBuf> for Spin {
    fn from(bin: PathBuf) -> Self {
        Self { bin }
    }
}

impl Spin {
    pub fn current() -> Result<Self, VarError> {
        let bin = var(BIN_PATH)?;
        Ok(Self { bin: bin.into() })
    }

    pub fn command(&self) -> Command {
        Command::new(&self.bin)
    }

    pub fn plugin_install_file(&self, file: impl AsRef<Path>) -> Result<ExitStatus, std::io::Error> {
        self.command()
            .arg("plugin")
            .arg("install")
            .arg("--file")
            .arg(file.as_ref())
            .arg("--yes")
            .spawn()?
            .wait()
    }
}
