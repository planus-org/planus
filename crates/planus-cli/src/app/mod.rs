mod check;
mod dot;
mod format;
mod gen_completions;
mod inspect;
mod rust;

use std::process::ExitCode;

use clap::Parser;
use color_eyre::Result;

#[derive(Parser)]
pub struct App {
    #[clap(flatten)]
    app_options: AppOptions,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub enum Command {
    Dot(dot::Command),
    Rust(rust::Command),
    Inspect(inspect::Command),
    Format(format::Command),
    Check(check::Command),
    GenerateCompletions(gen_completions::Command),
}

#[derive(Default, Parser)]
pub struct AppOptions {}

impl App {
    pub fn run(self) -> Result<ExitCode> {
        match self.command {
            Command::Dot(command) => command.run(self.app_options),
            Command::Rust(command) => command.run(self.app_options),
            Command::Format(command) => command.run(self.app_options),
            Command::Check(command) => command.run(self.app_options),
            Command::GenerateCompletions(command) => command.run(self.app_options),
            Command::Inspect(command) => command.run(self.app_options),
        }
    }
}
