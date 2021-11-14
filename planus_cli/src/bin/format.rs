use anyhow::Result;
use planus_cli::ctx::Ctx;

#[derive(structopt::StructOpt)]
struct Args {
    filename: String,
}

#[paw::main]
fn main(args: Args) -> Result<()> {
    let mut ctx = Ctx::default();
    let file_id = ctx.add_file(&args.filename).unwrap();
    if let Some(parsed) = ctx.parse_file(file_id) {
        let mut s = String::new();
        planus_cli::cst::pretty_print(ctx.get_source(file_id), &parsed, &mut s)?;
        print!("{}", s);
    }
    Ok(())
}
