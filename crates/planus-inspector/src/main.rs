use clap::{Parser, ValueHint};
use color_eyre::Result;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use planus_inspector::{run_inspector, Inspector};
use planus_translation::translate_files;
use std::io::Write;
use std::{io, ops::DerefMut, path::PathBuf, process::ExitCode, time::Duration};
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

    let Some(declarations) = translate_files(&args.schema_files)
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

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |x| {
        let mut stdout = io::stdout();
        cleanup_terminal(&mut stdout).ok();
        hook(x);
    }));

    // create app and run it
    let inspector = Inspector::new(&data);
    let res = run_inspector(&mut terminal, inspector, tick_rate);

    cleanup_terminal(terminal.backend_mut().deref_mut())?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(ExitCode::SUCCESS)
}

fn cleanup_terminal(stdout: &mut impl std::io::Write) -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show)?;
    Ok(())
}
