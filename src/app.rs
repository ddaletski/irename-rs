use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

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

impl App {
    pub fn with_files(mut self, files: Vec<PathBuf>) -> Self {
        self.source_files = files;
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
            .iter()
            .filter_map(|file_path| file_path.file_name())
            .filter_map(|name_os_str| name_os_str.to_str())
            .map(|name| ListItem::new(name))
            .collect();

        let files_view =
            List::new(items).block(Block::default().title("Files").borders(Borders::ALL));
        frame.render_widget(files_view, editor_layout[1]);

        let side_pane = Block::default().title("Help").borders(Borders::ALL);
        frame.render_widget(side_pane, hor_layout[1]);
    }
}
