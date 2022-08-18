use crate::path_utils;

use std::{
    fmt::{format, Display},
    path::PathBuf,
    str::FromStr,
    thread,
    time::Duration,
};

use lazy_static::lazy_static;
use num_derive::{FromPrimitive, ToPrimitive};
use regex::Regex;
use termion::{event::Key, input::TermRead};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use variant_count::VariantCount;

bitflags::bitflags! {
    struct MatchFlags : u8 {
        const NO_FLAGS = 0;
        const GLOBAL = 1;
        const ICASE = 2;
    }
}

impl Display for MatchFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.contains(MatchFlags::GLOBAL) {
            f.write_str("g")?;
        }
        if self.contains(MatchFlags::ICASE) {
            f.write_str("i")?;
        }

        Ok(())
    }
}

lazy_static! {
    static ref FLAGS_REGEX: Regex = Regex::new("[gi]{0,2}").unwrap();
}

impl FromStr for MatchFlags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if FLAGS_REGEX.is_match(s) {
            let mut flags = MatchFlags::NO_FLAGS;
            if s.contains("g") {
                flags |= MatchFlags::GLOBAL;
            }
            if s.contains("i") {
                flags |= MatchFlags::ICASE;
            }
            Ok(flags)
        } else {
            Err(format!("invalid regex flags: '{}'", s))
        }
    }
}

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

#[derive(Debug, PartialEq)]
enum ReplacementResult {
    InvalidRegex,
    NoMatch,
    Unchanged,
    Replaced(String),
}

fn compose_regex(regex_str: &str, flags: MatchFlags) -> Option<Regex> {
    let flags_str = if flags.contains(MatchFlags::ICASE) {
        "i"
    } else {
        ""
    };
    let composed_str = format!("(?{}:{})", flags_str, regex_str);

    Regex::new(&composed_str).ok()
}

fn try_replace(
    text: &str,
    regex: &Option<Regex>,
    replacement: &str,
    global: bool,
) -> ReplacementResult {
    if let Some(regex) = regex.as_ref() {
        if !regex.is_match(text) {
            ReplacementResult::NoMatch
        } else {
            let replaced = if global {
                regex.replace_all(text, replacement)
            } else {
                regex.replace(text, replacement)
            };

            if replaced == text {
                ReplacementResult::Unchanged
            } else {
                ReplacementResult::Replaced(replaced.into())
            }
        }
    } else {
        ReplacementResult::InvalidRegex
    }
}

pub enum AppResult {
    MoveFiles(Vec<(PathBuf, PathBuf)>),
    Exit,
}

pub struct App {
    /// current value of the regex input box
    regex: String,
    /// current value of the replacement string box
    replacement: String,
    /// match flags
    flags: MatchFlags,
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
            flags: MatchFlags::NO_FLAGS,
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

    fn is_global(&self) -> bool {
        self.flags.contains(MatchFlags::GLOBAL)
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<AppResult> {
        let mut keys_iter = termion::async_stdin().keys();

        loop {
            terminal.draw(|f| self.ui(f))?;

            let edited_string = match self.active_area {
                EditableArea::Regex => &mut self.regex,
                EditableArea::Replace => &mut self.replacement,
            };

            if let Some(Ok(key)) = keys_iter.next() {
                match key {
                    Key::Ctrl('c') => {
                        return Ok(AppResult::Exit);
                    }
                    Key::Char('\t') => {
                        self.active_area = self.active_area.next();
                    }
                    Key::BackTab => {
                        self.active_area = self.active_area.prev();
                    }
                    Key::Backspace => {
                        edited_string.pop();
                    }
                    Key::Ctrl('g') => {
                        self.flags ^= MatchFlags::GLOBAL;
                    }
                    Key::Ctrl('r') => {
                        self.flags ^= MatchFlags::ICASE;
                    }
                    Key::Char('\n') => {
                        let re = Regex::new(&self.regex).ok();

                        let move_pairs: Vec<(PathBuf, PathBuf)> = self
                            .source_files
                            .clone()
                            .into_iter()
                            .filter_map(path_utils::split_path)
                            .filter_map(|(parent, name)| {
                                match try_replace(&name, &re, &self.replacement, self.is_global()) {
                                    ReplacementResult::Replaced(dst_name) => {
                                        let src_path = parent.join(name);
                                        let dst_path = parent.join(dst_name);

                                        Some((src_path, dst_path))
                                    }
                                    _ => None,
                                }
                            })
                            .collect();

                        return Ok(AppResult::MoveFiles(move_pairs));
                    }
                    Key::Char(ch) => {
                        edited_string.push(ch);
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn ui<B: Backend>(&self, frame: &mut Frame<B>) {
        let re = compose_regex(&self.regex, self.flags);

        // editor and help areas
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50), Constraint::Max(25)])
            .split(frame.size());

        // editor area: regex, replacement, flags, files list
        let editor_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .split(main_layout[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }));

        // regex and replacement inputs, flags
        let top_row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(7)])
            .split(editor_layout[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }));

