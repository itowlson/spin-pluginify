use anyhow::{anyhow, Context};
use clap::Parser;
use flate2::{write::GzEncoder, Compression};
use path_absolutize::Absolutize;
use sha2::{Digest, Sha256};
use std::{io::Write, path::PathBuf};

mod plugin_manifest;
mod spin;

type Error = anyhow::Error;

#[derive(Parser)]
struct PluginifyCommand {
    /// The settings file. Defaults to `spin-pluginify.toml`.
    #[clap(name = "FILE", short = 'f')]
    file: Option<PathBuf>,

    /// Overrides the inferred OS - useful for cross compile
    /// situations
    #[clap(long = "os")]
    os_override: Option<String>,

    /// Overrides the inferred architecture - useful for cross compile
    /// situations
    #[clap(long = "arch")]
    arch_override: Option<String>,

    /// Used in multi-platform scenarios to merge per-platform manifests.
    #[clap(
        name = "MERGE",
        long = "merge",
        conflicts_with = "FILE",
        requires = "URL_BASE"
    )]
    merge: bool,

    /// Used in multi-platform scenarios to merge per-platform manifests.
    #[clap(name = "URL_BASE", long = "release-url-base", requires = "MERGE")]
    release_url_base: Option<url::Url>,

    /// Additional logging for diagnostics.
    #[clap(long = "verbose")]
    verbose: bool,

    /// Install the plugin when done.
    #[clap(short, long)]
    install: bool,
}

fn main() -> Result<(), Error> {
    let cmd = PluginifyCommand::parse();
    if cmd.merge {
        cmd.run_merge()
    } else {
        cmd.run_local()
    }
}

impl PluginifyCommand {
    fn run_local(&self) -> Result<(), Error> {
        let file = self
            .file
            .clone()
            .unwrap_or_else(|| PathBuf::from("spin-pluginify.toml"));
        let text = std::fs::read_to_string(&file)?;

        let ps: PackagingSettings = toml::from_str(&text)?;

        let package = self.package(&ps)?;

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
        if self.verbose {
            eprintln!("Manifest JSON:\n{manifest_json}");
        }
        let manifest_path = PathBuf::from(&ps.name).with_extension("json");
        std::fs::write(&manifest_path, manifest_json)?;

        if self.verbose {
            eprintln!(
                "Manifest {}created at {}",
                if manifest_path.exists() { "" } else { "NOT " },
                manifest_path.display()
            );
        }

        if self.install {
            spin::plugin_install_file(manifest_path)?;
        }

        Ok(())
    }

    fn run_merge(&self) -> Result<(), Error> {
        // This expects a set of subdirectories, one per supported platform
        // each containing ONE plugin manifest and ONE `tar.gz` file to which the
        // manifest refers (possibly with mad pathing in the URL but that's okay).
        // Directories without this structure are ignored (it's valid for them to
        // exist, they're just not candidates for merging).
        // let dir_entries = std::fs::read_dir(".")?;
        let subdirs = std::fs::read_dir(".")?
            .filter_map(|de| de.ok())
            .filter(|de| de.file_type().map(|ft| ft.is_dir()).unwrap_or_default())
            .map(|de| de.path());
        // .collect::<Vec<_>>();

        let subdirs_to_merge = subdirs
            .filter_map(|path| self.as_merge_set(&path))
            .collect::<Vec<_>>();

        let mut merged_manifest = None;
        let release_url_base = self
            .release_url_base
            .as_ref()
            .ok_or(anyhow!("must pass a URL base"))?;

        for subdir in subdirs_to_merge {
            let manifest = self.read_manifest_from_dir(&subdir.manifest)?;
            match merged_manifest {
                None => {
                    merged_manifest =
                        Some(self.releasify_url(manifest, &subdir.tar, &release_url_base)?);
                }
                Some(merged) => {
                    merged_manifest =
                        Some(self.merge_info(merged, manifest, &subdir.tar, &release_url_base)?);
                }
            }
        }

        let merged_manifest = match merged_manifest {
            Some(m) => m,
            None => anyhow::bail!("No manifests to merge"),
        };

        println!("{}", serde_json::to_string_pretty(&merged_manifest)?);

        Ok(())
    }

    fn as_merge_set(&self, path: &PathBuf) -> Option<MergeFiles> {
        let files = std::fs::read_dir(path)
            .ok()?
            .filter_map(|de| de.ok())
            .filter(|de| de.file_type().map(|ft| ft.is_file()).unwrap_or_default())
            .map(|de| de.path())
            .collect::<Vec<_>>();

        if files.len() != 2 {
            return None;
        }

        let tar = files
            .clone()
            .into_iter()
            .find(|f| f.extension().unwrap_or_default() == "gz");
        let manifest = files
            .into_iter()
            .find(|f| f.extension().unwrap_or_default() == "json");

        match (tar, manifest) {
            (Some(tar), Some(manifest)) => Some(MergeFiles { tar, manifest }),
            _ => None,
        }
    }

