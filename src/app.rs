// SPDX-License-Identifier: GPL-3.0-only

use std::sync::{Arc, Mutex};

use iced::{
    Element, Length, Subscription, Task, Theme,
    time::Instant,
    widget::{container, text},
};
use tracing::{error, info};

use crate::{
    APP_ID,
    app::{
        core::check_database,
        screen::{HomePage, Screen, UnlockDatabase, create, homepage, unlock},
        widgets::Toast,
    },
    config::Config,
};

mod core;
mod screen;
mod utils;
mod widgets;

pub struct FreeTotp {
    toasts: Vec<Toast>,
    config: Arc<Mutex<Config>>,
    now: Instant,
    screen: Screen,
    main_window_id: Option<iced::window::Id>,
    _tray: Option<tray_icon::TrayIcon>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Callback after loading the application [`Config`]
    ConfigLoaded(Result<Config, anywho::Error>),
    /// Add a new [`Toast`] to show in the app
    AddToast(Toast),
    /// Close the given [`Toast`]
    CloseToast(usize),
    /// Create Database [`Screen`] Messages
    CreateDatabase(create::Message),
    /// Unlock Database [`Screen`] Messages
    UnlockDatabase(unlock::Message),
    /// Homepage [`Screen`] Messages
    HomePage(homepage::Message),
    /// Tray icon events
    TrayIconEvent(tray_icon::TrayIconEvent),
    /// Tray menu events
    MenuEvent(tray_icon::menu::MenuEvent),
    /// Window events (for intercepting close)
    WindowEvent(iced::window::Id, iced::Event),
}

