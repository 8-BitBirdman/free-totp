// SPDX-License-Identifier: GPL-3.0-only

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use iced::{
    Alignment, Element,
    Length::{self},
    Subscription, Task, clipboard,
    time::Instant,
    widget::{Column, button, column, container, row, scrollable, space, text},
};
use tracing::error;

use crate::{
    app::{
        core::{FreeTotpDatabase, FreeTotpEntry},
        utils::{get_time_until_next_totp_refresh, style},
        widgets::{Toast, dot},
    },
    config::Config,
    icons,
};

mod settings;
mod upsert;

pub struct HomePage {
    config: Arc<Mutex<Config>>,
    database: Arc<FreeTotpDatabase>,
    state: State,
}

pub enum State {
    Loading,
    Ready { subscreen: SubScreen },
}

pub enum SubScreen {
    Home { entries: Vec<FreeTotpEntry> },
    UpsertPage(upsert::UpsertPage),
    SettingsPage(settings::SettingsPage),
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Attempt to copy some [`String`] to the user clipboard
    CopyToClipboard(String),
    /// Callback after attempting to copy something to the clipboard
    ClipboardResult(Result<(), iced::clipboard::Error>),
    /// Ask to load the [`FreeTotpEntry`]s to list on the page
    LoadEntries,
    /// Callback after asking to load [`FreeTotpEntry`]s, set's the entries on the state if Ok
    EntriesLoaded(Result<Vec<FreeTotpEntry>, anywho::Error>),

    /// Messages of the [`UpsertPage`]
    UpsertPage(upsert::Message),
    /// Ask to open the [`FreeTotpEntry`]  [`UpsertPage`]
    OpenUpsertPage(Option<FreeTotpEntry>),
    /// Callback after upserting a [`FreeTotpEntry`]
    EntryUpserted(Result<(), anywho::Error>),
    /// Callback after batch upserting multiple [`FreeTotpEntry`]
    BatchEntriesUpserted(Result<usize, anywho::Error>),

    /// Messages of the [`SettingsPage`]
    SettingsPage(settings::Message),
    /// Ask to open the [`SettingsPage`]
    OpenSettingsPage,

    /// Makes iced rerun the view to refresh and tick the timers, runs every second on a subscription
    RefreshCodes,
}

pub enum Action {
    /// Does nothing
    None,
    /// Ask parent to run an [`iced::Task`]
    Run(Task<Message>),
    /// Add a new [`Toast`] to show
    AddToast(Toast),
    /// Ask parent to run an [`iced::Task`] and add a [`Toast`] to show
    RunAndToast(Task<Message>, Toast),
}

impl HomePage {
    pub fn new(
        database: Arc<FreeTotpDatabase>,
        config: Arc<Mutex<Config>>,
    ) -> (Self, Task<Message>) {
        let db_clone = Arc::clone(&database);

        (
            Self {
                config,
                database,
                state: State::Loading,
            },
            Task::perform(
                async move { db_clone.list_entries().await },
                Message::EntriesLoaded,
            ),
        )
    }

    pub fn view(&self, now: Instant) -> iced::Element<'_, Message> {
        let content: Element<Message> = match &self.state {
            State::Loading => text("Loading...").into(),
            State::Ready { subscreen } => match subscreen {
                SubScreen::Home { entries } => {
                    let header = header_view(entries.len());
                    let content = content_view(entries);

                    container(column![header, content])
                        .padding(5.)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
                SubScreen::UpsertPage(upsert_page) => {
                    upsert_page.view(now).map(Message::UpsertPage)
                }
                SubScreen::SettingsPage(settings_page) => {
                    settings_page.view(now).map(Message::SettingsPage)
                }
            },
        };

        container(content).center(Length::Fill).into()
    }

