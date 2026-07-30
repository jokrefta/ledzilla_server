use embedded_graphics::text;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl From<Alignment> for text::Alignment {
    fn from(value: Alignment) -> Self {
        match value {
            Alignment::Left => text::Alignment::Left,
            Alignment::Center => text::Alignment::Center,
            Alignment::Right => text::Alignment::Right,
        }
    }
}

use embedded_graphics::mono_font::ascii;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Font {
    mono_default_4x6,
    mono_default_5x7,
    mono_default_5x8,
    mono_default_6x10,
    mono_default_6x12,
    mono_default_6x13,
    mono_default_6x13_bold,
    mono_default_6x13_italic,
    mono_default_6x9,
    mono_default_7x13,
    mono_default_7x13_bold,
    mono_default_7x13_italic,
    mono_default_7x14,
    mono_default_7x14_bold,
    mono_default_8x13,
    mono_default_8x13_bold,
    mono_default_8x13_italic,
    mono_default_9x15,
    mono_default_9x15_bold,
    mono_default_9x18,
    mono_default_9x18_bold,
    mono_default_10x20,
}

impl Font {
    pub fn get_eg_font(&self) -> &'static embedded_graphics::mono_font::MonoFont<'static> {
        match self {
            Self::mono_default_4x6 => &ascii::FONT_4X6,
            Self::mono_default_5x7 => &ascii::FONT_5X7,
            Self::mono_default_5x8 => &ascii::FONT_5X8,
            Self::mono_default_6x10 => &ascii::FONT_6X10,
            Self::mono_default_6x12 => &ascii::FONT_6X12,
            Self::mono_default_6x13 => &ascii::FONT_6X13,
            Self::mono_default_6x13_bold => &ascii::FONT_6X13_BOLD,
            Self::mono_default_6x13_italic => &ascii::FONT_6X13_ITALIC,
            Self::mono_default_6x9 => &ascii::FONT_6X9,
            Self::mono_default_7x13 => &ascii::FONT_7X13,
            Self::mono_default_7x13_bold => &ascii::FONT_7X13_BOLD,
            Self::mono_default_7x13_italic => &ascii::FONT_7X13_ITALIC,
            Self::mono_default_7x14 => &ascii::FONT_7X14,
            Self::mono_default_7x14_bold => &ascii::FONT_7X14_BOLD,
            Self::mono_default_8x13 => &ascii::FONT_8X13,
            Self::mono_default_8x13_bold => &ascii::FONT_8X13_BOLD,
            Self::mono_default_8x13_italic => &ascii::FONT_8X13_ITALIC,
            Self::mono_default_9x15 => &ascii::FONT_9X15,
            Self::mono_default_9x15_bold => &ascii::FONT_9X15_BOLD,
            Self::mono_default_9x18 => &ascii::FONT_9X18,
            Self::mono_default_9x18_bold => &ascii::FONT_9X18_BOLD,
            Self::mono_default_10x20 => &ascii::FONT_10X20,
        }
    }
}
