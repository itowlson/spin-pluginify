// MUST KEEP IN SYNC WITH SPIN

use serde::{Deserialize, Serialize};

/// Expected schema of a plugin manifest. Should match the latest Spin plugin
/// manifest JSON schema:
/// https://github.com/fermyon/spin-plugins/tree/main/json-schema
#[derive(Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Name of the plugin.
    pub(crate) name: String,
    /// Option description of the plugin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    /// Optional address to the homepage of the plugin producer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) homepage: Option<String>,
    /// Version of the plugin.
    pub(crate) version: String,
    /// Versions of Spin that the plugin is compatible with.
    pub(crate) spin_compatibility: String,
    /// License of the plugin.
    pub(crate) license: String,
    /// Points to source package[s] of the plugin..
    pub(crate) packages: Vec<PluginPackage>,
}

/// Describes compatibility and location of a plugin source.
#[derive(Serialize, Debug, Deserialize, PartialEq)]
pub struct PluginPackage {
    /// Compatible OS.
    pub(crate) os: Os,
    /// Compatible architecture.
    pub(crate) arch: Architecture,
    /// Address to fetch the plugin source tar file.
    pub(crate) url: String,
    /// Checksum to verify the plugin before installation.
    pub(crate) sha256: String,
}

/// Describes the compatible OS of a plugin
#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Os {
    Linux,
    Macos,
    Windows,
}

impl Os {
    pub(crate) fn current() -> Self {
        match std::env::consts::OS {
            "linux" => Os::Linux,
            "macos" => Os::Macos,
            "windows" => Os::Windows,
            _ => panic!("Unsupported OS {}", std::env::consts::OS),
        }
    }
}

/// Describes the compatible architecture of a plugin
#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Architecture {
    Amd64,
    Aarch64,
    Arm,
}

impl Architecture {
    pub(crate) fn current() -> Self {
        match std::env::consts::ARCH {
            "x86_64" => Architecture::Amd64,
            "aarch64" => Architecture::Aarch64,
            "arm64" => Architecture::Arm,
            _ => panic!("Unsupported architecture {}", std::env::consts::ARCH),
        }
    }
}
