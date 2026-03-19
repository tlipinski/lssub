use crate::ui::actions::Action;
use crate::ui::actions::Action::LanguagesUpdated;
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use Action::Init;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Line;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_checkbox::Checkbox;

const GRID_CELL_WIDTH: u16 = 26;
const ALL_LANGUAGES: [(&str, &str); 100] = [
    ("English", "en"),
    ("Spanish", "es"),
    ("French", "fr"),
    ("German", "de"),
    ("Italian", "it"),
    ("Portuguese", "pt"),
    ("Russian", "ru"),
    ("Chinese", "zh"),
    ("Japanese", "ja"),
    ("Korean", "ko"),
    ("Arabic", "ar"),
    ("Hindi", "hi"),
    ("Bengali", "bn"),
    ("Urdu", "ur"),
    ("Turkish", "tr"),
    ("Polish", "pl"),
    ("Dutch", "nl"),
    ("Swedish", "sv"),
    ("Norwegian", "no"),
    ("Danish", "da"),
    ("Finnish", "fi"),
    ("Greek", "el"),
    ("Hebrew", "he"),
    ("Thai", "th"),
    ("Vietnamese", "vi"),
    ("Indonesian", "id"),
    ("Malay", "ms"),
    ("Filipino", "tl"),
    ("Czech", "cs"),
    ("Slovak", "sk"),
    ("Hungarian", "hu"),
    ("Romanian", "ro"),
    ("Bulgarian", "bg"),
    ("Croatian", "hr"),
    ("Serbian", "sr"),
    ("Slovenian", "sl"),
    ("Estonian", "et"),
    ("Latvian", "lv"),
    ("Lithuanian", "lt"),
    ("Ukrainian", "uk"),
    ("Belarusian", "be"),
    ("Catalan", "ca"),
    ("Galician", "gl"),
    ("Basque", "eu"),
    ("Icelandic", "is"),
    ("Irish", "ga"),
    ("Welsh", "cy"),
    ("Scottish Gaelic", "gd"),
    ("Maltese", "mt"),
    ("Albanian", "sq"),
    ("Macedonian", "mk"),
    ("Bosnian", "bs"),
    ("Faroese", "fo"),
    ("Persian", "fa"),
    ("Pashto", "ps"),
    ("Kurdish", "ku"),
    ("Armenian", "hy"),
    ("Georgian", "ka"),
    ("Azerbaijani", "az"),
    ("Kazakh", "kk"),
    ("Uzbek", "uz"),
    ("Turkmen", "tk"),
    ("Kyrgyz", "ky"),
    ("Tajik", "tg"),
    ("Mongolian", "mn"),
    ("Nepali", "ne"),
    ("Sinhala", "si"),
    ("Tamil", "ta"),
    ("Telugu", "te"),
    ("Kannada", "kn"),
    ("Malayalam", "ml"),
    ("Marathi", "mr"),
    ("Gujarati", "gu"),
    ("Punjabi", "pa"),
    ("Assamese", "as"),
    ("Odia", "or"),
    ("Burmese", "my"),
    ("Khmer", "km"),
    ("Lao", "lo"),
    ("Swahili", "sw"),
    ("Amharic", "am"),
    ("Somali", "so"),
    ("Hausa", "ha"),
    ("Yoruba", "yo"),
    ("Igbo", "ig"),
    ("Zulu", "zu"),
    ("Afrikaans", "af"),
    ("Xhosa", "xh"),
    ("Maori", "mi"),
    ("Samoan", "sm"),
    ("Tongan", "to"),
    ("Haitian Creole", "ht"),
    ("Latin", "la"),
    ("Esperanto", "eo"),
    ("Luxembourgish", "lb"),
    ("Frisian", "fy"),
    ("Yiddish", "yi"),
    ("Javanese", "jv"),
    ("Sundanese", "su"),
    ("Malagasy", "mg"),
];

pub struct LanguagesWidget {
    selected: Vec<bool>,
    focused_idx: usize,
    grid_columns: usize,
}

