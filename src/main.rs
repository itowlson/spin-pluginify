use clap::Parser;
use pluginify::PluginifyCommand;

fn main() -> Result<(), anyhow::Error> {
    PluginifyCommand::parse().run()
}
