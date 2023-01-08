use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;

#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    file: PathBuf,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) -> Result<ExitCode> {
        Ok(ExitCode::SUCCESS)
    }
}
