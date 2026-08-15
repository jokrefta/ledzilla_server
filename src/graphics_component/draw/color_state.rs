use embedded_graphics::pixelcolor::Rgb888;

use crate::graphics_component::ColorSpec;
use crate::graphics_component::color;

#[derive(Debug)]
pub enum ColorDrawState {
    Static(StaticColorDrawState),
    Animated(AnimatedColorDrawState),
}

#[derive(Debug)]
pub struct StaticColorDrawState {
    color: color::Color,
}

impl From<color::StaticColorSpec> for StaticColorDrawState {
    fn from(colorspec: color::StaticColorSpec) -> Self {
        Self {
            color: colorspec.color,
        }
    }
}

impl StaticColorDrawState {
    fn get(&self) -> Rgb888 {
        self.color.into()
    }
}

/// Holds the state of the color animation.
/// All steps in the animation sequence are computed at construction, so we can
/// quickly grab the next one when needed.
///
/// Do we need to store a separate color for every single frame? Probably not.
/// A future optimization might be only updating color, say, every 4 frames.
/// This would reduce the number of animation points and the user wouldn't be
/// able to tell the difference
#[derive(Debug)]
pub struct AnimatedColorDrawState {
    color_steps: Vec<color::Color>,
    cur_step: usize,
}

impl TryFrom<color::AnimatedColorSpec> for AnimatedColorDrawState {
    type Error = color::util::GradientBuilderError;

    fn try_from(colorspec: color::AnimatedColorSpec) -> Result<Self, Self::Error> {
        let gradient = color::util::mk_gradient(&colorspec.keyframes, colorspec.duration)?;
        Ok(Self {
            color_steps: gradient,
            cur_step: 0,
        })
    }
}

impl AnimatedColorDrawState {
    fn get(&self) -> Rgb888 {
        self.color_steps[self.cur_step].into()
    }

    fn advance_frame(&mut self) {
        self.cur_step = (self.cur_step + 1) % self.color_steps.len();
    }
}

impl From<ColorSpec> for ColorDrawState {
    fn from(colorspec: ColorSpec) -> Self {
        match colorspec {
            ColorSpec::Static(spec) => Self::Static(spec.into()),
            // Creation of the animated color state is fallible, but really should never fail.
            // Validation is done when deserializing the animated color spec which should guarantee a valid
            // configuration.
            ColorSpec::Animated(spec) => Self::Animated(spec.try_into().unwrap()),
        }
    }
}

impl ColorDrawState {
    pub fn get(&self) -> Rgb888 {
        match self {
            Self::Static(static_draw_state) => static_draw_state.get(),
            Self::Animated(animated_draw_state) => animated_draw_state.get(),
        }
    }

    pub fn advance_frame(&mut self) {
        match self {
            Self::Static(_) => (),
            Self::Animated(animated_color_draw_state) => animated_color_draw_state.advance_frame(),
        }
    }
}
