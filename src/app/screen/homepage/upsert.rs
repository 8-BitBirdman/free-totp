// SPDX-License-Identifier: GPL-3.0-only

use iced::{
    Alignment, Element,
    Length::{self},
    Subscription, Task, event,
    keyboard::{self, Key, Modifiers, key::Named},
    time::Instant,
    widget::{
        button, column, container, image,
        operation::{focus_next, focus_previous},
        pick_list, row, scrollable, space, stack, text, text_input,
    },
};
use rfd::{AsyncFileDialog, FileHandle};
use totp_rs::Algorithm;

use crate::{
    app::{
        core::FreeTotpEntry,
        utils::{ALL_ALGORITHMS, InputableFreeTotpEntry, read_qr_from_file, style},
        widgets::Toast,
    },
    icons,
};

#[cfg(target_os = "linux")]
mod scan_qr;

pub struct UpsertPage {
    entry: InputableFreeTotpEntry,
    show_qr: bool,
    subscreen: SubScreen,
}

pub enum SubScreen {
    UpsertPage,
    #[cfg(target_os = "linux")]
    ScanQrPage(scan_qr::QrScanPage),
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Callback after pressing a [`Hotkey`] of this page
    Hotkey(Hotkey),
    /// Go back a screen
    Back,

    /// Input update of the various available fields
    InputUpdated(TOTPEntryInput),
    /// Submit the changes
    Submit,
    /// Delete the currently editing entry
    Delete,

    /// Opens the dialog to select a QR file
    OpenQrFileSelection,
    /// Callback after selecting QR files
    QrFilesSelected(Option<Vec<FileHandle>>),
    /// Callback after processing QR files in background
    QrFilesProcessed(Vec<InputableFreeTotpEntry>),

    /// Wants to show/hide the current entry qr code
    ToggleShowQRCode,

    /// Messages of the [`ScanQrPage`]
    #[cfg(target_os = "linux")]
    ScanQrPage(scan_qr::Message),

    /// Ask to open the [`ScanQrPage`]
    #[cfg(target_os = "linux")]
    OpenScanQrPage,
}

pub enum Action {
    /// Does nothing
    None,
    /// Go back a screen
    Back,
    // Ask parent to run an [`iced::Task`]
    Run(Task<Message>),
    /// Add a new [`Toast`] to show
    AddToast(Toast),
    /// Ask the parent to update the given [`FreeTotpEntry`]
    UpdateEntry(FreeTotpEntry),
    /// Ask the parent to create the given [`FreeTotpEntry`]
    CreateEntry(FreeTotpEntry),
    /// Ask the parent to create multiple [`FreeTotpEntry`]
    CreateEntries(Vec<FreeTotpEntry>),
    /// Ask the parent to delete the [`FreeTotpEntry`] with the give [`uuid::Uuid`]
    DeleteEntry(uuid::Uuid),
}

/// Represents the different inputs the user can perfrom on the upsert screen
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum TOTPEntryInput {
    UpdateName(String),
    UpdateAlgorithm(Algorithm),
    UpdateDigits(String),
    UpdateStep(String),
    UpdateSecret(String),
    UpdateIssuer(String),
    UpdateAccountName(String),
}

