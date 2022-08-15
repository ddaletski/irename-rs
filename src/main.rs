use irename::app::{App, AppResult};
use irename::cli::parse_args;

use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use tui::{backend::TermionBackend, Terminal};

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

    let files = {
        if !args.files.is_empty() {
            args.files.clone()
        } else {
            // if there are no files provided - read paths from stdin
            std::io::stdin()
                .lines()
                .filter_map(|l| l.ok())
                .filter_map(|s| PathBuf::from_str(&s).ok())
                .collect()
        }
    };

    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // run the app
    let mut app = App::default()
        .with_files(files)
        .with_regex(args.regex.unwrap_or_default())
        .with_replacement(args.replace.unwrap_or_default());

    let res = app.run(&mut terminal);
    drop(terminal); // restore terminal state

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
