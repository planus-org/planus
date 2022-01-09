mod check;
mod format;
mod rust;

use clap::StructOpt;

#[derive(StructOpt)]
pub struct App {
    #[clap(flatten)]
    app_options: AppOptions,

    #[clap(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
pub enum Command {
    Rust(rust::Command),
    Format(format::Command),
    Check(check::Command),
}

#[derive(Default, StructOpt)]
pub struct AppOptions {}

impl App {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Rust(command) => command.run(self.app_options)?,
            Command::Format(command) => command.run(self.app_options)?,
            Command::Check(command) => command.run(self.app_options)?,
        }
        Ok(())
    }
}
