use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueHint};

use crate::{ast_map::AstMap, ctx::Ctx, intermediate_language::translation::Translator};

/// Check validity of .fbs files
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) -> Result<()> {
        let mut ctx = Ctx::default();
        let mut ast_map = AstMap::default();
        for file in self.files {
            if let Some(file_id) = ctx.add_file(&file, []) {
                ast_map.add_files_recursively(&mut ctx, file_id);
            }
        }

        let mut translator = Translator::new(&ctx, ast_map.reachability());
        for schema in ast_map.iter() {
            translator.add_schema(schema);
        }

        let _ = translator.finish();

        if ctx.has_errors() {
            anyhow::bail!("Bailing because of errors");
        }

        Ok(())
    }
}
