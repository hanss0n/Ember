//! Settings page layout, one function per section.

use crate::app::{App, Message};
use crate::time_picker::time_picker;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, settings};
use cosmic::Element;
use cosmic_nightlight::{Mode, status};

const MAX_WIDTH: f32 = 720.0;
const FADE_STEP: u32 = 10;
const FADE_MAX: u32 = 180;

pub fn view(app: &App) -> Element<'_, Message> {
    let mut sections: Vec<Element<'_, Message>> = Vec::with_capacity(7);
    if let Some(error) = &app.error {
        sections.push(error_banner(error));
    }
    sections.extend([
        status_card(app),
        mode_section(app),
        schedule_section(app),
        warmth_section(app),
        monitors_section(app),
        background_section(app),
    ]);

    let page = settings::view_column(sections).max_width(MAX_WIDTH);
    widget::scrollable(
        widget::container(page).width(Length::Fill).center_x(Length::Fill).padding([0, 16, 16, 16]),
    )
    .into()
}

fn error_banner(error: &str) -> Element<'_, Message> {
    widget::warning(error).on_close(Message::DismissError).into()
}

fn status_card(app: &App) -> Element<'_, Message> {
    let content = widget::row::with_capacity(2)
        .spacing(16)
        .align_y(Alignment::Center)
        .push(widget::icon::from_name(status::icon_for(app.level)).size(40))
        .push(
            widget::column::with_capacity(2)
                .push(widget::text::title4(status::describe(&app.config, app.level)))
                .push(widget::text::caption(
                    "Warms your monitors' colors using their built-in DDC/CI controls.",
                )),
        );
    widget::container(content)
        .padding(16)
        .width(Length::Fill)
        .class(cosmic::theme::Container::Card)
        .into()
}

fn mode_section(app: &App) -> Element<'_, Message> {
    let selected = Some(app.config.mode);
    Mode::ALL
        .into_iter()
        .fold(settings::section().title("Mode"), |section, mode| {
            let item = settings::item::builder(mode.label()).description(mode_description(mode));
            section.add(item.radio(mode, selected, Message::SetMode))
        })
        .into()
}

fn mode_description(mode: Mode) -> &'static str {
    match mode {
        Mode::Auto => "Follow the schedule below",
        Mode::Night => "Keep the screen warm until switched back",
        Mode::Day => "Keep normal colors until switched back",
    }
}

fn schedule_section(app: &App) -> Element<'_, Message> {
    let fade = app.config.fade_minutes;
    settings::section()
        .title("Schedule")
        .add(
            settings::item::builder("Warm up at")
                .description("Evening time when the screen starts getting warmer")
                .control(time_picker(app.config.warm_time(), Message::SetWarmStart)),
        )
        .add(
            settings::item::builder("Cool down at")
                .description("Morning time when normal colors start returning")
                .control(time_picker(app.config.cool_time(), Message::SetCoolStart)),
        )
        .add(
            settings::item::builder("Fade duration")
                .description("How long each transition takes")
                .control(widget::spin_button(
                    fade_label(fade),
                    fade,
                    FADE_STEP,
                    0,
                    FADE_MAX,
                    Message::SetFadeMinutes,
                )),
        )
        .into()
}

fn fade_label(minutes: u32) -> String {
    match minutes {
        0 => "Instant".into(),
        m => format!("{m} min"),
    }
}

fn warmth_section(app: &App) -> Element<'_, Message> {
    let strength = app.config.strength;
    let slider = widget::row::with_capacity(2)
        .spacing(12)
        .align_y(Alignment::Center)
        .push(
            widget::slider(0..=100, strength, Message::SetStrength)
                .step(5u32)
                .on_release(Message::CommitStrength)
                .width(Length::Fixed(220.0)),
        )
        .push(widget::text::body(format!("{strength}%")).width(Length::Fixed(40.0)));

    let preview = widget::button::standard(if app.previewing { "Previewing…" } else { "Preview" })
        .on_press_maybe((!app.previewing).then_some(Message::Preview));

    settings::section()
        .title("Warmth")
        .add(
            settings::item::builder("Strength")
                .description("How warm the screen gets at night")
                .control(slider),
        )
        .add(
            settings::item::builder("Preview full warmth")
                .description("Shows night colors for a few seconds")
                .control(preview),
        )
        .into()
}

fn monitors_section(app: &App) -> Element<'_, Message> {
    let detect = widget::button::text(if app.detecting { "Detecting…" } else { "Detect" })
        .on_press_maybe((!app.detecting).then_some(Message::DetectMonitors));
    let header = widget::row::with_capacity(2)
        .align_y(Alignment::Center)
        .push(widget::text::heading("Monitors").width(Length::Fill))
        .push(detect);

    let section = settings::section().header(header);
    if app.config.monitors.is_empty() {
        return section
            .add(settings::item::builder("No DDC/CI monitors found").control(widget::Space::new()))
            .into();
    }

    app.config
        .monitors
        .iter()
        .enumerate()
        .fold(section, |section, (index, monitor)| {
            section.add(
                settings::item::builder(monitor.name.clone())
                    .description(format!("I²C bus {}", monitor.bus))
                    .toggler(monitor.enabled, move |on| Message::ToggleMonitor(index, on)),
            )
        })
        .into()
}

fn background_section(app: &App) -> Element<'_, Message> {
    let item = settings::item::builder("Run automatically")
        .description("Checks the schedule every few minutes, even when this window is closed");
    let toggler = match app.timer_enabled {
        Some(enabled) => item.toggler(enabled, Message::SetTimer),
        None => item.toggler_maybe(false, None::<fn(bool) -> Message>),
    };
    settings::section().title("Background").add(toggler).into()
}
