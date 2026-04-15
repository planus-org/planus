use std::{io::Write, path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;
use planus_codegen::{generate_rust, RustOptions};
use planus_translation::translate_files_with_options;

/// Generate rust code
#[derive(Parser)]
pub struct Command {
    #[clap(value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,

    /// Output file
    #[clap(short = 'o')]
    #[clap(value_hint = ValueHint::AnyPath)]
    output_filename: PathBuf,

    /// Run rustfmt on the generated code
    #[clap(long, default_value_t = true, action = clap::ArgAction::Set)]
    format: bool,

    /// Emit `#[serde(tag = "<NAME>")]` on generated union enums, producing
    /// an internally tagged serde representation (e.g. `{"type": "Variant",
    /// ...fields}`) instead of the default externally tagged one
    /// (`{"Variant": {...}}`).
    #[clap(long, value_name = "NAME")]
    serde_tag: Option<String>,
}

impl Command {
    pub fn run(self, options: super::AppOptions) -> Result<ExitCode> {
        let Some(declarations) =
            translate_files_with_options(&self.files, options.to_converter_options())
        else {
            return Ok(ExitCode::FAILURE);
        };

        let res = generate_rust(
            &declarations,
            &RustOptions {
                format: self.format,
                serde_enum_tag: self.serde_tag,
            },
        )?;
        let mut file = std::fs::File::create(&self.output_filename)?;
        file.write_all(res.as_bytes())?;
        file.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
