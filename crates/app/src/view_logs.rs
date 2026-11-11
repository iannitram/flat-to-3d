use crate::logs_page::{self, Filter, LogsState, Msg};
use iced::widget::{button, column, pick_list, row, scrollable, text};
use iced::Element;

pub fn view(s: &LogsState) -> Element<'_, Msg> {
    let lines = logs_page::visible(s)
        .into_iter()
        .fold(column![].spacing(4), |col, line| col.push(text(line)));
    let content = column![
        row![
            pick_list(
                ["All", "Errors"],
                Some(match s.filter {
                    Filter::All => "All",
                    Filter::Errors => "Errors",
                }),
                |value| Msg::SetFilter(if value == "Errors" {
                    Filter::Errors
                } else {
                    Filter::All
                }),
            ),
            button("Refresh").on_press(Msg::Refresh),
            button("Copy").on_press(Msg::CopyPressed)
        ]
        .spacing(8),
        scrollable(lines),
    ]
    .spacing(12);
    if let Some(error) = &s.error {
        content.push(text(error)).into()
    } else {
        content.into()
    }
}
