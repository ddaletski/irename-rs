use regex::Regex;
use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use num_derive::{FromPrimitive, ToPrimitive};
use variant_count::VariantCount;

use crate::path_utils;

#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive, VariantCount)]
enum EditableArea {
    Regex,
    Replace,
}

impl EditableArea {
    fn next(&self) -> Self {
        let num_value = num::ToPrimitive::to_usize(self).unwrap();
        let next_value = (num_value.overflowing_add(1).0) % EditableArea::VARIANT_COUNT;

        num::FromPrimitive::from_usize(next_value).unwrap()
    }

    fn prev(&self) -> Self {
        let num_value = num::ToPrimitive::to_usize(self).unwrap();
        let prev_value = (num_value.overflowing_sub(1).0) % EditableArea::VARIANT_COUNT;

        num::FromPrimitive::from_usize(prev_value).unwrap()
    }
}

pub struct App {
    /// Current value of the regex input box
    regex: String,
    /// Current value of the replacement string box
    replacement: String,
    /// active editing area where the cursor is
    active_area: EditableArea,
    /// source files to rename
    source_files: Vec<PathBuf>,
}

impl Default for App {
    fn default() -> App {
        App {
            regex: String::new(),
            replacement: String::new(),
            active_area: EditableArea::Regex,
            source_files: Vec::new(),
        }
    }
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

    pub fn with_replacement(mut self, replacement: String) -> Self {
        self.replacement = replacement;
        self
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(KeyEvent { code, modifiers }) = crossterm::event::read()? {
                // ctrl-c handler
                if code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
                    return Ok(());
                }

                let edited_string = match self.active_area {
                    EditableArea::Regex => &mut self.regex,
                    EditableArea::Replace => &mut self.replacement,
                };

                match code {
                    KeyCode::Tab => {
                        if modifiers == KeyModifiers::SHIFT {
                            self.active_area = self.active_area.prev();
                        } else {
                            self.active_area = self.active_area.next();
                        }
                    }
                    KeyCode::Backspace => {
                        edited_string.pop();
                    }
                    KeyCode::Char(ch) => {
                        edited_string.push(ch);
                    }
                    _ => {}
                }
            }
        }
    }

    fn ui<B: Backend>(&self, frame: &mut Frame<B>) {
        // editor and help areas
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(85), Constraint::Min(15)])
            .split(frame.size());

        // editor area: regex, replacement, files list
        let editor_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .split(main_layout[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }));

        // regex and replacement inputs
        let input_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(editor_layout[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }));

        let regex_input = Paragraph::new(self.regex.as_ref())
            .block(Block::default().title("Regex").borders(Borders::ALL));
        frame.render_widget(regex_input, input_layout[0]);

        let replace_input = Paragraph::new(self.replacement.as_ref())
            .block(Block::default().title("Replacement").borders(Borders::ALL));
        frame.render_widget(replace_input, input_layout[1]);

        match self.active_area {
            EditableArea::Regex => {
                frame.set_cursor(
                    // Put cursor past the end of the regex
                    input_layout[0].x + self.regex.len() as u16 + 1,
                    // Move one line down, from the border to the regex input
                    input_layout[0].y + 1,
                );
            }
            EditableArea::Replace => {
                frame.set_cursor(
                    // Put cursor past the end of the replacement
                    input_layout[1].x + self.replacement.len() as u16 + 1,
                    // Move one line down, from the border to the replacement input
                    input_layout[1].y + 1,
                );
            }
        }

        let re = Regex::new(&self.regex).ok();

        let items: Vec<ListItem> = self
            .source_files
            .clone()
            .into_iter()
            .map(path_utils::split_path)
            .filter_map(|(parent, name)| name.map(|name| (parent, name)))
            .map(|(parent, name)| {
                let dir_style = Style::default().add_modifier(Modifier::BOLD);
                let src_name_style = Style::default().fg(Color::Red);
                let dst_name_style = Style::default().fg(Color::Green);

                let dir_str = parent.to_str().unwrap().to_owned() + "/";

                // TODO: refactor with helper function returning enum of is_match/replace state
                // and handle it with match {} block
                if let Some(re) = re.as_ref() {
                    if re.is_match(&name) {
                        let dst_name: String = re.replace(&name, &self.replacement).into();

                        if dst_name != name {
                            Spans::from(vec![
                                Span::styled(dir_str, dir_style),
                                Span::styled(name, src_name_style),
                                Span::raw("->"),
                                Span::styled(dst_name, dst_name_style),
                            ])
                        } else {
                            Spans::from(vec![
                                Span::styled(dir_str, dir_style),
                                Span::styled(name, dst_name_style),
                            ])
                        }
                    } else {
                        Spans::from(vec![Span::styled(dir_str, dir_style), Span::from(name)])
                    }
                } else {
                    Spans::from(vec![Span::styled(dir_str, dir_style), Span::from(name)])
                }
            })
            .map(ListItem::new)
            .collect();

        let files_view =
            List::new(items).block(Block::default().title("Files").borders(Borders::ALL));
        frame.render_widget(files_view, editor_layout[1]);

        let side_pane = Block::default().title("Help").borders(Borders::ALL);
        frame.render_widget(side_pane, main_layout[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod editable_area {
        use super::*;

        #[rstest]
        #[case(EditableArea::Regex, EditableArea::Replace)]
        #[case(EditableArea::Replace, EditableArea::Regex)]
        fn next(#[case] current_area: EditableArea, #[case] expected_next_area: EditableArea) {
            let next_area = current_area.next();
            assert_eq!(next_area, expected_next_area);
        }

        #[rstest]
        #[case(EditableArea::Regex, EditableArea::Replace)]
        #[case(EditableArea::Replace, EditableArea::Regex)]
        fn prev(#[case] current_area: EditableArea, #[case] expected_next_area: EditableArea) {
            let next_area = current_area.prev();
            assert_eq!(next_area, expected_next_area);
        }
    }
}
