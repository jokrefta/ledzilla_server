use std::{fmt::Display, str::FromStr};

use embedded_graphics::pixelcolor::Rgb888;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ColorSpec {
    Static(StaticColorSpec),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct StaticColorSpec {
    pub color: Color,
}

#[derive(Debug, SerializeDisplay, DeserializeFromStr, PartialEq, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl FromStr for Color {
    type Err = colorgrad::ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Use html color parser from colorgrad to support many input formats
        match colorgrad::Color::from_html(s) {
            Ok(parsed_color) => {
                let [r, g, b, _] = parsed_color.to_rgba8();

                Ok(Self { r, g, b })
            }
            Err(e) => Err(e),
        }
    }
}

impl From<Color> for Rgb888 {
    fn from(c: Color) -> Self {
        Rgb888::new(c.r, c.g, c.b)
    }
}
