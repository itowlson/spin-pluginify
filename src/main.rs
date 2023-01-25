use anyhow::{anyhow, Context};
use clap::Parser;
use flate2::{write::GzEncoder, Compression};
use path_absolutize::Absolutize;
use sha2::{Sha256, Digest};
use std::{path::PathBuf, io::Write};

mod plugin_manifest;

type Error = anyhow::Error;

#[derive(Parser)]
struct PluginifyCommand {
    #[clap(short = 'f')]
    file: Option<PathBuf>,
}

fn main() -> Result<(), Error> {
    let cmd = PluginifyCommand::parse();
    let file = cmd.file.unwrap_or_else(|| PathBuf::from("spin-pluginify.toml"));
    let text = std::fs::read_to_string(&file)?;

    let ps: PackagingSettings = toml::from_str(&text)?;

    let package = package(&ps)?;

    let manifest = plugin_manifest::PluginManifest {
        name: ps.name.clone(),
        version: ps.version.clone(),
        description: ps.description.clone(),
        homepage: ps.homepage.clone(),
        spin_compatibility: ps.spin_compatibility.clone(),
        license: ps.license.clone(),
        packages: vec![package],
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    let manifest_path = PathBuf::from(&ps.name).with_extension("json");
    std::fs::write(manifest_path, manifest_json)?;

    Ok(())
}

fn package(ps: &PackagingSettings) -> Result<plugin_manifest::PluginPackage, Error> {
    let tar_path = tar_package_source(&ps)?;

    let sha256 = file_digest_string(&tar_path)?;

    let url = url::Url::from_file_path(&tar_path).unwrap().to_string();  // unwrap because Err(()) doesn't convert to anyhow

    Ok(plugin_manifest::PluginPackage {
        os: plugin_manifest::Os::current(),
        arch: plugin_manifest::Architecture::current(),
        url,
        sha256,
    })
}

fn tar_package_source(ps: &PackagingSettings) -> Result<PathBuf, Error> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let filename = ps.package.file_name().ok_or_else(|| anyhow!("Can't get filename of {}", ps.package.display()))?;
    let tar_path = PathBuf::from(format!("{}-{}-{}-{}.tar.gz", ps.name, ps.version, os, arch)).absolutize()?.to_path_buf();

    let mut writer = std::fs::File::create(&tar_path)?;
    {
        let mut enc = GzEncoder::new(&writer, Compression::default());
        {
            let mut tar_builder = tar::Builder::new(&mut enc);
            tar_builder.append_path_with_name(&ps.package, filename)?;
            tar_builder.finish()?;
        }
        enc.flush()?;
    }
    writer.flush()?;

    Ok(tar_path)
}

fn file_digest_string(path: &PathBuf) -> Result<String, Error> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("Could not open file at {}", path.display()))?;
    let mut sha = Sha256::new();
    std::io::copy(&mut file, &mut sha)?;
    let digest_value = sha.finalize();
    let digest_string = format!("{:x}", digest_value);
    Ok(digest_string)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct PackagingSettings {
    name: String,
    version: String,
    // base_uri: String,
    homepage: Option<String>,
    description: Option<String>,
    spin_compatibility: String,
    license: String,
    package: PathBuf,
}
