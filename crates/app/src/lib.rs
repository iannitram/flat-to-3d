//! Desktop app state and message routing. The `view_*` modules build iced widgets; `update` stays
//! pure (no I/O) and returns `Effect`s for `engine` to perform.

pub mod engine;
pub mod games_page;
pub mod headset_page;
pub mod logs_page;
pub mod setup_page;
pub mod view_games;
pub mod view_headset;
pub mod view_logs;
pub mod view_setup;

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

/// State held by the desktop window.
#[derive(Debug, Default, Clone)]
pub struct App {
    pub active: Tab,
    pub now: u64,
    pub ticks: u64,
    pub headset: headset_page::HeadsetState,
    pub games: games_page::GamesState,
    pub setup: setup_page::SetupState,
    pub logs: logs_page::LogsState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(Tab),
    Tick(u64),
    Headset(headset_page::Msg),
    Games(games_page::Msg),
    Setup(setup_page::Msg),
    Logs(logs_page::Msg),
    SwitchSaved(Result<(), String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    ReadSnapshot,
    Request(flat3d_inject::state::Request),
    SetSetting { key: &'static str, on: bool },
    LoadRows { filter: String },
    SetSwitch { appid: u32, on: bool },
    Undo(u32),
    ReadPackages,
    RunSetup(setup_page::SetupKind),
    LoadLogs,
    Copy(String),
}

/// Update state without performing IO; effects are executed by the engine adapter.
pub fn update(app: &mut App, message: Message) -> Vec<Effect> {
    use Effect as E;
    match message {
        Message::Select(tab) => {
            app.active = tab;
            let mut effects = vec![E::ReadSnapshot];
            match tab {
                Tab::Games => effects.push(E::LoadRows {
                    filter: app.games.filter.clone(),
                }),
                Tab::Setup => effects.push(E::ReadPackages),
                Tab::Logs => effects.push(E::LoadLogs),
                Tab::Headset => {}
            }
            effects
        }
        Message::Tick(now) => {
            app.now = now;
            app.ticks += 1;
            let mut effects = vec![E::ReadSnapshot];
            if app.active == Tab::Logs {
                effects.push(E::LoadLogs);
            }
            if app.active == Tab::Setup && app.ticks.is_multiple_of(10) {
                effects.push(E::ReadPackages);
            }
            effects
        }
        Message::Headset(message) => headset_page::update(&mut app.headset, message)
            .map(|a| match a {
                headset_page::Action::Start3d => E::Request(flat3d_inject::state::Request::Start3d),
                headset_page::Action::Stop3d => E::Request(flat3d_inject::state::Request::Stop3d),
                headset_page::Action::SetSetting { key, on } => E::SetSetting { key, on },
                headset_page::Action::Refresh => E::ReadSnapshot,
            })
            .into_iter()
            .collect(),
        Message::Games(message) => games_page::update(&mut app.games, message)
            .map(|a| match a {
                games_page::Action::LoadRows { filter } => E::LoadRows { filter },
                games_page::Action::SetSwitch { appid, on } => E::SetSwitch { appid, on },
                games_page::Action::Undo(id) => E::Undo(id),
            })
            .into_iter()
            .collect(),
        Message::Setup(message) => setup_page::update(&mut app.setup, message)
            .map(|a| match a {
                setup_page::Action::RunSetup(kind) => E::RunSetup(kind),
                setup_page::Action::Refresh => E::ReadPackages,
            })
            .into_iter()
            .collect(),
        Message::Logs(message) => logs_page::update(&mut app.logs, message)
            .map(|a| match a {
                logs_page::Action::Load => E::LoadLogs,
                logs_page::Action::Copy(text) => E::Copy(text),
            })
            .into_iter()
            .collect(),
        Message::SwitchSaved(Ok(())) => vec![E::LoadRows {
            filter: app.games.filter.clone(),
        }],
        Message::SwitchSaved(Err(error)) => {
            app.games.error = Some(error);
            vec![]
        }
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
