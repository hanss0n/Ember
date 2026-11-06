//! Ember settings app for COSMIC.

mod app;
mod location;
mod time_picker;
mod view;

use cosmic::app::Settings;
use cosmic::iced::Size;

fn main() -> cosmic::iced::Result {
    let settings = Settings::default()
        .size(Size::new(620.0, 820.0))
        .size_limits(cosmic::iced::Limits::NONE.min_width(420.0).min_height(400.0));
    cosmic::app::run::<app::App>(settings, ())
}
