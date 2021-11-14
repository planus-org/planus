use anyhow::Result;
use planus_cli::codegen::generate_code;

#[derive(structopt::StructOpt)]
struct Args {
    files: Vec<String>,

    /// Output file, stdout if not present
    #[structopt(short = "o")]
    output_filename: String,
}

#[paw::main]
fn main(args: Args) -> Result<()> {
    generate_code(&args.files, args.output_filename)?;

    Ok(())
}
