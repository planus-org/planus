use anyhow::Result;

#[paw::main]
pub fn main(args: planus_cli::app::App) -> Result<()> {
    args.run()
}
