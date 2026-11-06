//! COSMIC panel applet: sun/moon icon with a quick mode switcher.

mod applet;
mod popup;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<applet::Applet>(())
}
