use std::{fmt::Display, str::FromStr};

use embedded_graphics::pixelcolor::Rgb888;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

pub use animated_color::AnimatedColorSpec;
pub use static_color::StaticColorSpec;

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
            Ok(parsed_color) => Ok(parsed_color.into()),
            Err(e) => Err(e),
        }
    }
}

impl From<Color> for Rgb888 {
    fn from(c: Color) -> Self {
        Rgb888::new(c.r, c.g, c.b)
    }
}

impl From<colorgrad::Color> for Color {
    fn from(col: colorgrad::Color) -> Self {
        let [r, g, b, _] = col.to_rgba8();
        Self { r, g, b }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ColorSpec {
    Static(StaticColorSpec),
    Animated(AnimatedColorSpec),
}

mod static_color {
    use super::Color;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
    pub struct StaticColorSpec {
        pub color: Color,
    }
}

mod animated_color {
    use super::Color;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct UnvalidedAnimatedColorSpec {
        duration: usize,
        keyframes: Vec<(u8, Color)>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    #[serde(try_from = "UnvalidedAnimatedColorSpec")]
    pub struct AnimatedColorSpec {
        duration: usize,
        keyframes: Vec<(u8, Color)>,
    }

    impl AnimatedColorSpec {
        #[allow(unused)]
        pub fn new(duration: usize, keyframes: Vec<(u8, Color)>) -> Result<Self, String> {
            UnvalidedAnimatedColorSpec { duration, keyframes }.try_into()
        }

        pub fn duration(&self) -> usize {
            self.duration
        }

        pub fn keyframes(&self) -> &Vec<(u8, Color)> {
            &self.keyframes
        }
    }

    impl TryFrom<UnvalidedAnimatedColorSpec> for AnimatedColorSpec {
        type Error = String;

        fn try_from(input: UnvalidedAnimatedColorSpec) -> Result<Self, Self::Error> {
            let mut previous_key: Option<u8> = None;
            for (keyframe, _color) in &input.keyframes {
                if !(0..=100).contains(keyframe) {
                    return Err("Keyframe not in range [0,100]".to_string());
                }
                if let Some(prev) = previous_key
                    && *keyframe <= prev
                {
                    return Err("Keyframes must be strictly increasing".to_string());
                }
                previous_key = Some(*keyframe);
            }

            if !input.keyframes.iter().any(|pair| pair.0 == 100) {
                return Err("100 not found as a keyframe".to_string());
            }
            if !input.keyframes.iter().any(|pair| pair.0 == 0) {
                return Err("0 not found as a keyframe".to_string());
            }

            Ok(AnimatedColorSpec {
                duration: input.duration,
                keyframes: input.keyframes,
            })
        }
    }
}

pub mod util {
    use colorgrad::Gradient;

    use super::Color;
    pub type GradientBuilderError = colorgrad::GradientBuilderError;

    /// Make a gradient consisting of `num_steps` colors.
    /// Keyframes indicate what the color should be at defined steps in the gradient;
    /// the keys can be any arbitrary range.
    /// `num_steps` must be at least two, to account for the start/end points.
    pub fn mk_gradient(
        keyframes: &[(u8, Color)],
        num_steps: usize,
    ) -> Result<Vec<Color>, GradientBuilderError> {
        use colorgrad::{BlendMode, GradientBuilder, LinearGradient};

        let keys: Vec<f32> = keyframes.iter().map(|pair| f32::from(pair.0)).collect();
        let colors: Vec<colorgrad::Color> = keyframes
            .iter()
            .map(|pair| colorgrad::Color::from_rgba8(pair.1.r, pair.1.g, pair.1.b, 255))
            .collect();

        // TODO - allow changing blend mode through config option?
        let g = GradientBuilder::new()
            .mode(BlendMode::Oklab)
            .domain(&keys)
            .colors(&colors)
            .build::<LinearGradient>()?;

        let final_gradient: Vec<Color> = g.colors_iter(num_steps).map(Color::from).collect();
        assert!(final_gradient.len() >= 2);
        assert!(final_gradient[0] == keyframes[0].1);
        assert!(*final_gradient.last().unwrap() == keyframes.last().unwrap().1);

        Ok(final_gradient)
    }
}