impl UpsertPage {
    pub fn new(entry: Option<FreeTotpEntry>) -> (Self, Task<Message>) {
        let entry = entry.map(InputableFreeTotpEntry::from).unwrap_or_default();

        (
            Self {
                entry,
                show_qr: false,
                subscreen: SubScreen::UpsertPage,
            },
            Task::none(),
        )
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        match &self.subscreen {
            SubScreen::UpsertPage => {
                let header = header_view(&self.entry);
                let content = upsert_entry_view(&self.entry, self.show_qr);

                container(
                    container(column![header, content])
                        .padding(5.)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .center(Length::Fill)
                .into()
            }
            #[cfg(target_os = "linux")]
            SubScreen::ScanQrPage(qr_scan_page) => qr_scan_page.view(_now).map(Message::ScanQrPage),
        }
    }

    pub fn update(&mut self, message: Message, _now: Instant) -> Action {
        match message {
            Message::Hotkey(hotkey) => match hotkey {
                Hotkey::Tab(modifiers) => {
                    if modifiers.shift() {
                        Action::Run(focus_previous())
                    } else {
                        Action::Run(focus_next())
                    }
                }
                Hotkey::Esc => Action::Back,
            },
            Message::Back => Action::Back,

            Message::InputUpdated(input) => {
                match input {
                    TOTPEntryInput::UpdateName(v) => self.entry.name = v,
                    TOTPEntryInput::UpdateAlgorithm(v) => self.entry.algorithm = v,
                    TOTPEntryInput::UpdateDigits(v) => {
                        if !v.is_empty() {
                            if let Ok(parsed) = v.parse::<usize>() {
                                self.entry.digits = parsed;
                            }
                        } else {
                            self.entry.digits = 0;
                        }
                    }
                    TOTPEntryInput::UpdateStep(v) => {
                        if !v.is_empty() {
                            if let Ok(parsed) = v.parse::<u64>() {
                                self.entry.step = parsed;
                            }
                        } else {
                            self.entry.step = 0;
                        }
                    }
                    TOTPEntryInput::UpdateSecret(v) => self.entry.secret = v,
                    TOTPEntryInput::UpdateIssuer(v) => {
                        if v.is_empty() {
                            self.entry.issuer = None;
                        } else {
                            self.entry.issuer = Some(v);
                        }
                    }
                    TOTPEntryInput::UpdateAccountName(v) => self.entry.account_name = v,
                }
                Action::None
            }
            Message::Submit => {
                if self.entry.valid() {
                    let free_totp_entry_res = FreeTotpEntry::try_from(self.entry.clone());
                    match free_totp_entry_res {
                        Ok(free_totp_entry) => {
                            if free_totp_entry.id.is_some() {
                                Action::UpdateEntry(free_totp_entry)
                            } else {
                                Action::CreateEntry(free_totp_entry)
                            }
                        }
                        Err(err) => Action::AddToast(Toast::error_toast(err)),
                    }
                } else {
                    Action::AddToast(Toast::error_toast("Invalid TOTP Entity"))
                }
            }
            Message::Delete => {
                if let Some(id) = self.entry.uuid {
                    Action::DeleteEntry(id)
                } else {
                    Action::None
                }
            }

            Message::OpenQrFileSelection => {
                if self.entry.uuid.is_none() {
                    Action::Run(Task::perform(
                        async move {
                            AsyncFileDialog::new()
                                .add_filter("Image Files", &["png", "jpeg", "jpg", "webp"])
                                .set_directory(dirs::download_dir().unwrap_or("/".into()))
                                .pick_files()
                                .await
                        },
                        Message::QrFilesSelected,
                    ))
                } else {
                    Action::None
                }
            }
            Message::QrFilesSelected(handles) => {
                if let Some(file_handles) = handles {
                    let paths: Vec<_> = file_handles.iter().map(|h| h.path().to_path_buf()).collect();
                    
                    Action::Run(Task::perform(
                        async move {
                            let mut valid_entries = Vec::new();
                            for path in paths {
                                if let Ok(values) = read_qr_from_file(path) {
                                    for value in values {
                                        if let Ok(entries) = InputableFreeTotpEntry::from_url(value) {
                                            valid_entries.extend(entries);
                                        }
                                    }
                                }
                            }
                            valid_entries
                        },
                        Message::QrFilesProcessed,
                    ))
                } else {
                    Action::None
                }
            }
            Message::QrFilesProcessed(valid_entries) => {
                if valid_entries.is_empty() {
                    Action::AddToast(Toast::warning_toast(
                        "No valid TOTP entries were found in the selected files",
                    ))
                } else if valid_entries.len() == 1 {
                    self.entry = valid_entries[0].clone();
                    Action::None
                } else {
                    // Batch add all and go back
                    let entries = valid_entries
                        .into_iter()
                        .filter_map(|e| FreeTotpEntry::try_from(e).ok())
                        .collect::<Vec<_>>();

                    if entries.is_empty() {
                        Action::AddToast(Toast::error_toast("Failed to process entries"))
                    } else {
                        Action::CreateEntries(entries)
                    }
                }
            }

            Message::ToggleShowQRCode => {
                if self.entry.uuid.is_some() {
                    self.show_qr = !self.show_qr;
                }
                Action::None
            }

            #[cfg(target_os = "linux")]
            Message::ScanQrPage(message) => {
                let SubScreen::ScanQrPage(scan_qr_page) = &mut self.subscreen else {
                    return Action::None;
                };

                match scan_qr_page.update(message, _now) {
                    scan_qr::Action::None => Action::None,
                    scan_qr::Action::Back => {
                        self.subscreen = SubScreen::UpsertPage;
                        Action::None
                    }
                    scan_qr::Action::AddToast(toast) => Action::AddToast(toast),
                    scan_qr::Action::AddToastAndBack(toast) => {
                        self.subscreen = SubScreen::UpsertPage;
                        Action::AddToast(toast)
                    }
                    scan_qr::Action::EntriesDetected(entries) => {
                        if entries.len() == 1 {
                            self.entry = entries[0].clone();
                            self.subscreen = SubScreen::UpsertPage;
                            Action::AddToast(Toast::success_toast(format!(
                                "Code detected correctly for: {}",
                                &self.entry.account_name
                            )))
                        } else {
                            // Convert and batch add
                            let converted_entries = entries
                                .into_iter()
                                .filter(|e| e.valid())
                                .filter_map(|e| FreeTotpEntry::try_from(e).ok())
                                .collect::<Vec<_>>();

                            if converted_entries.is_empty() {
                                self.subscreen = SubScreen::UpsertPage;
                                Action::AddToast(Toast::warning_toast(
                                    "Detected entries could not be converted",
                                ))
                            } else {
                                let count = converted_entries.len();
                                Action::CreateEntries(converted_entries)
                            }
                        }
                    }
                }
            }
            #[cfg(target_os = "linux")]
            Message::OpenScanQrPage => match scan_qr::QrScanPage::new() {
                Ok((scan_qr_page, task)) => {
                    self.subscreen = SubScreen::ScanQrPage(scan_qr_page);
                    Action::Run(task.map(Message::ScanQrPage))
                }
                Err(e) => {
                    error!("{e}");
                    Action::AddToast(Toast::error_toast(format!("Failed to open camera: {}", e)))
                }
            },
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        match &self.subscreen {
            SubScreen::UpsertPage => event::listen_with(handle_event),
            #[cfg(target_os = "linux")]
            SubScreen::ScanQrPage(qr_scan_page) => {
                qr_scan_page.subscription(_now).map(Message::ScanQrPage)
            }
        }
    }
}

/// View of the header of this screen
fn header_view<'a>(entry: &'a InputableFreeTotpEntry) -> Element<'a, Message> {
    iced::widget::responsive(move |size| {
        let (title, subtitle) = if entry.uuid.is_some() {
            ("Edit Entry", "Modify your TOTP entry")
        } else {
            ("New Entry", "Add a new TOTP entry")
        };

        let mobile_view = size.width < 400.0; // Threshold for stacked/horizontally alligned buttons
        let buttons_width = if mobile_view {
            Length::Fill
        } else {
            Length::Shrink
        };

        let is_new_entry = entry.uuid.is_none();

        let mut buttons: Vec<Element<Message>> = Vec::new();

        if is_new_entry {
            // Only for new entries
            buttons.push(
                button(
                    row![
                        icons::get_icon("qr-symbolic", 21).style(|theme, _status| {
                            let primary_style =
                                button::primary(theme, iced::widget::button::Status::Active);
                            iced::widget::svg::Style {
                                color: Some(primary_style.text_color),
                            }
                        }),
                        text("File").size(style::font_size::BODY)
                    ]
                    .spacing(style::spacing::TINY)
                    .align_y(iced::Alignment::Center),
                )
                .width(buttons_width)
                .style(style::primary_button)
                .padding(8)
                .on_press(Message::OpenQrFileSelection)
                .into(),
            );

            #[cfg(target_os = "linux")]
            buttons.push(
                button(
                    row![
                        icons::get_icon("qr-symbolic", 21).style(|theme, _status| {
                            let primary_style =
                                button::primary(theme, iced::widget::button::Status::Active);
                            iced::widget::svg::Style {
                                color: Some(primary_style.text_color),
                            }
                        }),
                        text("Camera").size(style::font_size::BODY)
                    ]
                    .spacing(style::spacing::TINY)
                    .align_y(iced::Alignment::Center),
                )
                .width(buttons_width)
                .style(style::primary_button)
                .padding(8)
                .on_press(Message::OpenScanQrPage)
                .into(),
            );
        } else {
            // Only for existing entries
            buttons.extend([
                button(
                    row![
                        icons::get_icon("user-trash-full-symbolic", 21).style(|theme, _status| {
                            let danger_style =
                                button::danger(theme, iced::widget::button::Status::Active);
                            iced::widget::svg::Style {
                                color: Some(danger_style.text_color),
                            }
                        }),
                        text("Delete").size(style::font_size::BODY)
                    ]
                    .spacing(style::spacing::TINY)
                    .align_y(iced::Alignment::Center),
                )
                .width(buttons_width)
                .style(style::danger_button)
                .padding(8)
                .on_press(Message::Delete)
                .into(),
                button(
                    row![
                        icons::get_icon("qr-symbolic", 21).style(|theme, _status| {
                            let success_style =
                                button::success(theme, iced::widget::button::Status::Active);
                            iced::widget::svg::Style {
                                color: Some(success_style.text_color),
                            }
                        }),
                        text("Show QR").size(style::font_size::BODY)
                    ]
                    .spacing(style::spacing::TINY)
                    .align_y(iced::Alignment::Center),
                )
                .width(buttons_width)
                .style(style::success_button)
                .padding(8)
                .on_press(Message::ToggleShowQRCode)
                .into(),
            ]);
        }

        let buttons_layout: Element<Message> = if mobile_view && buttons.len() > 1 {
            // buttons vertically on narrow screens
            column(buttons).spacing(style::spacing::SMALL).into()
        } else {
            // buttons horizontal on wide screens
            row(buttons).spacing(style::spacing::LARGE).into()
        };

        row![
            button(
                row![
                    icons::get_icon("go-previous-symbolic", 21),
                    text("Back").size(style::font_size::BODY)
                ]
                .spacing(style::spacing::TINY)
                .align_y(iced::Alignment::Center)
            )
            .on_press(Message::Back)
            .padding(8)
            .style(style::secondary_button),
            column![
                text(title).size(style::font_size::TITLE),
                text(subtitle)
                    .size(style::font_size::SMALL)
                    .style(style::muted_text),
            ]
            .spacing(style::spacing::TINY),
            space().width(if mobile_view {
                Length::Shrink
            } else {
                Length::Fill
            }),
            buttons_layout
        ]
        .spacing(style::spacing::LARGE)
        .padding(10)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .into()
    })
    .height(Length::Shrink)
    .into()
}

