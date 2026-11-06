//! Settings app state, messages and update logic.

use crate::location::LocationInput;
use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::{Action, ApplicationExt, Element};
use cosmic_ember::clock::ClockTime;
use cosmic_ember::{Config, Mode, Monitor, Schedule, Status, backend};
use std::time::Duration;

const APP_ID: &str = "dev.bear.CosmicEmber";
const TITLE: &str = "Ember";
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const PREVIEW_SECONDS: u32 = 4;

pub struct App {
    core: Core,
    /// Settings being edited; saved on every change.
    pub(crate) config: Config,
    /// Latest state reported by the CLI.
    pub(crate) status: Status,
    pub(crate) location: LocationInput,
    /// `None` until the first systemctl query returns.
    pub(crate) timer_enabled: Option<bool>,
    pub(crate) detecting: bool,
    pub(crate) previewing: bool,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Refresh,
    StatusLoaded(Result<Status, String>),
    SetMode(Mode),
    SetSchedule(Schedule),
    SetWarmStart(ClockTime),
    SetCoolStart(ClockTime),
    SetFadeMinutes(u32),
    LatitudeInput(String),
    LongitudeInput(String),
    UseTimezoneLocation,
    /// Slider is being dragged.
    SetStrength(u32),
    /// Slider was released: save and preview.
    CommitStrength,
    Preview,
    PreviewDone(Result<(), String>),
    ToggleMonitor(usize, bool),
    DetectMonitors,
    MonitorsDetected(Result<Vec<Monitor>, String>),
    SetTimer(bool),
    TimerStatus(bool),
    TimerChanged(Result<(), String>),
    Applied(Result<(), String>),
    DismissError,
}

/// Run a future and wrap its output in an app message.
fn perform<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static,
    to_message: impl FnOnce(T) -> Message + Send + 'static,
) -> Task<Message> {
    Task::perform(future, move |out| Action::App(to_message(out)))
}

impl App {
    fn load_status() -> Task<Message> {
        perform(backend::status(), Message::StatusLoaded)
    }

    /// Save the config, apply it, then refresh the status.
    fn persist(&mut self) -> Task<Message> {
        if let Err(e) = self.config.save() {
            self.error = Some(format!("Could not save settings: {e}"));
            return Task::none();
        }
        perform(backend::apply(), Message::Applied)
    }

    fn preview(&mut self) -> Task<Message> {
        if self.previewing {
            return Task::none();
        }
        self.previewing = true;
        perform(backend::preview(PREVIEW_SECONDS), Message::PreviewDone)
    }

    /// Save the location once both fields hold valid coordinates.
    fn commit_location(&mut self) -> Task<Message> {
        match self.location.parse() {
            Some(coords) => {
                self.config.set_location(Some(coords));
                self.persist()
            }
            None => Task::none(),
        }
    }

    fn report(&mut self, result: Result<(), String>) {
        if let Err(e) = result {
            self.error = Some(e);
        }
    }
}

impl cosmic::Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let config = Config::load();
        let mut app = App {
            core,
            location: LocationInput::from_config(&config),
            config,
            status: Status::default(),
            timer_enabled: None,
            detecting: false,
            previewing: false,
            error: None,
        };
        app.set_header_title(TITLE.into());

        let mut tasks = vec![
            Self::load_status(),
            perform(backend::timer_enabled(), Message::TimerStatus),
        ];
        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(TITLE.into(), id));
        }
        if app.config.monitors.is_empty() {
            tasks.push(cosmic::task::message(Action::App(Message::DetectMonitors)));
        }
        (app, Task::batch(tasks))
    }

    fn subscription(&self) -> Subscription<Message> {
        cosmic::iced::time::every(REFRESH_INTERVAL).map(|_| Message::Refresh)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => return Self::load_status(),
            Message::StatusLoaded(Ok(status)) => {
                // Pick up mode changes made from the applet or CLI.
                self.config.mode = status.mode;
                self.status = status;
            }
            Message::StatusLoaded(Err(e)) => self.error = Some(e),
            Message::SetMode(mode) => {
                self.config.mode = mode;
                return self.persist();
            }
            Message::SetSchedule(schedule) => {
                self.config.schedule = schedule;
                return self.persist();
            }
            Message::SetWarmStart(t) => {
                self.config.set_warm_time(t);
                return self.persist();
            }
            Message::SetCoolStart(t) => {
                self.config.set_cool_time(t);
                return self.persist();
            }
            Message::SetFadeMinutes(m) => {
                self.config.fade_minutes = m;
                return self.persist();
            }
            Message::LatitudeInput(text) => {
                self.location.latitude = text;
                return self.commit_location();
            }
            Message::LongitudeInput(text) => {
                self.location.longitude = text;
                return self.commit_location();
            }
            Message::UseTimezoneLocation => {
                self.location = LocationInput::default();
                self.config.set_location(None);
                return self.persist();
            }
            Message::SetStrength(s) => self.config.strength = s,
            Message::CommitStrength => return Task::batch([self.persist(), self.preview()]),
            Message::Preview => return self.preview(),
            Message::PreviewDone(result) => {
                self.previewing = false;
                self.report(result);
                return Self::load_status();
            }
            Message::ToggleMonitor(index, enabled) => {
                if let Some(monitor) = self.config.monitors.get_mut(index) {
                    monitor.enabled = enabled;
                    return self.persist();
                }
            }
            Message::DetectMonitors => {
                self.detecting = true;
                return perform(backend::detect_monitors(), Message::MonitorsDetected);
            }
            Message::MonitorsDetected(result) => {
                self.detecting = false;
                match result {
                    Ok(monitors) => self.config.monitors = monitors,
                    Err(e) => self.error = Some(format!("Monitor detection failed: {e}")),
                }
            }
            Message::SetTimer(enabled) => {
                self.timer_enabled = Some(enabled);
                return perform(backend::set_timer(enabled), Message::TimerChanged);
            }
            Message::TimerChanged(result) => {
                self.report(result);
                return perform(backend::timer_enabled(), Message::TimerStatus);
            }
            Message::TimerStatus(enabled) => self.timer_enabled = Some(enabled),
            Message::Applied(result) => {
                self.report(result);
                return Self::load_status();
            }
            Message::DismissError => self.error = None,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        crate::view::view(self)
    }
}