        // regex and replacement inputs
        let input_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(top_row_layout[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }));

        let regex_input = Paragraph::new(self.regex.as_ref())
            .style(if re.as_ref().is_some() {
                Style::default()
            } else {
                Style::default().fg(Color::Red)
            })
            .block(Block::default().title("Regex").borders(Borders::ALL));
        frame.render_widget(regex_input, input_layout[0]);

        let replace_input = Paragraph::new(self.replacement.as_ref())
            .block(Block::default().title("Replacement").borders(Borders::ALL));
        frame.render_widget(replace_input, input_layout[1]);

        let flags_view = Paragraph::new(self.flags.to_string())
            .block(Block::default().title("Flags").borders(Borders::ALL));
        frame.render_widget(flags_view, top_row_layout[1]);

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

        let files_list: Vec<ListItem> = self
            .source_files
            .clone()
            .into_iter()
            .filter_map(path_utils::split_path)
            .map(|(parent, name)| {
                let dir_style = Style::default().add_modifier(Modifier::BOLD);
                let src_name_style = Style::default().fg(Color::Red);
                let dst_name_style = Style::default().fg(Color::Green);

                let dir_str = parent.to_str().unwrap().to_owned() + "/";

                match try_replace(&name, &re, &self.replacement, self.is_global()) {
                    ReplacementResult::Replaced(dst_name) => Spans::from(vec![
                        Span::styled(dir_str, dir_style),
                        Span::styled(name, src_name_style),
                        Span::raw("->"),
                        Span::styled(dst_name, dst_name_style),
                    ]),
                    _ => Spans::from(vec![Span::styled(dir_str, dir_style), Span::from(name)]),
                }
            })
            .map(ListItem::new)
            .collect();

        let files_view =
            List::new(files_list).block(Block::default().title("Files").borders(Borders::ALL));
        frame.render_widget(files_view, editor_layout[1]);

        let help_list: Vec<Spans> = vec![
            ("Tab", "switch between regex and replacement areas"),
            ("Enter", "execute renaming"),
            ("Ctrl-g", "'global' flag"),
            ("Ctrl-r", "'icase' flag"),
            ("Ctrl-c", "exit"),
        ]
        .into_iter()
        .map(|(key, descr)| {
            Spans::from(vec![
                Span::styled(key, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::raw(descr),
            ])
        })
        .collect();
        let help_view = Paragraph::new(help_list)
            .wrap(Wrap { trim: false })
            .block(Block::default().title("Help").borders(Borders::ALL));
        frame.render_widget(help_view, main_layout[1]);
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

    #[rstest]
    #[case("a", None, "b", false, ReplacementResult::InvalidRegex)]
    #[case("abc", Regex::new("bc").ok(), "bc", false, ReplacementResult::Unchanged)]
    #[case("abc", Regex::new("b").ok(), "f", false, ReplacementResult::Replaced("afc".into()))]
    #[case("abc", Regex::new("(ab)(.*)").ok(), "$2$1", false, ReplacementResult::Replaced("cab".into()))]
    fn try_replace_works(
        #[case] text: &str,
        #[case] regex: Option<Regex>,
        #[case] replacement: &str,
        #[case] global: bool,
        #[case] expected_result: ReplacementResult,
    ) {
        let replacement_result = try_replace(text, &regex, replacement, global);
        assert_eq!(replacement_result, expected_result);
    }
}
