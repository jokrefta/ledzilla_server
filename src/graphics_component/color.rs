use std::{fmt::Display, str::FromStr};

use embedded_graphics::pixelcolor::Rgb888;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(SerializeDisplay, DeserializeFromStr, Debug, PartialEq, Clone, Copy)]
pub struct ColorSpec {
    r: u8,
    g: u8,
    b: u8,
}

impl ColorSpec {
    pub fn try_from_rgb<T: TryInto<u8>>(r: T, g: T, b: T) -> Result<Self, T::Error> {
        Ok(Self::from_rgb(r.try_into()?, g.try_into()?, b.try_into()?))
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn as_rgb(&self) -> (u8, u8, u8) {
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

        if let Ok(r) = u8::from_str_radix(&s[1..=2], 16)
            && let Ok(g) = u8::from_str_radix(&s[3..=4], 16)
            && let Ok(b) = u8::from_str_radix(&s[5..=6], 16)
        {
            Ok(Self { r, g, b })
        } else {
            Err("An error occurred parsing color".to_string())
        }
    }
}

impl From<ColorSpec> for Rgb888 {
    fn from(c: ColorSpec) -> Self {
        Rgb888::new(c.r, c.g, c.b)
    }
}
