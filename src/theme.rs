use iced::overlay::menu;
use iced::widget::{
    button, container, pick_list, scrollable, slider, svg, text, text_input, toggler,
};
use iced::{application, color, Color};

#[derive(Debug, Clone, Copy, Default)]
pub struct Theme;

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: Color::from_rgb8(30, 30, 46),
            text_color: color!(205, 214, 244),
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub enum Text {
    #[default]
    Default,
    Peach,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
        Text::Default => text::Appearance {
            color: color!(205, 214, 244).into(),
        },
        Text::Peach => text::Appearance {
            color: color!(250, 179, 135).into(),
        },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Container {
    #[default]
    Default,
    BlackContainer,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Default => container::Appearance::default(),
            Container::BlackContainer => container::Appearance {
                background: Color::from_rgb8(49, 50, 68).into(),
                border_radius: 25.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Transparent,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Primary => button::Appearance {
                background: Color::from_rgb8(30, 102, 245).into(),
                border_radius: 15.0,
                border_width: 1.0,
                border_color: color!(0x45, 0x85, 0x88),
                ..Default::default()
            },
            Button::Secondary => button::Appearance {
                background: Color::from_rgb8(5, 194, 112).into(),
                border_radius: 15.0,
                ..Default::default()
            },
            Button::Transparent => button::Appearance {
                background: Color::TRANSPARENT.into(),
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Primary => button::Appearance {
                background: Color::from_rgb8(30, 102, 245).into(),
                border_radius: 15.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.90, 0.90, 0.90),
                ..Default::default()
            },
            Button::Secondary => button::Appearance {
                background: Color::from_rgb8(5, 194, 112).into(),
                border_radius: 15.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.90, 0.90, 0.90),
                ..Default::default()
            },
            Button::Transparent => button::Appearance {
                background: Color::from_rgb8(30, 30, 46).into(),
                border_radius: 100.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Default)]
pub enum TextInput {
    #[default]
    Default,
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            border_radius: 15.0,
            border_width: 1.0,
            border_color: Color::from_rgb8(49, 50, 68),
            icon_color: Color::from_rgb(0.90, 0.90, 0.90),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            border_radius: 15.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.90, 0.90, 0.90),
            icon_color: Color::from_rgb(0.90, 0.90, 0.90),
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            border_radius: 15.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.90, 0.90, 0.90),
            icon_color: Color::from_rgb(0.90, 0.90, 0.90),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.90, 0.90, 0.90)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.90, 0.90, 0.90)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.90, 0.90, 0.90)
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(Color::from_rgb(
                0x20 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x25 as f32 / 255.0,
            )),
            border_radius: 15.0,
            border_width: 1.0,
            border_color: Color::from_rgb(
                0x20 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x25 as f32 / 255.0,
            ),
            icon_color: Color::from_rgb(0.90, 0.90, 0.90),
        }
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        self.placeholder_color(style)
    }
}

#[derive(Clone, Default)]
pub enum PickList {
    #[default]
    Default,
}

impl pick_list::StyleSheet for Theme {
    type Style = PickList;

    fn active(&self, style: &Self::Style) -> pick_list::Appearance {
        match style {
            PickList::Default => pick_list::Appearance {
                text_color: Color::from_rgb(0.90, 0.90, 0.90),
                background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
                placeholder_color: Color::from_rgb(
                    0x20 as f32 / 255.0,
                    0x22 as f32 / 255.0,
                    0x25 as f32 / 255.0,
                ),
                handle_color: Color::from_rgb(0.90, 0.90, 0.90),
                border_radius: 15.0,
                border_width: 1.0,
                border_color: Color::from_rgb8(49, 50, 68),
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: Color::from_rgb(0.90, 0.90, 0.90),
            background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
            placeholder_color: Color::from_rgb(
                0x20 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x25 as f32 / 255.0,
            ),
            handle_color: Color::from_rgb(0.90, 0.90, 0.90),
            border_radius: 15.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.90, 0.90, 0.90),
        }
    }
}

#[derive(Default)]
pub enum Svg {
    #[default]
    Default,
}

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        match style {
            Svg::Default => Default::default(),
        }
    }
}

#[derive(Default)]
pub enum Slider {
    #[default]
    Default,
}

impl slider::StyleSheet for Theme {
    type Style = Slider;

