use crate::games_page::{GamesState, Msg};
use flat3d_inject::status::STATUSES;
use iced::widget::{button, column, pick_list, row, scrollable, text, toggler};
use iced::{Color, Element};

pub fn view(s: &GamesState) -> Element<'_, Msg> {
    let statuses: Vec<String> = std::iter::once("all".to_string())
        .chain(STATUSES.iter().map(|s| (*s).to_string()))
        .collect();
    let mut rows = column![].spacing(8);
    for game in &s.rows {
        let colour = game.status_colour.parse::<Color>().unwrap_or(Color::WHITE);
        let appid = game.appid;
        let mut line = row![
            text(&game.name),
            text(&game.status_label).color(colour),
            toggler(game.switch_on).on_toggle(move |on| Msg::Switch { appid, on }),
            button("Undo").on_press(Msg::UndoPressed(game.appid)),
        ]
        .spacing(12);
        if let Some(reason) = &game.flat_reason {
            line = line.push(text(reason));
        }
        rows = rows.push(line);
    }
    let mut content = column![
        pick_list(statuses, Some(s.filter.clone()), Msg::Filter),
        scrollable(rows)
    ]
    .spacing(12);
    if let Some(note) = &s.note {
        content = content.push(text(note));
    }
    if let Some(error) = &s.error {
        content = content.push(text(error));
    }
    content.into()
}
