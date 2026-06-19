use ratatui::style::Color;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub base: Color,
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,
    pub overlay0: Color,
    pub overlay1: Color,
    pub overlay2: Color,
    pub subtext0: Color,
    pub subtext1: Color,
    pub text: Color,
    pub accent_blue: Color,
    pub accent_teal: Color,
    pub accent_yellow: Color,
    pub accent_green: Color,
    pub accent_red: Color,
    pub accent_orange: Color,
    pub accent_purple: Color,
    pub bg_dark: Color,
    pub bg_panel: Color,
    pub border: Color,
}

pub fn catppuccin_mocha() -> Theme {
    Theme {
        base: Color::Rgb(30, 30, 46),
        surface0: Color::Rgb(49, 50, 68),
        surface1: Color::Rgb(69, 71, 90),
        surface2: Color::Rgb(88, 91, 112),
        overlay0: Color::Rgb(108, 112, 134),
        overlay1: Color::Rgb(127, 132, 156),
        overlay2: Color::Rgb(147, 153, 178),
        subtext0: Color::Rgb(166, 173, 200),
        subtext1: Color::Rgb(186, 194, 222),
        text: Color::Rgb(205, 214, 244),
        accent_blue: Color::Rgb(137, 180, 250),
        accent_teal: Color::Rgb(148, 226, 213),
        accent_yellow: Color::Rgb(249, 226, 175),
        accent_green: Color::Rgb(166, 227, 161),
        accent_red: Color::Rgb(243, 139, 168),
        accent_orange: Color::Rgb(250, 179, 135),
        accent_purple: Color::Rgb(203, 166, 247),
        bg_dark: Color::Rgb(24, 24, 37),
        bg_panel: Color::Rgb(30, 30, 46),
        border: Color::Rgb(69, 71, 90),
    }
}

use std::sync::OnceLock;
static THEME: OnceLock<Theme> = OnceLock::new();

pub fn get() -> &'static Theme {
    THEME.get_or_init(catppuccin_mocha)
}
