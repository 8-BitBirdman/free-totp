use iced::widget::{button, container, text};
use iced::{Border, Theme, Shadow, Vector};

/// Standard spacing values
pub mod spacing {
    pub const TINY: f32 = 6.0;
    pub const SMALL: f32 = 12.0;
    pub const MEDIUM: f32 = 16.0;
    pub const LARGE: f32 = 24.0;
    pub const XLARGE: f32 = 32.0;
}

/// Standard border radius values
pub mod radius {
    pub const SMALL: f32 = 8.0;
    pub const MEDIUM: f32 = 14.0;
    pub const LARGE: f32 = 20.0;
}

/// Standard font sizes
pub mod font_size {
    pub const SMALL: f32 = 13.0;
    pub const BODY: f32 = 15.0;
    pub const MEDIUM: f32 = 18.0;
    pub const LARGE: f32 = 20.0;
    pub const TITLE: f32 = 28.0;
    pub const HERO: f32 = 36.0;
}

/// Card container style - used for entry cards, form containers, etc.
pub fn card_container(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(theme.palette().background.base.color.into()),
        border: Border {
            color: theme.palette().background.weak.text.scale_alpha(0.05),
            width: 1.0,
            radius: radius::LARGE.into(),
        },
        shadow: Shadow {
            color: theme.palette().background.base.text.scale_alpha(0.05),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Entry card style - for TOTP entry items
pub fn entry_card(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(theme.palette().background.base.color.into()),
        border: Border {
            color: theme.palette().background.weak.text.scale_alpha(0.08),
            width: 1.0,
            radius: radius::MEDIUM.into(),
        },
        shadow: Shadow {
            color: theme.palette().background.base.text.scale_alpha(0.04),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Primary submit button style
pub fn primary_submit_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border = Border {
        radius: radius::MEDIUM.into(),
        ..Default::default()
    };
    style
}

/// Primary button style
pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border = Border {
        radius: radius::SMALL.into(),
        ..Default::default()
    };
    style
}

/// Secondary button style with rounded corners
pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    style.border = Border {
        radius: radius::SMALL.into(),
        ..Default::default()
    };
    style
}

/// Danger button style with rounded corners
pub fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border = Border {
        radius: radius::SMALL.into(),
        ..Default::default()
    };
    style
}

/// Success button style with rounded corners
pub fn success_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::success(theme, status);
    style.border = Border {
        radius: radius::SMALL.into(),
        ..Default::default()
    };
    style
}

/// Label text style (subdued color)
pub fn label_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().background.weak.text.scale_alpha(0.7)),
    }
}

/// Muted text style (for hints, subtitles, etc.)
pub fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().background.weak.text.scale_alpha(0.5)),
    }
}

/// Link text style (for clickable urls...)
pub fn link_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().primary.base.color),
    }
}

/// Subtitle text style (slightly muted)
pub fn subtitle_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().background.weak.text.scale_alpha(0.6)),
    }
}
