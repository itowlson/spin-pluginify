// MUST KEEP IN SYNC WITH SPIN

use anyhow::anyhow;
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
    pub(crate) fn parse(src: &str) -> anyhow::Result<Self> {
        match src {
            "linux" => Ok(Os::Linux),
            "macos" | "osx" => Ok(Os::Macos),
            "windows" | "win32" => Ok(Os::Windows),
            _ => Err(anyhow!("unknown OS {}", src)),
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
    pub(crate) fn parse(src: &str) -> anyhow::Result<Self> {
        match src {
            "amd64" | "x86_64" => Ok(Architecture::Amd64),
            "aarch64" => Ok(Architecture::Aarch64),
            "arm" => Ok(Architecture::Arm),
            _ => Err(anyhow!("unknown architecture {}", src)),
        }
    }
}
