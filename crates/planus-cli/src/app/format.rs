use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;
use planus_translation::format_file;

/// Format .fbs files
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    file: PathBuf,

    /// Try to generate output even if the input has errors
    #[clap(long)]
    ignore_errors: bool,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) -> Result<ExitCode> {
        if let Some(out) = format_file(&self.file, self.ignore_errors) {
            println!("{out}");
            Ok(ExitCode::SUCCESS)
        } else {
            Ok(ExitCode::FAILURE)
        }
    }
}
