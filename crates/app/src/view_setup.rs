use crate::setup_page::{self, Msg, SetupKind, SetupState};
use flat3d_inject::api::LinkSnapshot;
use iced::widget::{button, column, text};
use iced::Element;

pub fn view<'a>(s: &'a SetupState, snap: Option<&'a LinkSnapshot>) -> Element<'a, Msg> {
    let mut content = column![].spacing(10);
    for step in setup_page::steps(s.packages.as_ref(), snap) {
        let marker = if step.done { "✓" } else { "○" };
        let line = if step.done {
            format!("{marker} {}", step.title)
        } else {
            format!("{marker} {} — {}", step.title, step.hint)
        };
        content = content.push(text(line));
    }
    content = content.push(text(setup_page::adb_source_text(snap)));
    if let Some(error) = &s.error {
        content = content.push(text(error));
    }
    if s.busy {
        content = content
            .push(button("Check again"))
            .push(button("Install headset apps"))
            .push(button("Run doctor"));
    } else {
        content = content
            .push(button("Check again").on_press(Msg::RunPressed(SetupKind::Packages)))
            .push(button("Install headset apps").on_press(Msg::RunPressed(SetupKind::InstallApps)))
            .push(button("Run doctor").on_press(Msg::RunPressed(SetupKind::Doctor)));
    }
    content.into()
}
