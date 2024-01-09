use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;
use planus_translation::translate_files_with_options;

/// Check validity of .fbs files
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,
}

impl Command {
    pub fn run(self, options: super::AppOptions) -> Result<ExitCode> {
        if translate_files_with_options(&self.files, options.to_converter_options()).is_none() {
            Ok(ExitCode::FAILURE)
        } else {
            Ok(ExitCode::SUCCESS)
        }
    }
}
