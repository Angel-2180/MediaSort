use crate::event::handle_events;
use crate::tui::ui;
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::palette::tailwind::{self},
    text::Line,
    widgets::{Block, Padding, Paragraph, Tabs, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Debug, Default)]
pub struct App {
    //Tabs for the application
    state: AppState,
    //The current tab
    pub current_tab: SelectedTab,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, Debug)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = "MediaSort")]
    MediaSort,
    #[strum(to_string = "Bad Keywords")]
    BadKeywords,
    #[strum(to_string = "Flags")]
    Flags,
    #[strum(to_string = "Profiles")]
    Profiles,
}

//logic for tabs
impl App {
    pub fn run(mut self, terminal: &mut ui::Tui) -> Result<()> {
        while self.state == AppState::Running {
            terminal.draw(|frame| frame.render_widget(&self, frame.size()))?;
            handle_events(&mut self)?;
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}

//tab functions
impl SelectedTab {
    /// Get the previous tab, if there is no previous tab circle back to the last tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or_else(|| Self::Profiles)
    }

    /// Get the next tab, if there is no next tab circle back to the first tab.
    fn next(self) -> Self {
        let current_index: usize = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or_else(|| Self::MediaSort)
    }
}

//tab rendering
impl SelectedTab {
    fn title(self) -> Line<'static> {
        <Line<'static>>::from(format!(
            "   {}: {}   ",
            self.to_string().chars().next().unwrap().to_lowercase(),
            self
        ))
    }

    fn render_media_sort_tab(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Media Sort Tab")
            .block(self.block())
            .render(area, buf);
    }

    fn render_bad_keywords_tab(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Bad Keywords Tab")
            .block(self.block())
            .render(area, buf);
    }

    fn render_flags_tab(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Flags Tab")
            .block(self.block())
            .render(area, buf);
    }

    fn render_profiles_tab(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Profiles Tab")
            .block(self.block())
            .render(area, buf);
    }

    fn block(self) -> Block<'static> {
        Block::default()
            // .title(self.to_string())
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(tailwind::ZINC.c600)
            .padding(Padding::new(1, 1, 1, 1))
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::layout::Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Length(3), Min(1), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Percentage(100)]);
        let [tabs_area] = horizontal.areas(header_area);

        self.render_tabs(tabs_area, buf);
        self.current_tab.render(inner_area, buf);
        match self.current_tab {
            SelectedTab::MediaSort => {
                render_footer(footer_area, buf, Some("| enter: start program |"))
            }
            SelectedTab::BadKeywords => render_footer(
                footer_area,
                buf,
                Some("| +: add bad keyword | -: remove bad keyword |"),
            ),
            SelectedTab::Flags => render_footer(
                footer_area,
                buf,
                Some("| +: enable flag | -: disable flag |"),
            ),
            SelectedTab::Profiles => render_footer(
                footer_area,
                buf,
                Some("| c: create | d: delete | e: edit |"),
            ),
        }
    }
}

//rendering the app
impl App {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        // let highlight_style = (Color::default(), tailwind::ZINC.c600);
        let selected_tab_index = self.current_tab as usize;
        Tabs::new(SelectedTab::iter().map(SelectedTab::title))
            .block(Block::bordered().border_style(tailwind::ZINC.c600))
            // .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider("")
            .render(area, buf);
    }
}

fn render_footer(area: Rect, buf: &mut Buffer, str: Option<&str>) {
    // dependant of tabs different commands will be shown always between | commands |.
    let rendered_str = format!("↑↓ navigate {} q: quit", str.unwrap_or("|"));
    Line::raw(rendered_str).centered().render(area, buf);
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            SelectedTab::MediaSort => self.render_media_sort_tab(area, buf),
            SelectedTab::BadKeywords => self.render_bad_keywords_tab(area, buf),
            SelectedTab::Flags => self.render_flags_tab(area, buf),
            SelectedTab::Profiles => self.render_profiles_tab(area, buf),
        }
    }
}
