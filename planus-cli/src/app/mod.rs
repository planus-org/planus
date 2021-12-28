mod check;
mod format;
mod rust;

#[derive(structopt::StructOpt)]
pub struct App {
    #[structopt(flatten)]
    app_options: AppOptions,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(structopt::StructOpt)]
pub enum Command {
    Rust(rust::Command),
    Format(format::Command),
    Check(check::Command),
}

#[derive(Default, structopt::StructOpt)]
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
