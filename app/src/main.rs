//! The desktop window: a row of four tab buttons over the active page's placeholder.

use flat3d_app::{update, App, Message, Tab};
use iced::widget::{button, column, row, text};
use iced::Element;

fn main() -> iced::Result {
    iced::application(App::default, update, view)
        .title("Flat to 3D")
        .run()
}

fn view(app: &App) -> Element<'_, Message> {
    let tabs = row(Tab::ALL.into_iter().map(|tab| {
        button(text(tab.title()))
            .on_press(Message::Select(tab))
            .into()
    }))
    .spacing(8);

    column![tabs, text(app.active.placeholder())]
        .spacing(16)
        .padding(16)
        .into()
}