impl Component for LanguagesWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Init => Some(LanguagesUpdated(self.languages())),
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if let Event::Key(key_event) = event {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE) => Some(LanguagesUpdated(self.languages())),
                (KeyCode::Char(' '), KeyModifiers::NONE) => {
                    self.toggle_focused();
                    None
                }
                (KeyCode::Left, KeyModifiers::NONE) | (KeyCode::Char('h'), KeyModifiers::NONE) => {
                    self.move_left();
                    None
                }
                (KeyCode::Right, KeyModifiers::NONE)
                | (KeyCode::Char('l'), KeyModifiers::NONE) => {
                    self.move_right();
                    None
                }
                (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                    self.move_up();
                    None
                }
                (KeyCode::Down, KeyModifiers::NONE)
                | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                    self.move_down();
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title_pad("Languages (Space toggle, Enter apply)")
            .border_set(border::PLAIN);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let hint = Paragraph::new(Line::from(
            "Use arrow keys (or h/j/k/l) to move, Space to select/unselect.",
        ));
        let hint_height = 1u16.min(inner.height);
        frame.render_widget(
            hint,
            Rect::new(inner.x, inner.y, inner.width, hint_height),
        );

        if inner.height <= hint_height {
            return;
        }

        let grid_area = Rect::new(
            inner.x,
            inner.y + hint_height,
            inner.width,
            inner.height - hint_height,
        );

        self.grid_columns = self.columns_for_width(grid_area.width);
        let columns = self.grid_columns as u16;
        let cell_width = (grid_area.width / columns).max(1);

        for idx in 0..ALL_LANGUAGES.len() {
            let row = idx / self.grid_columns;
            let col = idx % self.grid_columns;
            let y = grid_area.y + row as u16;
            if y >= grid_area.y + grid_area.height {
                break;
            }

            let x = grid_area.x + (col as u16 * cell_width);
            let width = if col + 1 == self.grid_columns {
                grid_area.width - (cell_width * (columns - 1))
            } else {
                cell_width
            };

            let (name, code) = ALL_LANGUAGES[idx];
            let mut checkbox =
                Checkbox::new(format!("{name} ({code})"), self.selected[idx]).style(Style::default()).checked_symbol("[x]").unchecked_symbol("[ ]");

            if idx == self.focused_idx {
                checkbox = checkbox
                    .checkbox_style(
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                    .label_style(
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    );
            }

            frame.render_widget(checkbox, Rect::new(x, y, width, 1));
        }
    }
}

impl LanguagesWidget {
    pub fn new(languages: Vec<String>) -> LanguagesWidget {
        let selected = ALL_LANGUAGES
            .iter()
            .map(|(_, code)| languages.iter().any(|selected_code| selected_code == code))
            .collect();

        Self {
            selected,
            focused_idx: 0,
            grid_columns: 4,
        }
    }

    pub fn languages(&self) -> Vec<String> {
        ALL_LANGUAGES
            .iter()
            .enumerate()
            .filter_map(|(idx, (_, code))| self.selected[idx].then(|| (*code).to_string()))
            .collect()
    }

    fn columns_for_width(&self, width: u16) -> usize {
        usize::max(1, (width / GRID_CELL_WIDTH) as usize)
    }

    fn toggle_focused(&mut self) {
        if let Some(item) = self.selected.get_mut(self.focused_idx) {
            *item = !*item;
        }
    }

    fn move_left(&mut self) {
        if self.focused_idx > 0 {
            self.focused_idx -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.focused_idx + 1 < ALL_LANGUAGES.len() {
            self.focused_idx += 1;
        }
    }

    fn move_up(&mut self) {
        if self.focused_idx >= self.grid_columns {
            self.focused_idx -= self.grid_columns;
        }
    }

    fn move_down(&mut self) {
        let next = self.focused_idx + self.grid_columns;
        if next < ALL_LANGUAGES.len() {
            self.focused_idx = next;
        }
    }
}
