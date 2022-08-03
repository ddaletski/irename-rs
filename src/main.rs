use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use tui::{backend::CrosstermBackend, Terminal};

use irn::app::App;
use irn::cli::parse_args;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // run the app
    let mut app = App::default()
        .with_files(args.files)
        .with_regex(args.regex.unwrap_or(String::new()));

    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err)
    }

    Ok(())
}
