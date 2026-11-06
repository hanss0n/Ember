//! Applet state, messages and the panel button.

use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::{Rectangle, Subscription, Vector};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::{Action, Element, widget};
use cosmic_ember::{Mode, Status, backend};
use std::time::Duration;

const APP_ID: &str = "dev.bear.CosmicAppletEmber";
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

pub struct Applet {
    core: Core,
    popup: Option<Id>,
    pub(crate) status: Status,
    /// A mode change is running; ignore refreshes so the UI doesn't flicker back.
    busy: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    Refresh,
    StatusLoaded(Result<Status, String>),
    PopupClosed(Id),
    Surface(cosmic::surface::Action<Message>),
    SetMode(Mode),
    ModeApplied(Result<(), String>),
    OpenSettings,
}

/// Run a future and wrap its output in an applet message.
fn perform<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static,
    to_message: impl FnOnce(T) -> Message + Send + 'static,
) -> Task<Message> {
    Task::perform(future, move |out| Action::App(to_message(out)))
}

impl Applet {
    fn load_status() -> Task<Message> {
        perform(backend::status(), Message::StatusLoaded)
    }

    fn close_popup(&mut self) -> Task<Message> {
        match self.popup.take() {
            Some(id) => cosmic::task::message(Action::Surface(destroy_popup(id))),
            None => Task::none(),
        }
    }

    /// Build the surface action that opens the popup anchored to the panel button.
    fn open_popup(bounds: Rectangle, offset: Vector) -> Message {
        Message::Surface(app_popup::<Applet>(
            |_| Default::default(),
            move |state: &mut Applet| {
                let id = Id::unique();
                state.popup = Some(id);
                let mut settings = state.core.applet.get_popup_settings(
                    state.core.main_window_id().expect("applet has a main window"),
                    id,
                    None,
                    None,
                    None,
                );
                settings.positioner.anchor_rect = Rectangle {
                    x: (bounds.x - offset.x) as i32,
                    y: (bounds.y - offset.y) as i32,
                    width: bounds.width as i32,
                    height: bounds.height as i32,
                };
                settings
            },
            Some(Box::new(|state: &Applet| {
                let content = crate::popup::view(state);
                Element::from(state.core.applet.popup_container(content)).map(Action::App)
            })),
        ))
    }
}

impl cosmic::Application for Applet {
    type Executor = cosmic::SingleThreadExecutor;
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
        let applet = Applet { core, popup: None, status: Status::default(), busy: false };
        (applet, Self::load_status())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        cosmic::iced::time::every(REFRESH_INTERVAL).map(|_| Message::Refresh)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh if !self.busy => return Self::load_status(),
            Message::Refresh => {}
            Message::StatusLoaded(Ok(status)) if !self.busy => self.status = status,
            Message::StatusLoaded(Ok(_)) => {}
            Message::StatusLoaded(Err(e)) => eprintln!("ember: {e}"),
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Surface(action) => return cosmic::task::message(Action::Surface(action)),
            Message::SetMode(mode) => {
                self.status.mode = mode;
                self.busy = true;
                return perform(backend::set_mode(mode), Message::ModeApplied);
            }
            Message::ModeApplied(result) => {
                if let Err(e) = result {
                    eprintln!("ember: {e}");
                }
                self.busy = false;
                return Self::load_status();
            }
            Message::OpenSettings => {
                backend::open_settings_app();
                return self.close_popup();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let popup = self.popup;
        let button = self
            .core
            .applet
            .icon_button(self.status.icon())
            .on_press_with_rectangle(move |offset, bounds| match popup {
                Some(id) => Message::Surface(destroy_popup(id)),
                None => Self::open_popup(bounds, offset),
            });

        let tooltip = format!("Ember: {}", self.status.headline());
        self.core
            .applet
            .applet_tooltip::<Message>(button, tooltip, self.popup.is_some(), Message::Surface, None)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        widget::text("").into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
