use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;

/// Check validity of .fbs files
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    schema_file: PathBuf,

    root_type: String,

    #[clap(value_hint = ValueHint::FilePath)]
    data_file: PathBuf,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) -> Result<ExitCode> {
        planus_inspector::app::run_app(&self.schema_file, &self.root_type, &self.data_file)
    }
}
