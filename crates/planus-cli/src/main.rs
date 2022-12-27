use clap::StructOpt;
use color_eyre::Result;

mod app;
mod pretty_print;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = crate::app::App::parse();

    args.run()
}
