use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;

/// View a binary file with a schema in the TUI viewer
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    schema_file: PathBuf,

    root_type: String,

    #[clap(value_hint = ValueHint::FilePath)]
    data_file: PathBuf,
}

impl Command {
    pub fn run(self, options: super::AppOptions) -> Result<ExitCode> {
        let buffer = std::fs::read(&self.data_file)?;
        planus_inspector::app::run_app(
            &self.schema_file,
            &self.root_type,
            &buffer,
            options.to_converter_options(),
        )
    }
}
