use std::process::ExitCode;

use clap::StructOpt;
use color_eyre::Result;

mod app;

fn main() -> Result<ExitCode> {
    color_eyre::install()?;

    let args = crate::app::App::parse();

    args.run()
}
