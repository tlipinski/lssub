use ratatui::text::Line;

pub struct Pad<'a>(pub &'a str);

impl <'a> Into<Line<'a>> for Pad<'a> {
    fn into(self) -> Line<'a> {
        Line::from(format!(" {} ", self.0))
    }
}