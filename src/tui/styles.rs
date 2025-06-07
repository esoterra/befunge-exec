use crossterm::style::{Attribute, Attributes, Color, ContentStyle};

use super::FocusedTab;

pub const BORDER: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    // background_color: Some(Color::Black),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none().with(Attribute::Bold),
};

pub const GREEN_HEADING: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Green),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none().with(Attribute::Bold),
};

pub const GREEN_HEADING_UNFOCUSED: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkGreen),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none()
        .with(Attribute::NoBold)
        .with(Attribute::NoUnderline),
};

pub const CYAN_HEADING: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Cyan),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none().with(Attribute::Bold),
};

pub const GRAY_HEADING: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Grey),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none()
        .with(Attribute::NoBold)
        .with(Attribute::NoUnderline),
};

pub const LOGO_OUTLINE: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Yellow),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none(),
};

pub const LOGO_EYES: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Cyan),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none(),
};

pub const PROGRAM_TEXT: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_EMPTY: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkGrey),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_QUOTED: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: Some(Color::DarkGrey),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_NORMAL: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_NUMBER: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Cyan),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_DIR: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Green),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::Bold),
};

pub const VISITED_STACK: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Magenta),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_IO: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Yellow),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_PG: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Magenta),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::NormalIntensity),
};

pub const VISITED_RED: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Red),
    background_color: Some(Color::Reset),
    underline_color: None,
    attributes: Attributes::none().with(Attribute::Bold),
};

pub fn tab_heading(current: FocusedTab, focused: FocusedTab) -> ContentStyle {
    if current == focused {
        GREEN_HEADING
    } else {
        GREEN_HEADING_UNFOCUSED
    }
}
