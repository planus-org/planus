use std::{io::Write, path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;
use planus_codegen::generate_rust;
use planus_translation::translate_files;

/// Generate rust code
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
    pub fn run(self, _options: super::AppOptions) -> Result<ExitCode> {
        let Some(declarations) = translate_files(&self.files)
        else {
            return Ok(ExitCode::FAILURE)
        };

        let res = generate_rust(&declarations)?;
        let mut file = std::fs::File::create(&self.output_filename)?;
        file.write_all(res.as_bytes())?;
        file.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
