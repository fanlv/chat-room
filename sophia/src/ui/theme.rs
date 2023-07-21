use tui::style::Color;

pub struct Theme {
    pub message_colors: Vec<Color>,
    pub my_user_color: Color,
    pub date_color: Color,
    pub address_color: Color,
    pub system_info_color: (Color, Color),
    pub panel_border_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl Theme {
    pub fn dark_theme() -> Self {
        Self {
            message_colors: vec![
                Color::Yellow,
                Color::Magenta,
                Color::Rgb(255, 0, 255),
                Color::Rgb(0, 255, 255),
                Color::Rgb(255, 165, 0),
                Color::Rgb(153, 50, 205),
                Color::Rgb(153, 50, 205),
                Color::Rgb(255, 215, 255)],
            my_user_color: Color::Green,
            date_color: Color::DarkGray,
            address_color: Color::DarkGray,
            system_info_color: (Color::LightRed, Color::LightCyan),
            panel_border_color: Color::White,
        }
    }

    pub fn light_theme() -> Self {
        Self {
            message_colors: vec![Color::Black],
            my_user_color: Color::Green,
            date_color: Color::Black,
            address_color: Color::Black,
            system_info_color: (Color::LightRed, Color::LightCyan),
            panel_border_color: Color::Black,
        }
    }
}