    fn read_manifest_from_dir(
        &self,
        path: &PathBuf,
    ) -> Result<plugin_manifest::PluginManifest, Error> {
        let buf = std::fs::read(path)?;
        let manifest = serde_json::from_slice(&buf)?;
        Ok(manifest)
    }

    fn merge_info(
        &self,
        mut dest: plugin_manifest::PluginManifest,
        source: plugin_manifest::PluginManifest,
        tar_path: &PathBuf,
        release_url_base: &url::Url,
    ) -> Result<plugin_manifest::PluginManifest, Error> {
        let releasified = self.releasify_url(source, tar_path, release_url_base)?;
        let package = releasified
            .packages
            .into_iter()
            .nth(0)
            .ok_or(anyhow!("there is no package"))?;
        dest.packages.push(package);
        Ok(dest)
    }

    fn releasify_url(
        &self,
        mut source: plugin_manifest::PluginManifest,
        tar_path: &PathBuf,
        release_url_base: &url::Url,
    ) -> Result<plugin_manifest::PluginManifest, Error> {
        // We have to go from file://git/hub/stuff/blah.tar.gz -> https://github.com/user/project/releases/download/v<VERSION>>/blah.tar.gz
        // (e.g. https://github.com/fermyon/spin-js-sdk/releases/download/v0.3.0/js2wasm-v0.3.0-linux-amd64.tar.gz)
        // So base should be e.g. "https://github.com/fermyon/spin-js-sdk/releases/download/v0.3.0/"
        let tar_filename = tar_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(anyhow!("can't get tar filename"))?;
        let release_url = release_url_base.join(tar_filename)?;

        let mut package = source
            .packages
            .iter_mut()
            .nth(0)
            .ok_or(anyhow!("there is no package"))?;
        package.url = release_url.to_string();
        Ok(source)
    }

    fn package(&self, ps: &PackagingSettings) -> Result<plugin_manifest::PluginPackage, Error> {
        let os = self
            .os_override
            .clone()
            .unwrap_or_else(|| std::env::consts::OS.to_owned());
        let arch = self
            .arch_override
            .clone()
            .unwrap_or_else(|| std::env::consts::ARCH.to_owned());

        if self.verbose {
            eprintln!("Packaging: os = {os}, arch = {arch}");
        }

        let tar_path = self.tar_package_source(ps, &os, &arch)?;

        if self.verbose {
            eprintln!(
                "Tar archive {}created at {}",
                if tar_path.exists() { "" } else { "NOT " },
                tar_path.display()
            );
        }

        let sha256 = file_digest_string(&tar_path)?;

        let url = url::Url::from_file_path(&tar_path).unwrap().to_string(); // unwrap because Err(()) doesn't convert to anyhow
        if self.verbose {
            eprintln!("Tar archive local URL {url}");
        }

        Ok(plugin_manifest::PluginPackage {
            os: plugin_manifest::Os::parse(&os)?,
            arch: plugin_manifest::Architecture::parse(&arch)?,
            url,
            sha256,
        })
    }

    fn tar_package_source(
        &self,
        ps: &PackagingSettings,
        os: &str,
        arch: &str,
    ) -> Result<PathBuf, Error> {
        let package = infer_package_path(ps);
        if self.verbose {
            eprintln!("Expecting package at {}", package.display());
            eprintln!("...package exists = {}", package.exists());
        }

        let filename = package
            .file_name()
            .ok_or_else(|| anyhow!("Can't get filename of {}", package.display()))?;
        let tar_path = PathBuf::from(format!("{}-{}-{}-{}.tar.gz", ps.name, ps.version, os, arch))
            .absolutize()?
            .to_path_buf();
        if self.verbose {
            eprintln!("About to create tar archive at {}", tar_path.display());
        }

        if self.verbose {
            eprintln!("Appending {} with name {:?}", package.display(), filename);
        }

        let mut writer = std::fs::File::create(&tar_path)?;
        {
            let mut enc = GzEncoder::new(&writer, Compression::default());
            {
                let mut tar_builder = tar::Builder::new(&mut enc);
                tar_builder.append_path_with_name(&package, filename)?;
                tar_builder.finish()?;
            }
            enc.flush()?;
        }
        writer.flush()?;

        if self.verbose {
            eprintln!("Appended {} with name {:?}", package.display(), filename);
        }

        Ok(tar_path)
    }
}

struct MergeFiles {
    manifest: PathBuf,
    tar: PathBuf,
}

#[cfg(not(target_os = "windows"))]
fn infer_package_path(ps: &PackagingSettings) -> PathBuf {
    ps.package.clone()
}

#[cfg(target_os = "windows")]
fn infer_package_path(ps: &PackagingSettings) -> PathBuf {
    let mut package = ps.package.clone();
    {
        if !package.exists() && package.with_extension("exe").exists() {
            package = package.with_extension("exe");
        }
    }
    package
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
