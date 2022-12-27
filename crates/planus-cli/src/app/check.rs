use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;

use planus_translation::intermediate_language::translate_files;

/// Check validity of .fbs files
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) -> Result<ExitCode> {
        if translate_files(&self.files).is_none() {
            Ok(ExitCode::FAILURE)
        } else {
            Ok(ExitCode::SUCCESS)
        }
    }
}
