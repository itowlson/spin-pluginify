use std::path::PathBuf;
use std::process::Command;

pub fn plugin_install_file(name: &str, file: PathBuf) -> Result<(), std::io::Error> {
    let spin = std::env::var("SPIN_BIN_PATH").unwrap_or("spin".into());

    let uninstall_result = Command::new(&spin)
        .arg("plugin")
        .arg("uninstall")
        .arg(name)
        .spawn()?
        .wait();

    if is_fail(&uninstall_result) {
        eprintln!("Failed to uninstall old plugin - continuing");
    }

    Command::new(&spin)
        .arg("plugin")
        .arg("install")
        .arg("--file")
        .arg(file)
        .arg("--yes")
        .spawn()?
        .wait()?;

    Ok(())
}

fn is_fail(res: &std::io::Result<std::process::ExitStatus>) -> bool {
    match res {
        Err(_) => true,
        Ok(st) => st.code() != Some(0),
    }
}
