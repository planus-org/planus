use clap::{Parser, ValueHint};
use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use planus_inspector::{run_inspector, Inspector};
use planus_translation::translate_files;
use std::{io, path::PathBuf, process::ExitCode, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser)]
pub struct App {
    #[clap(value_hint = ValueHint::FilePath)]
    data_file: PathBuf,

    #[clap(value_hint = ValueHint::FilePath)]
    schema_files: Vec<PathBuf>,
}

fn main() -> Result<ExitCode> {
    let args = App::parse();
    let data = std::fs::read(args.data_file)?;

    let Some(_declarations) = translate_files(&args.schema_files)
    else {
        return Ok(ExitCode::FAILURE);
    };

    let tick_rate = Duration::from_millis(100);
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let inspector = Inspector::new(&data);
    let res = run_inspector(&mut terminal, inspector, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(ExitCode::SUCCESS)
}
