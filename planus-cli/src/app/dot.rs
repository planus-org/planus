use anyhow::Result;

use crate::codegen::dot::generate_code;
use clap::StructOpt;

/// Generates a dot graph
#[derive(StructOpt)]
pub struct Command {
    files: Vec<String>,

    /// Output file
    #[structopt(short = 'o')]
    output_filename: String,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) -> Result<()> {
        generate_code(&self.files, &self.output_filename)?;

        Ok(())
    }
}
