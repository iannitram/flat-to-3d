//! The iced desktop window. All filesystem and process work stays in `engine`.
use flat3d_app::{engine, update, App, Effect, Message, Tab};
use iced::widget::{button, column, row, text};
use iced::{Element, Task};
use std::time::Duration;

fn main() -> iced::Result {
    iced::application(App::default, step, view)
        .title("Flat to 3D")
        .subscription(|_| {
            iced::time::every(Duration::from_secs(1))
                .map(|_| Message::Tick(flat3d_inject::sys::unix_now()))
        })
        .run()
}

fn step(app: &mut App, message: Message) -> Task<Message> {
    Task::batch(update(app, message).into_iter().map(run_effect))
}

fn run_effect(effect: Effect) -> Task<Message> {
    match effect {
        Effect::Copy(text) => iced::clipboard::write(text),
        effect => {
            let failed = engine::worker_stopped(&effect);
            Task::perform(
                engine::off_thread(move || engine::perform(effect)),
                move |result| result.unwrap_or(failed),
            )
        }
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let tabs = row(Tab::ALL.into_iter().map(|tab| {
        button(text(tab.title()))
            .on_press(Message::Select(tab))
            .into()
    }))
    .spacing(8);
    let page = match app.active {
        Tab::Setup => flat3d_app::view_setup::view(&app.setup, app.headset.snapshot.as_ref())
            .map(Message::Setup),
        Tab::Headset => flat3d_app::view_headset::view(&app.headset, app.now).map(Message::Headset),
        Tab::Games => flat3d_app::view_games::view(&app.games).map(Message::Games),
        Tab::Logs => flat3d_app::view_logs::view(&app.logs).map(Message::Logs),
    };
    column![tabs, page].spacing(16).padding(16).into()
}
