use std::{fmt::Display, str::FromStr};

use embedded_graphics::pixelcolor::Rgb888;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(SerializeDisplay, DeserializeFromStr, Debug, PartialEq, Clone, Copy)]
pub struct ColorSpec {
    r: u32,
    g: u32,
    b: u32,
}

impl ColorSpec {
    pub fn from_rgb(r: u32, g: u32, b: u32) -> Result<Self, String> {
        if r < 256 && g < 256 && b < 256 {
            Ok(Self { r, g, b })
        } else {
            Err("RGB value out of range".to_string())
        }
    }

    pub fn as_rgb(&self) -> (u32, u32, u32) {
        (self.r, self.g, self.b)
    }
}

impl Display for ColorSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl FromStr for ColorSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Assume hex string, e.g. "#FFffFF"
        if !s.is_ascii() || s.len() != 7 {
            return Err("Bad - not an ascii string of length 7".to_string());
        }
        if s.chars().nth(0) != Some('#') {
            return Err("Bad - does not start with #".to_string());
        }

        if let Ok(r) = u32::from_str_radix(&s[1..=2], 16)
            && let Ok(g) = u32::from_str_radix(&s[3..=4], 16)
            && let Ok(b) = u32::from_str_radix(&s[5..=6], 16)
        {
            Ok(Self { r, g, b })
        } else {
            Err("An error occurred parsing color".to_string())
        }
    }
}

impl Into<Rgb888> for ColorSpec {
    fn into(self) -> Rgb888 {
        todo!()
    }
}
