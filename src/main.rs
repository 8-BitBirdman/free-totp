// SPDX-License-Identifier: GPL-3.0-only
#![windows_subsystem = "windows"]

use crate::app::FreeTotp;

mod app;
mod config;
mod icons;

const APP_ID: &str = "io.github.8_bitbirdman.FreeTotp";
pub const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// SEE: https://github.com/pop-os/cosmic-bg/pull/73
/// Access glibc malloc tunables.
#[cfg(target_env = "gnu")]
mod malloc {
    use std::os::raw::c_int;
    const M_MMAP_THRESHOLD: c_int = -3;

    unsafe extern "C" {
        fn mallopt(param: c_int, value: c_int) -> c_int;
    }

    /// Prevents glibc from hoarding memory via memory fragmentation.
    pub fn limit_mmap_threshold() {
        unsafe {
            mallopt(M_MMAP_THRESHOLD, 65536);
        }
    }
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Prevents glibc from hoarding memory via memory fragmentation.
    #[cfg(target_env = "gnu")]
    malloc::limit_mmap_threshold();

    // Init the icon cache
    icons::ICON_CACHE.get_or_init(|| std::sync::Mutex::new(icons::IconCache::new()));

    let app_icon_data = include_bytes!("../resources/icons/hicolor/256x256/apps/icon.png");
    let app_icon = iced::window::icon::from_file_data(app_icon_data, None);

    // Init the tray icon
    let _tray_icon = {
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

    let platform_settings = {
        #[cfg(target_os = "linux")]
        {
            iced::window::settings::PlatformSpecific {
                application_id: String::from(APP_ID),
                ..Default::default()
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Default::default()
        }
    };

    iced::application::timed(
        FreeTotp::new,
        FreeTotp::update,
        FreeTotp::subscription,
        FreeTotp::view,
    )
    .title("FreeTotp")
    .theme(FreeTotp::theme)
    .window_size(iced::Size {
        width: 1100.,
        height: 700.,
    })
    .window(iced::window::Settings {
        min_size: Some(iced::Size {
            width: 360.,
            height: 500.,
        }),
        icon: app_icon.ok(),
        platform_specific: platform_settings,
        ..Default::default()
    })
    .run()
}
