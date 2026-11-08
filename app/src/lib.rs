//! The desktop app's state: the four pages (Setup, Headset, Games, Logs), the messages
//! that change them and the pure `update`. No iced here, so the tests run without a window;
//! the widgets live in `main.rs`.

/// The four page tabs, in the order the window shows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Setup,
    Headset,
    Games,
    Logs,
}

impl Tab {
    /// Every tab, left to right.
    pub const ALL: [Tab; 4] = [Tab::Setup, Tab::Headset, Tab::Games, Tab::Logs];

    /// The label on the tab button.
    pub fn title(self) -> &'static str {
        match self {
            Tab::Setup => "Setup",
            Tab::Headset => "Headset",
            Tab::Games => "Games",
            Tab::Logs => "Logs",
        }
    }

    /// The text a page shows until its own issue fills it in.
    pub fn placeholder(self) -> &'static str {
        match self {
            Tab::Setup => "Setup: placeholder. The page is not built yet.",
            Tab::Headset => "Headset: placeholder. The page is not built yet.",
            Tab::Games => "Games: placeholder. The page is not built yet.",
            Tab::Logs => "Logs: placeholder. The page is not built yet.",
        }
    }
}

/// What the window holds: the tab on screen.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct App {
    pub active: Tab,
}

/// What the user can do. More variants come with the pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// A tab button was pressed.
    Select(Tab),
}

/// Apply a message to the state.
pub fn update(app: &mut App, message: Message) {
    match message {
        Message::Select(tab) => app.active = tab,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_opens_on_setup() {
        assert_eq!(App::default().active, Tab::Setup);
    }

    #[test]
    fn the_four_tabs_are_in_order() {
        let titles: Vec<&str> = Tab::ALL.iter().map(|t| t.title()).collect();
        assert_eq!(titles, ["Setup", "Headset", "Games", "Logs"]);
    }

    #[test]
    fn every_tab_has_placeholder_text() {
        for tab in Tab::ALL {
            assert!(!tab.placeholder().is_empty(), "{tab:?}");
        }
    }

    #[test]
    fn select_moves_to_the_tab() {
        let mut app = App::default();
        update(&mut app, Message::Select(Tab::Logs));
        assert_eq!(app.active, Tab::Logs);
        update(&mut app, Message::Select(Tab::Headset));
        assert_eq!(app.active, Tab::Headset);
    }
}
