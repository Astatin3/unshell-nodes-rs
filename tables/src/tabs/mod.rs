mod env_vars;
mod tab1;
mod tab2;
mod tab3;

use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{Block, Borders, Padding, Tabs},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub struct TabController {
    current_index: usize,
    tabs: Vec<Tab>,
}

impl Default for TabController {
    fn default() -> Self {
        Self {
            current_index: 0,
            tabs: vec![Tab::Env(env_vars::State::default()), Tab::Tab2, Tab::Tab3],
        }
    }
}

impl TabController {
    pub fn previous(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    pub fn next(&mut self) {
        if self.current_index < self.tabs.len() - 1 {
            self.current_index += 1;
        }
    }

    fn tab(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_index]
    }

    pub fn render_tab_titles(&mut self, area: Rect, frame: &mut Frame) {
        let titles = Tab::iter().map(Tab::title);
        let highlight_style = (Color::default(), self.tab().color().c700);

        frame.render_widget(
            Tabs::new(titles)
                .highlight_style(highlight_style)
                .select(self.current_index)
                .padding("", "")
                .divider(" "),
            area,
        );
    }

    pub fn render(&mut self, area: Rect, frame: &mut Frame) {
        self.tab().render(area, frame)
    }

    pub fn key(&mut self, keycode: KeyCode) -> bool {
        self.tab().key(keycode)
    }
}

#[derive(Clone, Display, FromRepr, EnumIter)]
pub enum Tab {
    #[strum(to_string = "Env")]
    Env(env_vars::State),
    #[strum(to_string = "Tab 2")]
    Tab2,
    #[strum(to_string = "Tab 3")]
    Tab3,
}

impl Default for Tab {
    fn default() -> Self {
        return Self::Env(env_vars::State::default());
    }
}

impl Tab {
    /// Return tab's name as a styled `Line`
    pub fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.color().c900)
            .into()
    }

    /// A block surrounding the tab's content
    fn block(&self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.color().c700)
            .borders(Borders::TOP)
    }

    pub const fn color(&self) -> tailwind::Palette {
        match self {
            Self::Env(_) => env_vars::color(),
            Self::Tab2 => tab2::color(),
            Self::Tab3 => tab3::color(),
            // _ => tab3::color(),
        }
    }

    pub fn render(&mut self, area: Rect, frame: &mut Frame) {
        let block = self.block();
        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        match self {
            Self::Env(state) => state.render(inner_area, frame),
            Self::Tab2 => tab2::render(inner_area, frame),
            Self::Tab3 => tab3::render(inner_area, frame),
            // _ => tab3::render(inner_area, frame),
        }
    }

    pub fn key(&mut self, keycode: KeyCode) -> bool {
        match self {
            Self::Env(state) => state.key(keycode),
            _ => false,
        }
    }
}
