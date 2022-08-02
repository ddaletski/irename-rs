use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin},
    widgets::{Block, Borders},
    Frame, Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| ui(f))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>) {
    let hor_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(85), Constraint::Min(15)])
        .split(frame.size());

    let editor_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Percentage(100)])
        .split(hor_layout[0].inner(&Margin {
            vertical: 0,
            horizontal: 0,
        }));

    let block = Block::default().title("Regex").borders(Borders::ALL);
    frame.render_widget(block, editor_layout[0]);

    let block = Block::default().title("Result").borders(Borders::ALL);
    frame.render_widget(block, editor_layout[1]);

    let block = Block::default().title("Help").borders(Borders::ALL);
    frame.render_widget(block, hor_layout[1]);
}
