use anyhow::Result;
use clap::StructOpt;

fn main() -> Result<()> {
    let args = planus_cli::app::App::parse();

    args.run()
}
