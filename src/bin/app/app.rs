//! Settings app state, messages and update logic.

use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::{Action, ApplicationExt, Element};
use cosmic_nightlight::schedule::ClockTime;
use cosmic_nightlight::{Config, Mode, Monitor, backend};
use std::time::Duration;

const APP_ID: &str = "dev.bear.CosmicNightlight";
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const PREVIEW_SECONDS: u32 = 4;

pub struct App {
    core: Core,
    pub(crate) config: Config,
    /// Level last applied to the monitors.
    pub(crate) level: u32,
    /// `None` until the first systemctl query returns.
    pub(crate) timer_enabled: Option<bool>,
    pub(crate) detecting: bool,
    pub(crate) previewing: bool,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Refresh,
    SetMode(Mode),
    SetWarmStart(ClockTime),
    SetCoolStart(ClockTime),
    SetFadeMinutes(u32),
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

impl App {
    fn refresh_level(&mut self) {
        self.level = backend::applied_level().unwrap_or_else(|| self.config.target_level());
    }

    /// Save the config and ask the CLI to apply it.
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

    fn report(&mut self, result: Result<(), String>) {
        if let Err(e) = result {
            self.error = Some(e);
        }
    }
}

/// Run a future and wrap its output in an app message.
fn perform<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static,
    to_message: impl FnOnce(T) -> Message + Send + 'static,
) -> Task<Message> {
    Task::perform(future, move |out| Action::App(to_message(out)))
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
        let mut app = App {
            core,
            config: Config::load(),
            level: 0,
            timer_enabled: None,
            detecting: false,
            previewing: false,
            error: None,
        };
        app.refresh_level();
        app.set_header_title("Night Light".into());

        let mut tasks = vec![perform(backend::timer_enabled(), Message::TimerStatus)];
        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title("Night Light".into(), id));
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
            Message::Refresh => {
                // Pick up mode changes made from the applet or CLI.
                self.config.mode = Config::load().mode;
                self.refresh_level();
            }
            Message::SetMode(mode) => {
                self.config.mode = mode;
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
            Message::SetStrength(s) => self.config.strength = s,
            Message::CommitStrength => {
                return Task::batch([self.persist(), self.preview()]);
            }
            Message::Preview => return self.preview(),
            Message::PreviewDone(result) => {
                self.previewing = false;
                self.report(result);
                self.refresh_level();
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
                self.refresh_level();
            }
            Message::DismissError => self.error = None,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        crate::view::view(self)
    }
}
