use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueHint};

use crate::codegen::dot::generate_code;

/// Generate a dot graph
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,

    /// Output file
    #[clap(short = 'o')]
    #[clap(value_hint = ValueHint::AnyPath)]
    output_filename: PathBuf,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) -> Result<()> {
        generate_code(&self.files, &self.output_filename)?;

        Ok(())
    }
}
