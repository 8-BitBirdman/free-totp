// SPDX-License-Identifier: GPL-3.0-only

use anywho::anywho;
use iced::Theme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub theme: FreeTotpTheme,
    #[serde(default)]
    pub stay_on_tray: bool,
}

impl Config {
    pub async fn load(app_id: &str) -> Result<Self, anywho::Error> {
        use dirs;
        use std::fs;

        let app_id = app_id.to_string();

        smol::unblock(move || {
            let config_dir = dirs::data_dir()
                .ok_or_else(|| anywho!("Could not determine config directory"))?
                .join(&app_id);

            // create config directory if it doesn't exist
            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)
                    .map_err(|e| anywho!("Failed to create config directory: {}", e))?;
            }

            let config_path = config_dir.join("config.ron");

            if config_path.exists() {
                let config_content = fs::read_to_string(&config_path)
                    .map_err(|e| anywho!("Failed to read config file: {}", e))?;

                ron::from_str(&config_content)
                    .map_err(|e| anywho!("Failed to parse config file: {}", e))
            } else {
                let config = Config::default();

                // Save default config
                let config_content =
                    ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default())
                        .map_err(|e| anywho!("Failed to serialize config: {}", e))?;

                fs::write(&config_path, config_content)
                    .map_err(|e| anywho!("Failed to write config file: {}", e))?;

                Ok(config)
            }
        })
        .await
        .map_err(|e| anywho!("Error loading config: {}", e))
    }

    pub async fn save(self, app_id: &str) -> Result<(), anywho::Error> {
        use dirs;
        use std::fs;

        let config_clone = self.clone();
        let app_id = app_id.to_string();

        smol::unblock(move || {
            let config_dir = dirs::data_dir()
                .ok_or_else(|| anywho!("Could not determine config directory"))?
                .join(&app_id);

            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)
                    .map_err(|e| anywho!("Failed to create config directory: {}", e))?;
            }

            let config_path = config_dir.join("config.ron");

            let config_content =
                ron::ser::to_string_pretty(&config_clone, ron::ser::PrettyConfig::default())
                    .map_err(|e| anywho!("Failed to serialize config: {}", e))?;

            fs::write(&config_path, config_content)
                .map_err(|e| anywho!("Failed to write config file: {}", e))?;

            Ok(())
        })
        .await
        .map_err(|e: anywho::Error| anywho!("Unblock error: {}", e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum FreeTotpTheme {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    #[default]
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

impl From<FreeTotpTheme> for Theme {
    fn from(config_theme: FreeTotpTheme) -> Self {
        match config_theme {
            FreeTotpTheme::Light => Theme::Light,
            FreeTotpTheme::Dark => Theme::Dark,
            FreeTotpTheme::Dracula => Theme::Dracula,
            FreeTotpTheme::Nord => Theme::Nord,
            FreeTotpTheme::SolarizedLight => Theme::SolarizedLight,
            FreeTotpTheme::SolarizedDark => Theme::SolarizedDark,
            FreeTotpTheme::GruvboxLight => Theme::GruvboxLight,
            FreeTotpTheme::GruvboxDark => Theme::GruvboxDark,
            FreeTotpTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            FreeTotpTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            FreeTotpTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            FreeTotpTheme::CatppuccinMocha => Theme::CatppuccinMocha,
            FreeTotpTheme::TokyoNight => Theme::TokyoNight,
            FreeTotpTheme::TokyoNightStorm => Theme::TokyoNightStorm,
            FreeTotpTheme::TokyoNightLight => Theme::TokyoNightLight,
            FreeTotpTheme::KanagawaWave => Theme::KanagawaWave,
            FreeTotpTheme::KanagawaDragon => Theme::KanagawaDragon,
            FreeTotpTheme::KanagawaLotus => Theme::KanagawaLotus,
            FreeTotpTheme::Moonfly => Theme::Moonfly,
            FreeTotpTheme::Nightfly => Theme::Nightfly,
            FreeTotpTheme::Oxocarbon => Theme::Oxocarbon,
            FreeTotpTheme::Ferra => Theme::Ferra,
        }
    }
}

/// Will fail for custom themes
impl TryFrom<&Theme> for FreeTotpTheme {
    type Error = &'static str;

    fn try_from(theme: &Theme) -> Result<Self, Self::Error> {
        match theme {
            Theme::Light => Ok(FreeTotpTheme::Light),
            Theme::Dark => Ok(FreeTotpTheme::Dark),
            Theme::Dracula => Ok(FreeTotpTheme::Dracula),
            Theme::Nord => Ok(FreeTotpTheme::Nord),
            Theme::SolarizedLight => Ok(FreeTotpTheme::SolarizedLight),
            Theme::SolarizedDark => Ok(FreeTotpTheme::SolarizedDark),
            Theme::GruvboxLight => Ok(FreeTotpTheme::GruvboxLight),
            Theme::GruvboxDark => Ok(FreeTotpTheme::GruvboxDark),
            Theme::CatppuccinLatte => Ok(FreeTotpTheme::CatppuccinLatte),
            Theme::CatppuccinFrappe => Ok(FreeTotpTheme::CatppuccinFrappe),
            Theme::CatppuccinMacchiato => Ok(FreeTotpTheme::CatppuccinMacchiato),
            Theme::CatppuccinMocha => Ok(FreeTotpTheme::CatppuccinMocha),
            Theme::TokyoNight => Ok(FreeTotpTheme::TokyoNight),
            Theme::TokyoNightStorm => Ok(FreeTotpTheme::TokyoNightStorm),
            Theme::TokyoNightLight => Ok(FreeTotpTheme::TokyoNightLight),
            Theme::KanagawaWave => Ok(FreeTotpTheme::KanagawaWave),
            Theme::KanagawaDragon => Ok(FreeTotpTheme::KanagawaDragon),
            Theme::KanagawaLotus => Ok(FreeTotpTheme::KanagawaLotus),
            Theme::Moonfly => Ok(FreeTotpTheme::Moonfly),
            Theme::Nightfly => Ok(FreeTotpTheme::Nightfly),
            Theme::Oxocarbon => Ok(FreeTotpTheme::Oxocarbon),
            Theme::Ferra => Ok(FreeTotpTheme::Ferra),
            Theme::Custom(_) => Err("Custom themes cannot be converted to ConfigTheme"),
        }
    }
}
