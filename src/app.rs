use std::{env::split_paths, path::PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::path_utils;

pub struct App {
    /// Current value of the regex input box
    pub regex: String,
    pub source_files: Vec<PathBuf>,
}

impl Default for App {
    fn default() -> App {
        App {
            regex: String::new(),
            source_files: Vec::new(),
        }
    }
}

fn split_path(mut path: PathBuf) -> (PathBuf, String) {
    let filename = path.file_name().unwrap().to_str().unwrap().into();
    path.pop();

    (path, filename)
}

impl App {
    pub fn with_files(mut self, files: Vec<PathBuf>) -> Self {
        self.source_files = files
            .into_iter()
            .map(|path| path_utils::normalize_path(&path))
            .collect();
        self
    }

    pub fn with_regex(mut self, regex: String) -> Self {
        self.regex = regex;
        self
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(KeyEvent { code, modifiers }) = crossterm::event::read()? {
                if code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
                    return Ok(());
                }

                match code {
                    KeyCode::Backspace => {
                        self.regex.pop();
                    }
                    KeyCode::Char(ch) => {
                        self.regex.push(ch);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn ui<B: Backend>(&self, frame: &mut Frame<B>) {
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

        let regex_input = Paragraph::new(self.regex.as_ref())
            .style(Style::default().add_modifier(Modifier::RAPID_BLINK))
            .block(Block::default().title("Regex").borders(Borders::ALL));
        frame.render_widget(regex_input, editor_layout[0]);

        frame.set_cursor(
            // Put cursor past the end of the regex
            editor_layout[0].x + self.regex.len() as u16 + 1,
            // Move one line down, from the border to the regex input
            editor_layout[0].y + 1,
        );

        let items: Vec<ListItem> = self
            .source_files
            .clone()
            .into_iter()
            .map(split_path)
            .map(|(parent, name)| {
                let dir_str = parent.to_str().unwrap().to_owned() + "/";
                let dir_style = Style::default();

                let src_name_style = Style::default().fg(Color::Red);
                let dst_name_style = Style::default().fg(Color::Green);

                let dst_name = name.clone();

                Spans::from(vec![
                    Span::styled(dir_str, dir_style),
                    Span::styled(name, src_name_style),
                    Span::raw("->"),
                    Span::styled(dst_name, dst_name_style),
                ])
            })
            .map(|lines| ListItem::new(lines))
            .collect();

        let files_view =
            List::new(items).block(Block::default().title("Files").borders(Borders::ALL));
        frame.render_widget(files_view, editor_layout[1]);

        let side_pane = Block::default().title("Help").borders(Borders::ALL);
        frame.render_widget(side_pane, hor_layout[1]);
    }
}

#[cfg(test)]
mod tests {
    //#[rstest]
    //def test_split_path() {
    //}

    //fn split_path(path: PathBuf) -> (PathBuf, String) {
    //let filename = path.file_name().unwrap().to_str().unwrap().into();
    //path.pop();

    //(path, filename)
}