impl FreeTotp {
    pub fn new() -> (Self, Task<Message>) {
        info!("Starting app");

        // On macOS, set the app to Accessory mode so it doesn't show in the Dock
        utils::set_activation_policy_accessory();

        // Initialize tray icon here to avoid conflict with winit on macOS
        let tray = {
            use tray_icon::{Icon, TrayIconBuilder};

            let icon_data = include_bytes!("../resources/icons/hicolor/256x256/apps/icon.png");
            let icon = image::load_from_memory(icon_data)
                .ok()
                .and_then(|img| {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    Icon::from_rgba(rgba.into_raw(), width, height).ok()
                });

            let tray_menu = tray_icon::menu::Menu::new();
            let show_item = tray_icon::menu::MenuItem::with_id("show", "Show", true, None);
            let quit_item = tray_icon::menu::MenuItem::with_id("quit", "Quit", true, None);
            let _ = tray_menu.append_items(&[&show_item, &tray_icon::menu::PredefinedMenuItem::separator(), &quit_item]);

            if let Some(icon) = icon {
                TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu))
                    .with_tooltip("FreeTotp")
                    .with_icon(icon)
                    .build()
                    .ok()
            } else {
                None
            }
        };

        let (screen, task) = Screen::from_database_check(check_database());
        (
            Self {
                toasts: Vec::new(),
                config: Arc::from(Mutex::new(Config::default())),
                now: Instant::now(),
                screen,
                main_window_id: None,
                _tray: tray,
            },
            Task::perform(Config::load(APP_ID), Message::ConfigLoaded).chain(task),
        )
    }

    pub fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::ConfigLoaded(res) => {
                match res {
                    Ok(config) => {
                        info!("Config loaded successfully");
                        self.config = Arc::new(Mutex::from(config));
                    }
                    Err(err) => {
                        error!("Error loading config: {err}");
                    }
                }
                Task::none()
            }
            Message::AddToast(toast) => {
                self.toasts.push(toast);
                Task::none()
            }
            Message::CloseToast(index) => {
                self.toasts.remove(index);
                Task::none()
            }

            Message::CreateDatabase(message) => {
                let Screen::CreateDatabase(create_database) = &mut self.screen else {
                    return Task::none();
                };

                match create_database.update(message, self.now) {
                    create::Action::None => Task::none(),
                    create::Action::Run(task) => task.map(Message::CreateDatabase),
                    create::Action::AddToast(toast) => self.update(Message::AddToast(toast), now),
                    create::Action::OpenUnlockDatabase(db_path) => {
                        let (unlock_database, task) = UnlockDatabase::new(db_path);

                        self.screen = Screen::UnlockDatabase(unlock_database);
                        task.map(Message::UnlockDatabase)
                    }
                }
            }

            Message::UnlockDatabase(message) => {
                let Screen::UnlockDatabase(unlock_database) = &mut self.screen else {
                    return Task::none();
                };

                match unlock_database.update(message, self.now) {
                    unlock::Action::None => Task::none(),
                    unlock::Action::Run(task) => task.map(Message::UnlockDatabase),
                    unlock::Action::AddToast(toast) => self.update(Message::AddToast(toast), now),
                    unlock::Action::OpenHomePage(database) => {
                        let (homepage, task) =
                            HomePage::new(Arc::new(*database), Arc::clone(&self.config));

                        self.screen = Screen::HomePage(homepage);
                        task.map(Message::HomePage)
                    }
                }
            }

            Message::HomePage(message) => {
                let Screen::HomePage(homepage) = &mut self.screen else {
                    return Task::none();
                };

                match homepage.update(message, self.now) {
                    homepage::Action::None => Task::none(),
                    homepage::Action::Run(task) => task.map(Message::HomePage),
                    homepage::Action::AddToast(toast) => self.update(Message::AddToast(toast), now),
                    homepage::Action::RunAndToast(task, toast) => Task::batch([
                        task.map(Message::HomePage),
                        self.update(Message::AddToast(toast), now),
                    ]),
                }
            }
            Message::TrayIconEvent(event) => {
                if let tray_icon::TrayIconEvent::Click { .. } = event {
                    if let Some(id) = self.main_window_id {
                        return Task::batch([
                            iced::window::set_mode(id, iced::window::Mode::Windowed),
                            iced::window::gain_focus(id),
                        ]);
                    }
                }
                Task::none()
            }
            Message::MenuEvent(event) => {
                match event.id.0.as_str() {
                    "show" => {
                        if let Some(id) = self.main_window_id {
                            return Task::batch([
                                iced::window::set_mode(id, iced::window::Mode::Windowed),
                                iced::window::gain_focus(id),
                            ]);
                        }
                    }
                    "quit" => {
                        if let Some(id) = self.main_window_id {
                            return iced::window::close(id);
                        } else {
                            // If we don't have the window ID yet, just exit
                            std::process::exit(0);
                        }
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::WindowEvent(id, event) => {
                if self.main_window_id.is_none() {
                    self.main_window_id = Some(id);
                }

                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    let stay_on_tray =
                        self.config.lock().map(|c| c.stay_on_tray).unwrap_or_default();
                    if stay_on_tray {
                        return iced::window::set_mode(id, iced::window::Mode::Hidden);
                    } else {
                        return iced::exit();
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = match &self.screen {
            Screen::Error(error) => container(text(error)).center(Length::Fill).into(),
            Screen::CreateDatabase(create_database) => {
                create_database.view(self.now).map(Message::CreateDatabase)
            }
            Screen::UnlockDatabase(unlock_database) => {
                unlock_database.view(self.now).map(Message::UnlockDatabase)
            }
            Screen::HomePage(homepage) => homepage.view(self.now).map(Message::HomePage),
        };

        widgets::toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            iced::event::listen_with(|event, _status, id| Some(Message::WindowEvent(id, event))),
            iced::Subscription::run(|| {
                iced::futures::stream::unfold((), |_| async {
                    let event =
                        smol::unblock(|| tray_icon::TrayIconEvent::receiver().recv().ok()).await?;
                    Some((Message::TrayIconEvent(event), ()))
                })
            }),
            iced::Subscription::run(|| {
                iced::futures::stream::unfold((), |_| async {
                    let event =
                        smol::unblock(|| tray_icon::menu::MenuEvent::receiver().recv().ok()).await?;
                    Some((Message::MenuEvent(event), ()))
                })
            }),
        ];

        match &self.screen {
            Screen::Error(_) => {}
            Screen::CreateDatabase(create_database) => {
                subscriptions.push(
                    create_database
                        .subscription(self.now)
                        .map(Message::CreateDatabase),
                );
            }
            Screen::UnlockDatabase(unlock_database) => {
                subscriptions.push(
                    unlock_database
                        .subscription(self.now)
                        .map(Message::UnlockDatabase),
                );
            }
            Screen::HomePage(homepage) => {
                subscriptions.push(homepage.subscription(self.now).map(Message::HomePage));
            }
        }

        Subscription::batch(subscriptions)
    }

    pub fn theme(&self) -> Theme {
        self.config.lock().map_or_else(
            |_| iced::Theme::Light, // fallback theme if lock fails
            |cfg| cfg.theme.clone().into(),
        )
    }
}
