use std::process::Command;
use std::process::ExitStatus;

#[derive(Debug, Default, serde::Deserialize)]
pub struct Task {
    #[serde(default)]
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

impl Task {
    pub fn run(&self) -> std::io::Result<ExitStatus> {
        let mut command = Command::new(&self.command);
        command.args(&self.args);
        command.spawn()?.wait()
    }
}
