use std::path::PathBuf;
use std::process::Command;

pub fn plugin_install_file(file: PathBuf) -> Result<(), std::io::Error> {
    let spin = std::env::var("SPIN_BIN_PATH").unwrap_or("spin".into());

    Command::new(spin)
        .arg("plugin")
        .arg("install")
        .arg("--file")
        .arg(file)
        .arg("--yes")
        .spawn()?
        .wait()?;

    Ok(())
}
