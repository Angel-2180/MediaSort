use crate::event::handle_events;
use crate::tui::ui;
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{self},
        Color, Stylize,
    },
    symbols,
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
        format!(
            "   {}: {}   ",
            self.to_string().chars().next().unwrap().to_lowercase(),
            self
        )
        .fg(tailwind::ZINC.c700)
        .bg(tailwind::AMBER.c600)
        .into()
    }

    fn render_media_sort_tab(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(3), // Input area
            Constraint::Length(3), // Output area
            Constraint::Min(0),    // Process area
        ]);
        let chunks = layout.split(area);

        // Render input block
        let input_block = Block::default()
            .title("Input")
            .borders(ratatui::widgets::Borders::ALL);
        input_block.render(chunks[0], buf);

        // Render output block
        let output_block = Block::default()
            .title("Output")
            .borders(ratatui::widgets::Borders::ALL);
        output_block.render(chunks[1], buf);

        // Render process block
        let process_block = Block::default()
            .title("Process")
            .borders(ratatui::widgets::Borders::ALL);
        process_block.render(chunks[2], buf);
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
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(tailwind::ZINC.c600)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        Line::raw("Media Sort")
            .fg(tailwind::SLATE.c700)
            .bg(tailwind::ZINC.c600)
            .centered()
            .render(title_area, buf);

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
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = (Color::default(), tailwind::ZINC.c600);
        let selected_tab_index = self.current_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
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