fn upsert_entry_view<'a>(
    entry: &'a InputableFreeTotpEntry,
    show_qr_code: bool,
) -> Element<'a, Message> {
    let button_text = if entry.uuid.is_some() {
        "Update Entry"
    } else {
        "Create Entry"
    };

    let form = column![
        // Name field
        column![
            text("Name")
                .size(style::font_size::BODY)
                .style(style::label_text),
            text_input("e.g., Google Account", &entry.name)
                .on_input(|v| Message::InputUpdated(TOTPEntryInput::UpdateName(v)))
                .padding(12)
                .size(style::font_size::MEDIUM)
        ]
        .spacing(style::spacing::TINY),
        // Secret field
        column![
            text("Secret Key")
                .size(style::font_size::BODY)
                .style(style::label_text),
            text_input("Secret", &entry.secret)
                .on_input(|v| Message::InputUpdated(TOTPEntryInput::UpdateSecret(v)))
                .padding(12)
                .size(style::font_size::MEDIUM)
        ]
        .spacing(style::spacing::TINY),
        // Two column layout for Issuer and Account Name
        row![
            column![
                text("Issuer (Optional)")
                    .size(style::font_size::BODY)
                    .style(style::label_text),
                text_input("e.g., GitHub", entry.issuer.as_deref().unwrap_or(""))
                    .on_input(|v| Message::InputUpdated(TOTPEntryInput::UpdateIssuer(v)))
                    .padding(12)
                    .size(style::font_size::MEDIUM)
            ]
            .spacing(style::spacing::TINY)
            .width(Length::FillPortion(1)),
            column![
                text("Account Name")
                    .size(style::font_size::BODY)
                    .style(style::label_text),
                text_input("e.g., user@example.com", &entry.account_name)
                    .on_input(|v| Message::InputUpdated(TOTPEntryInput::UpdateAccountName(v)))
                    .padding(12)
                    .size(style::font_size::MEDIUM)
            ]
            .spacing(style::spacing::TINY)
            .width(Length::FillPortion(1)),
        ]
        .spacing(style::spacing::MEDIUM),
        // Advanced settings section
        container(
            column![
                text("Advanced Settings").size(style::font_size::MEDIUM),
                // Three column layout for algorithm, digits, and period
                row![
                    column![
                        text("Algorithm")
                            .size(style::font_size::BODY)
                            .style(style::label_text),
                        pick_list(Some(&entry.algorithm), ALL_ALGORITHMS, |v: &_| v
                            .to_string())
                        .on_select(|v| Message::InputUpdated(TOTPEntryInput::UpdateAlgorithm(v)))
                        .width(Length::Fill)
                        .padding(12)
                    ]
                    .spacing(style::spacing::TINY)
                    .width(Length::FillPortion(1)),
                    column![
                        text("Digits")
                            .size(style::font_size::BODY)
                            .style(style::label_text),
                        text_input("6 or 8", &entry.digits.to_string())
                            .on_input(|v| Message::InputUpdated(TOTPEntryInput::UpdateDigits(v)))
                            .padding(12)
                            .size(style::font_size::MEDIUM)
                    ]
                    .spacing(style::spacing::TINY)
                    .width(Length::FillPortion(1)),
                    column![
                        text("Period")
                            .size(style::font_size::BODY)
                            .style(style::label_text),
                        text_input("30", &entry.step.to_string())
                            .on_input(|v| Message::InputUpdated(TOTPEntryInput::UpdateStep(v)))
                            .padding(12)
                            .size(style::font_size::MEDIUM)
                    ]
                    .spacing(style::spacing::TINY)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(style::spacing::MEDIUM),
            ]
            .spacing(style::spacing::MEDIUM)
        )
        .padding(16)
        .style(style::entry_card),
        // Submit button
        button(
            text(button_text)
                .size(style::font_size::MEDIUM)
                .width(Length::Fill)
                .align_x(Alignment::Center)
        )
        .on_press_maybe(entry.valid().then_some(Message::Submit))
        .padding(16)
        .width(Length::Fill)
        .style(style::primary_submit_button),
    ]
    .spacing(style::spacing::XLARGE)
    .padding(10)
    .max_width(600);

    let form_view = scrollable(container(form).center_x(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill);

    if show_qr_code {
        // QR code content - either image or error message
        let qr_content: Element<Message> = match &entry.get_qr_bytes() {
            Ok(bytes) => image(image::Handle::from_bytes(bytes.to_owned()))
                .width(Length::Fixed(400.0))
                .height(Length::Fixed(400.0))
                .into(),
            Err(e) => column![
                icons::get_icon("dialog-error-symbolic", 48),
                text("Failed to generate QR code").size(style::font_size::TITLE),
                text(e.to_string())
                    .size(style::font_size::BODY)
                    .style(style::muted_text),
            ]
            .spacing(style::spacing::MEDIUM)
            .align_x(iced::Alignment::Center)
            .into(),
        };

        // QR code overlay
        let qr_modal = container(
            column![
                // Close button
                button(
                    row![
                        icons::get_icon("window-close-symbolic", 21),
                        text("Close").size(style::font_size::BODY)
                    ]
                    .spacing(style::spacing::TINY)
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::ToggleShowQRCode)
                .padding(8)
                .style(style::secondary_button),
                // QR code image or error message
                qr_content,
            ]
            .spacing(style::spacing::MEDIUM)
            .padding(24),
        )
        .style(style::entry_card)
        .center(Length::Fill);

        stack![form_view, qr_modal].into()
    } else {
        form_view.into()
    }
}

//
// SUBSCRIPTIONS
//

#[derive(Debug, Clone)]
pub enum Hotkey {
    Tab(Modifiers),
    Esc,
}

fn handle_event(event: event::Event, _: event::Status, _: iced::window::Id) -> Option<Message> {
    #[allow(clippy::collapsible_match)]
    match event {
        event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
            Key::Named(Named::Tab) => Some(Message::Hotkey(Hotkey::Tab(modifiers))),
            Key::Named(Named::Escape) => Some(Message::Hotkey(Hotkey::Esc)),
            _ => None,
        },
        _ => None,
    }
}