    pub fn update(&mut self, message: Message, now: Instant) -> Action {
        match message {
            Message::CopyToClipboard(value) => {
                Action::Run(clipboard::write(value).map(Message::ClipboardResult))
            }
            Message::ClipboardResult(result) => match result {
                Ok(_) => Action::AddToast(Toast::success_toast("Copied to clipboard")),
                Err(err) => {
                    error!("{:?}", err);
                    Action::None
                }
            },
            Message::LoadEntries => {
                self.state = State::Loading;

                let db_clone = Arc::clone(&self.database);
                Action::Run(Task::perform(
                    async move { db_clone.list_entries().await },
                    Message::EntriesLoaded,
                ))
            }
            Message::EntriesLoaded(result) => match result {
                Ok(entries) => {
                    self.state = State::Ready {
                        subscreen: SubScreen::Home { entries },
                    };
                    Action::None
                }
                Err(err) => {
                    error!("{}", err);
                    Action::AddToast(Toast::error_toast(err))
                }
            },

            Message::UpsertPage(message) => {
                let State::Ready { subscreen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::UpsertPage(upsert_page) = subscreen else {
                    return Action::None;
                };

                match upsert_page.update(message, now) {
                    upsert::Action::None => Action::None,
                    upsert::Action::Back => self.update(Message::LoadEntries, now),
                    upsert::Action::Run(task) => Action::Run(task.map(Message::UpsertPage)),
                    upsert::Action::AddToast(toast) => Action::AddToast(toast),
                    upsert::Action::UpdateEntry(free_totp_entry) => {
                        let db_clone = Arc::clone(&self.database);
                        Action::Run(Task::perform(
                            async move { db_clone.update_entry(free_totp_entry).await },
                            Message::EntryUpserted,
                        ))
                    }
                    upsert::Action::CreateEntry(free_totp_entry) => {
                        let db_clone = Arc::clone(&self.database);
                        Action::Run(Task::perform(
                            async move { db_clone.add_entry(free_totp_entry).await },
                            Message::EntryUpserted,
                        ))
                    }
                    upsert::Action::CreateEntries(free_totp_entries) => {
                        let db_clone = Arc::clone(&self.database);
                        let count = free_totp_entries.len();
                        Action::Run(Task::perform(
                            async move {
                                db_clone
                                    .add_entries(free_totp_entries)
                                    .await
                                    .map(|_| count)
                            },
                            Message::BatchEntriesUpserted,
                        ))
                    }
                    upsert::Action::DeleteEntry(uuid) => {
                        let db_clone = Arc::clone(&self.database);
                        Action::Run(Task::perform(
                            async move { db_clone.delete_entry(uuid).await },
                            Message::EntryUpserted,
                        ))
                    }
                }
            }
            Message::OpenUpsertPage(entry) => {
                let State::Ready { subscreen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (upsert_page, task) = upsert::UpsertPage::new(entry);
                *subscreen = SubScreen::UpsertPage(upsert_page);
                Action::Run(task.map(Message::UpsertPage))
            }
            Message::EntryUpserted(result) => match result {
                Ok(_) => self.update(Message::LoadEntries, now),
                Err(err) => {
                    self.state = State::Loading;
                    let db_clone = Arc::clone(&self.database);
                    Action::RunAndToast(
                        Task::perform(
                            async move { db_clone.list_entries().await },
                            Message::EntriesLoaded,
                        ),
                        Toast::error_toast(err),
                    )
                }
            },
            Message::BatchEntriesUpserted(result) => match result {
                Ok(count) => {
                    let action = self.update(Message::LoadEntries, now);
                    match action {
                        Action::Run(task) => Action::RunAndToast(
                            task,
                            Toast::success_toast(format!("Successfully imported {} entries", count)),
                        ),
                        _ => action,
                    }
                }
                Err(err) => {
                    self.state = State::Loading;
                    let db_clone = Arc::clone(&self.database);
                    Action::RunAndToast(
                        Task::perform(
                            async move { db_clone.list_entries().await },
                            Message::EntriesLoaded,
                        ),
                        Toast::error_toast(err),
                    )
                }
            },

            Message::SettingsPage(message) => {
                let State::Ready { subscreen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::SettingsPage(settings_page) = subscreen else {
                    return Action::None;
                };

                match settings_page.update(message, now) {
                    settings::Action::None => Action::None,
                    settings::Action::Back => self.update(Message::LoadEntries, now),
                    settings::Action::Run(task) => Action::Run(task.map(Message::SettingsPage)),
                    settings::Action::AddToast(toast) => Action::AddToast(toast),
                    settings::Action::ImportContent(path_buf) => {
                        let db_clone = Arc::clone(&self.database);
                        Action::Run(Task::perform(
                            async move { db_clone.import_content(path_buf).await },
                            Message::EntryUpserted,
                        ))
                    }
                    settings::Action::ExportContent(path_buf) => {
                        let db_clone = Arc::clone(&self.database);
                        Action::Run(Task::perform(
                            async move { db_clone.export_content(path_buf).await },
                            Message::EntryUpserted,
                        ))
                    }
                }
            }
            Message::OpenSettingsPage => {
                let State::Ready { subscreen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (settings_page, task) = settings::SettingsPage::new(Arc::clone(&self.config));
                *subscreen = SubScreen::SettingsPage(settings_page);
                Action::Run(task.map(Message::SettingsPage))
            }

            Message::RefreshCodes => {
                // This forces a re-render every second
                // Since view() calls totp.generate_current(), codes will update automatically
                Action::None
            }
        }
    }

    pub fn subscription(&self, now: Instant) -> Subscription<Message> {
        let State::Ready { subscreen, .. } = &self.state else {
            return Subscription::none();
        };

        match subscreen {
            SubScreen::Home { entries } => {
                if entries.is_empty() {
                    Subscription::none()
                } else {
                    iced::time::every(Duration::from_secs(1)).map(|_| Message::RefreshCodes)
                }
            }
            SubScreen::UpsertPage(upsert_page) => {
                upsert_page.subscription(now).map(Message::UpsertPage)
            }
            SubScreen::SettingsPage(settings_page) => {
                settings_page.subscription(now).map(Message::SettingsPage)
            }
        }
    }
}

/// View of the header of this screen
fn header_view<'a>(entry_count: usize) -> Element<'a, Message> {
    row![
        // Title section
        column![
            text("FreeTotp")
                .size(style::font_size::TITLE),
            text(format!(
                "{} {}",
                entry_count,
                if entry_count == 1 { "Entry" } else { "Entries" }
            ))
            .size(style::font_size::BODY)
            .style(style::muted_text)
        ]
        .spacing(style::spacing::TINY),
        space().width(Length::Fill),
        // Action buttons
        row![
            iced::widget::tooltip(
                button(
                    icons::get_icon("list-add-symbolic", 24).style(|theme, _status| {
                        let style = style::primary_button(theme, iced::widget::button::Status::Active);
                        iced::widget::svg::Style {
                            color: Some(style.text_color),
                        }
                    })
                )
                .on_press(Message::OpenUpsertPage(None))
                .padding(10)
                .style(style::primary_button),
                "Add New Entry",
                iced::widget::tooltip::Position::Bottom
            ),
            iced::widget::tooltip(
                button(
                    icons::get_icon("emblem-system-symbolic", 24).style(|theme, _status| {
                        let style = style::secondary_button(theme, iced::widget::button::Status::Active);
                        iced::widget::svg::Style {
                            color: Some(style.text_color),
                        }
                    })
                )
                .on_press(Message::OpenSettingsPage)
                .padding(10)
                .style(style::secondary_button),
                "Settings",
                iced::widget::tooltip::Position::Bottom
            ),
        ]
        .spacing(style::spacing::MEDIUM)
    ]
    .spacing(style::spacing::LARGE)
    .padding(20)
    .align_y(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// View of the contents of this screen
fn content_view<'a>(entries: &'a [FreeTotpEntry]) -> Element<'a, Message> {
    if entries.is_empty() {
        container(
            column![
                text("No TOTP entries found")
                    .size(style::font_size::TITLE),
                text("Add your first entry to get started")
                    .size(style::font_size::BODY)
                    .style(style::muted_text),
                space().height(Length::Fixed(20.0)),
                button(text("Add Entry").size(style::font_size::MEDIUM))
                    .on_press(Message::OpenUpsertPage(None))
                    .padding([12.0, 24.0])
                    .style(style::primary_button),
            ]
            .align_x(Alignment::Center)
            .spacing(style::spacing::MEDIUM),
        )
        .center(Length::Fill)
        .into()
    } else {
        let entries_list = entries.iter().fold(
            Column::new()
                .height(Length::Fill)
                .spacing(style::spacing::LARGE)
                .padding(20),
            |col, entry| {
                let code = entry.totp.generate_current().unwrap_or_default();
                let time_remaining = get_time_until_next_totp_refresh(entry.totp.step);

                // Add a space to format the TOTP code nicely (e.g., 123 456)
                let formatted_code = if code.len() == 6 {
                    format!("{} {}", &code[0..3], &code[3..6])
                } else if code.len() == 8 {
                    format!("{} {}", &code[0..4], &code[4..8])
                } else {
                    code.clone()
                };

                let entry_view = container(
                    row![
                        column![
                            text(&entry.name)
                                .wrapping(text::Wrapping::Glyph)
                                .size(style::font_size::LARGE),
                            row![
                                text(format!(
                                    "{} digits · {}s",
                                    entry.totp.digits, time_remaining
                                ))
                                .size(style::font_size::SMALL)
                                .style(style::muted_text),
                                dot(time_remaining)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(style::spacing::SMALL),
                        ]
                        .spacing(style::spacing::TINY)
                        .width(Length::Fill),
                        
                        text(formatted_code)
                            .size(style::font_size::HERO)
                            .font(iced::Font::MONOSPACE),
                            
                        space().width(Length::Fixed(24.0)),
                        
                        row![
                            button(icons::get_icon("edit-copy-symbolic", 22))
                                .on_press(Message::CopyToClipboard(code))
                                .padding(10)
                                .style(style::primary_button),
                            button(icons::get_icon("edit-symbolic", 22))
                                .on_press(Message::OpenUpsertPage(Some(entry.clone())))
                                .padding(10)
                                .style(style::secondary_button),
                        ]
                        .spacing(style::spacing::SMALL)
                    ]
                    .padding(24)
                    .align_y(iced::Alignment::Center),
                )
                .style(style::entry_card);

                col.push(entry_view)
            },
        );

        scrollable(entries_list).height(Length::Fill).into()
    }
}
