//! Hour/minute picker built from two COSMIC spin buttons.

use cosmic::iced::Alignment;
use cosmic::widget;
use cosmic::Element;
use cosmic_ember::clock::ClockTime;

const MINUTE_STEP: u32 = 5;

/// Two spin buttons ("HH" and "MM") that emit the updated time.
pub fn time_picker<'a, M: Clone + 'static>(
    time: ClockTime,
    on_change: impl Fn(ClockTime) -> M + Clone + 'static,
) -> Element<'a, M> {
    let on_hour = on_change.clone();
    let hour = widget::spin_button(
        format!("{:02}", time.hour),
        time.hour,
        1,
        0,
        23,
        move |h| on_hour(ClockTime::new(h, time.minute)),
    );
    let minute = widget::spin_button(
        format!("{:02}", time.minute),
        time.minute,
        MINUTE_STEP,
        0,
        60 - MINUTE_STEP,
        move |m| on_change(ClockTime::new(time.hour, m)),
    );

    widget::row::with_capacity(3)
        .spacing(4)
        .align_y(Alignment::Center)
        .push(hour)
        .push(widget::text::body(":"))
        .push(minute)
        .into()
}
