use std::path::Path;
use std::path::PathBuf;
use crate::task::Task;

#[derive(Debug, serde::Deserialize)]
pub struct Target {
    package: PathBuf,
    build: Option<Task>,
}

impl Target {
    pub fn package(&self) -> &Path {
        &self.package
    }

    pub fn build(&self) -> Option<&Task> {
        self.build.as_ref()
    }

    #[cfg(not(target_os = "windows"))]
    pub fn infer_package_path(&self) -> PathBuf {
        self.package().to_owned()
    }

    #[cfg(target_os = "windows")]
    pub fn infer_package_path(&self) -> PathBuf {
        let mut package = self.package().to_owned();
        {
            if !package.exists() && package.with_extension("exe").exists() {
                package = package.with_extension("exe");
            }
        }
        package
    }
}
