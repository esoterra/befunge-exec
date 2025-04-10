use crossterm::style::{Attribute, Attributes, Color, ContentStyle};

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
