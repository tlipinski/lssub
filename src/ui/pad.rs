use ratatui::text::Line;
use ratatui::widgets::Block;

pub trait BlockTitlePadExt<'a> {
    fn title_pad(self, title: &'a str) -> Block<'a>;
}

impl<'a> BlockTitlePadExt<'a> for Block<'a> {
    fn title_pad(self, title: &'a str) -> Block<'a> {
        self.title(format!(" {} ", title))
    }
}
