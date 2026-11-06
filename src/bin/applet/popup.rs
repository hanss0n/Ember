//! Popup contents shown when the panel icon is clicked.

use crate::applet::{Applet, Message};
use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::widget::{self, settings};
use cosmic_ember::Mode;

pub fn view(state: &Applet) -> Element<'_, Message> {
    widget::column::with_capacity(4)
        .push(header(state))
        .push(mode_list(state))
        .push(cosmic::applet::padded_control(widget::divider::horizontal::default()))
        .push(
            cosmic::applet::menu_button(widget::text::body("Ember Settings…"))
                .on_press(Message::OpenSettings),
        )
        .padding([8, 0])
        .into()
}

fn header(state: &Applet) -> Element<'_, Message> {
    widget::row::with_capacity(2)
        .spacing(12)
        .align_y(Alignment::Center)
        .push(widget::icon::from_name(state.status.icon()).size(24))
        .push(
            widget::column::with_capacity(2)
                .push(widget::text::heading("Ember"))
                .push(widget::text::caption(state.status.headline())),
        )
        .padding([8, 16])
        .into()
}

fn mode_list(state: &Applet) -> Element<'_, Message> {
    let selected = Some(state.status.mode);
    let schedule = state.status.schedule_summary();

    let section = Mode::ALL.into_iter().fold(settings::section(), |section, mode| {
        let item = settings::item::builder(mode.label());
        let item = if mode == Mode::Auto { item.description(schedule.clone()) } else { item };
        section.add(item.radio(mode, selected, Message::SetMode))
    });

    widget::container(section).padding([0, 8]).into()
}
