use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueHint};

use crate::ctx::Ctx;

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
    pub fn run(self, _options: super::AppOptions) -> Result<()> {
        let mut ctx = Ctx::default();
        let file_id = ctx.add_file(&self.file, []).unwrap();
        if let Some(parsed) = ctx.parse_file(file_id) {
            if ctx.has_errors() && !self.ignore_errors {
                anyhow::bail!("Bailing because of errors");
            } else {
                let mut s = String::new();
                crate::cst::pretty_print(ctx.get_source(file_id), &parsed, &mut s)?;
                print!("{s}");
            }
        }
        Ok(())
    }
}
