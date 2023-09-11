use std::process::ExitCode;

use clap::Parser;
use color_eyre::Result;

mod app;

fn main() -> Result<ExitCode> {
    color_eyre::install()?;

    let args = crate::app::App::parse();

    args.run()
}
