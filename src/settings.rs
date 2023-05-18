use std::path::PathBuf;

use crate::plugin_manifest::PluginManifest;
use crate::plugin_manifest::PluginPackage;



#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PackagingSettings {
    name: String,
    version: String,
    // base_uri: String,
    homepage: Option<String>,
    description: Option<String>,
    spin_compatibility: String,
    license: String,
    package: PathBuf,
}

impl PackagingSettings {
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(&s)
    }

    pub fn plugin_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn plugin_homepage(&self) -> Option<String> {
        self.homepage.to_owned()
    }

    pub fn plugin_description(&self) -> Option<String> {
        self.description.to_owned()
    }

    pub fn plugin_version(&self) -> String {
        self.version.to_owned()
    }

    pub fn plugin_spin_compatibility(&self) -> String {
        self.spin_compatibility.to_owned()
    }

    pub fn plugin_license(&self) -> String {
        self.license.to_owned()
    }

    pub fn manifest_path(&self) -> PathBuf {
        PathBuf::from(&self.plugin_name()).with_extension("json")
    }

    #[cfg(not(target_os = "windows"))]
    pub fn infer_package_path(&self) -> PathBuf {
        self.package.clone()
    }

    #[cfg(target_os = "windows")]
    pub fn infer_package_path(&self) -> PathBuf {
        let mut package = self.package.clone();
        {
            if !package.exists() && package.with_extension("exe").exists() {
                package = package.with_extension("exe");
            }
        }
        package
    }
}

impl PluginManifest {
    pub(crate) fn new(
        settings: &PackagingSettings,
        packages: impl IntoIterator<Item = PluginPackage>,
    ) -> Self {
        Self {
            name: settings.plugin_name(),
            description: settings.plugin_description(),
            homepage: settings.plugin_homepage(),
            version: settings.plugin_version(),
            spin_compatibility: settings.plugin_spin_compatibility(),
            license: settings.plugin_license(),
            packages: packages.into_iter().collect(),
        }
    }
}