    fn active(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
                let handle = slider::Handle {
                    shape: slider::HandleShape::Rectangle {
                        width: 8,
                        border_radius: 4.0,
                    },
                    color: Color::from_rgb8(30, 102, 245),
                    border_color: Color::from_rgb8(30, 102, 245),
                    border_width: 1.0,
                };

                slider::Appearance {
                    rail: slider::Rail {
                        colors: (
                            Color::from_rgb(0.90, 0.90, 0.90),
                            Color::from_rgb(0.90, 0.90, 0.90),
                        ),
                        width: 2.0,
                    },
                    handle: slider::Handle {
                        color: Color::from_rgb(
                            0x20 as f32 / 255.0,
                            0x22 as f32 / 255.0,
                            0x25 as f32 / 255.0,
                        ),
                        border_color: Color::from_rgb(0.90, 0.90, 0.90),
                        ..handle
                    },
                }
            }
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
                let active = self.active(style);

                slider::Appearance {
                    handle: slider::Handle {
                        color: Color::from_rgb8(30, 102, 245),
                        ..active.handle
                    },
                    ..active
                }
            }
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
                let active = self.active(style);

                slider::Appearance {
                    handle: slider::Handle {
                        color: Color::from_rgb8(30, 102, 245),
                        ..active.handle
                    },
                    ..active
                }
            }
        }
    }
}

#[derive(Clone, Default)]
pub enum Menu {
    #[default]
    Default,
}

impl menu::StyleSheet for Theme {
    type Style = Menu;

    fn appearance(&self, style: &Self::Style) -> menu::Appearance {
        match style {
            Menu::Default => menu::Appearance {
                text_color: Color::from_rgb(0.90, 0.90, 0.90),
                background: iced::Background::Color(Color::from_rgb8(49, 50, 68)),
                border_width: 0.0,
                border_radius: 15.0,
                border_color: Color::from_rgb(0.90, 0.90, 0.90),
                selected_text_color: Color::from_rgb(0.90, 0.90, 0.90),
                selected_background: iced::Background::Color(Color::from_rgb8(30, 102, 245)),
            },
        }
    }
}

impl From<PickList> for Menu {
    fn from(_pick_list: PickList) -> Self {
        Self::Default
    }
}

#[derive(Default)]
pub enum Scrollable {
    #[default]
    Default,
}

impl scrollable::StyleSheet for Theme {
    type Style = Scrollable;

    fn active(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => scrollable::Scrollbar {
                background: Some(iced::Background::Color(Color::from_rgb(
                    0x20 as f32 / 255.0,
                    0x22 as f32 / 255.0,
                    0x25 as f32 / 255.0,
                ))),
                border_radius: 15.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: Color::from_rgb(0.90, 0.90, 0.90),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => {
                if is_mouse_over_scrollbar {
                    scrollable::Scrollbar {
                        background: Some(iced::Background::Color(Color::from_rgb(
                            0x20 as f32 / 255.0,
                            0x22 as f32 / 255.0,
                            0x25 as f32 / 255.0,
                        ))),
                        border_radius: 15.0,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                        scroller: scrollable::Scroller {
                            color: Color::from_rgb(0.90, 0.90, 0.90),
                            border_radius: 2.0,
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        },
                    }
                } else {
                    self.active(style)
                }
            }
        }
    }

    fn dragging(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => self.hovered(style, true),
        }
    }

    fn active_horizontal(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => self.active(style),
        }
    }

    fn hovered_horizontal(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => self.hovered(style, is_mouse_over_scrollbar),
        }
    }

    fn dragging_horizontal(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Default => self.hovered_horizontal(style, true),
        }
    }
}

#[derive(Default)]
pub enum Toggler {
    #[default]
    Default,
}

impl toggler::StyleSheet for Theme {
    type Style = Toggler;

    fn active(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        match style {
            Toggler::Default => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(30, 102, 245)
                } else {
                    Color::from_rgb(0.90, 0.90, 0.90)
                },
                background_border: None,
                foreground: if is_active {
                    Color::from_rgb(0.90, 0.90, 0.90)
                } else {
                    Color::from_rgb(0.5, 0.5, 0.5)
                },
                foreground_border: None,
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        match style {
            Toggler::Default => toggler::Appearance {
                foreground: if is_active {
                    Color {
                        a: 0.5,
                        ..Color::from_rgb(0.90, 0.90, 0.90)
                    }
                } else {
                    Color::from_rgb(0.5, 0.5, 0.5)
                },
                ..self.active(style, is_active)
            },
        }
    }
}
