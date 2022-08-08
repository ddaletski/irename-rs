use irn::app::{App, AppResult};
use irn::cli::parse_args;

use std::collections::HashSet;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

/// check if all items of an iterator are unique
fn unique<T>(mut items: T) -> bool
where
    T: Iterator,
    <T as Iterator>::Item: Eq,
    <T as Iterator>::Item: std::hash::Hash,
{
    let mut set: HashSet<T::Item> = HashSet::new();

    items.all(move |item| set.insert(item))
}

fn main() -> anyhow::Result<()> {
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
        .with_regex(args.regex.unwrap_or_default());

    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(result) => match result {
            AppResult::MoveFiles(move_pairs) => {
                if !unique(move_pairs.iter().map(|pair| &pair.1)) {
                    anyhow::bail!("destination files are not unique. Aborting")
                }

                for (src, dst) in move_pairs {
                    let command = format!("mv {} {}", src.to_str().unwrap(), dst.to_str().unwrap());

                    if args.dry_run {
                        println!("{}", command);
                    } else {
                        std::fs::rename(src, dst)?;
                    }
                }
            }
            AppResult::Exit => {}
        },
        Err(err) => {
            eprintln!("{:?}", err)
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(vec![], true)]
    #[case(vec!["a"], true)]
    #[case(vec!["a", "b", "c", "d"], true)]
    #[case(vec!["a", "a"], false)]
    #[case(vec!["a", "b", "c", "a", "d"], false)]
    fn unique_works(#[case] items: Vec<&str>, #[case] expected_result: bool) {
        assert_eq!(unique(items.iter()), expected_result);
    }
}
