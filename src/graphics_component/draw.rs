use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::{
    Drawable, geometry as eg_geo, image as eg_image, mono_font as eg_mono, prelude::Primitive,
    primitives as eg_prim, text as eg_text,
};
use log::trace;
use thiserror::Error;

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::graphics_component::ColorSpec;
use crate::graphics_component::color;
use crate::upload;

#[derive(Debug)]
enum ColorDrawState {
    Static(StaticColorDrawState),
    Animated(AnimatedColorDrawState),
}

#[derive(Debug)]
struct StaticColorDrawState {
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
    fn get_next(&mut self) -> Rgb888 {
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
struct AnimatedColorDrawState {
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
    fn get_next(&mut self) -> Rgb888 {
        let next = self.color_steps[self.cur_step];
        self.cur_step = (self.cur_step + 1) % self.color_steps.len();
        next.into()
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
    fn get_next(&mut self) -> Rgb888 {
        match self {
            ColorDrawState::Static(static_draw_state) => static_draw_state.get_next(),
            ColorDrawState::Animated(animated_draw_state) => animated_draw_state.get_next(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DrawerCreationError {
    #[error("Failed to construct drawer due to bad component spec: {0}")]
    BadComponentSpec(String),
    #[error("{0}")]
    _GeneralError(String),
}

// Draw-er as in "one who draws", not the furniture
// TODO maybe box to reduce enum size? Idk probably unneeded, should probably profile first or something
pub enum ComponentDrawer {
    Image(ImageDrawer),
    Line(LineDrawer),
    Text(TextDrawer),
    Rect(RectDrawer),
}

impl ComponentDrawer {
    pub fn draw_next_frame<T>(&mut self, target: &mut T)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        match self {
            Self::Image(drawer) => drawer.draw_next_frame(target),
            Self::Line(drawer) => drawer.draw_next_frame(target),
            Self::Text(drawer) => drawer.draw_next_frame(target),
            Self::Rect(drawer) => drawer.draw_next_frame(target),
        }
    }

    pub fn get_cloned_component(&self) -> super::Component {
        match self {
            Self::Image(drawer) => drawer.get_cloned_component(),
            Self::Line(drawer) => drawer.get_cloned_component(),
            Self::Text(drawer) => drawer.get_cloned_component(),
            Self::Rect(drawer) => drawer.get_cloned_component(),
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Self::Image(drawer) => drawer.is_static(),
            Self::Line(drawer) => drawer.is_static(),
            Self::Text(drawer) => drawer.is_static(),
            Self::Rect(drawer) => drawer.is_static(),
        }
    }
}

pub fn into_drawer(
    comp: super::Component,
    upload_manager: &Mutex<upload::UploadManager>,
) -> Result<ComponentDrawer, DrawerCreationError> {
    match comp {
        super::Component::Image(image) => Ok(ComponentDrawer::Image(ImageDrawer::try_from_component(
            image,
            upload_manager,
        )?)),
        super::Component::Text(text) => Ok(ComponentDrawer::Text(TextDrawer::from(text))),
        super::Component::Line(line) => Ok(ComponentDrawer::Line(LineDrawer::from(line))),
        super::Component::Rectangle(rectangle) => Ok(ComponentDrawer::Rect(RectDrawer::from(rectangle))),
    }
}

pub struct LineDrawer {
    component: super::Line,
    color: ColorDrawState,
}

impl LineDrawer {
    pub fn draw_next_frame<T>(&mut self, target: &mut T)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        let start = eg_geo::Point::new(self.component.x1, self.component.y1);
        let end = eg_geo::Point::new(self.component.x2, self.component.y2);
        trace!("Drawing Line(({})--({}))", start, end);
        eg_prim::Line::new(start, end)
            .into_styled(eg_prim::PrimitiveStyle::with_stroke(
                self.color.get_next(),
                self.component.stroke_width,
            ))
            .draw(target)
            .unwrap();
    }

    pub fn get_cloned_component(&self) -> super::Component {
        super::Component::Line(self.component.clone())
    }

    pub fn is_static(&self) -> bool {
        if let ColorDrawState::Animated(..) = self.color {
            return false;
        }
        true
    }
}

impl From<super::Line> for LineDrawer {
    fn from(component: super::Line) -> Self {
        let color: ColorDrawState = component.color.clone().into();
        Self { component, color }
    }
}

pub struct TextDrawer {
    component: super::Text,
    color: ColorDrawState,
}

impl TextDrawer {
    pub fn draw_next_frame<T>(&mut self, target: &mut T)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        let pos = eg_geo::Point::new(self.component.x, self.component.y);
        trace!("Drawing Text(pos {})", pos);
        let style = eg_mono::MonoTextStyle::new(self.component.font.get_eg_font(), self.color.get_next());
        eg_text::Text::with_alignment(
            &self.component.content,
            pos,
            style,
            self.component.alignment.into(),
        )
        .draw(target)
        .unwrap();
    }

    pub fn get_cloned_component(&self) -> super::Component {
        super::Component::Text(self.component.clone())
    }

    pub fn is_static(&self) -> bool {
        if let ColorDrawState::Animated(..) = self.color {
            return false;
        }
        true
    }
}

impl From<super::Text> for TextDrawer {
    fn from(component: super::Text) -> Self {
        let color: ColorDrawState = component.color.clone().into();
        Self { component, color }
    }
}

pub struct ImageDrawer {
    component: super::Image,
    image_data: Arc<upload::UploadedAsset>,
    frame_num: usize, // Only used for animated image. Counts LED frames, not GIF frames
}

impl ImageDrawer {
    pub fn draw_next_frame<T>(&mut self, target: &mut T)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        let pos = eg_geo::Point::new(self.component.x, self.component.y);

        let raw_image = match &*self.image_data {
            upload::UploadedAsset::Image(image_buf) => {
                eg_image::ImageRaw::<Rgb888>::new(image_buf.get_rgb_raw(), image_buf.get_width())
            }
            upload::UploadedAsset::AnimatedImage(animated_image_buf) => {
                let frames = animated_image_buf.get_frames();

                let slowdown = match self.component.frame_slowdown {
                    Some(0) | None => 5, // a reasonable default
                    Some(x) => x,
                };
                let led_frame_modulus = frames.len().saturating_mul(slowdown);

                let next = &frames[self.frame_num / slowdown];
                self.frame_num = (self.frame_num + 1) % led_frame_modulus;

                eg_image::ImageRaw::<Rgb888>::new(next.get_rgb_raw(), animated_image_buf.get_width())
            }
        };

        let image = eg_image::Image::new(&raw_image, pos);

        image.draw(target).unwrap();
    }

    pub fn get_cloned_component(&self) -> super::Component {
        super::Component::Image(self.component.clone())
    }

    pub fn is_static(&self) -> bool {
        if let upload::UploadedAsset::AnimatedImage(..) = *self.image_data {
            return false;
        }
        true
    }
}

impl ImageDrawer {
    pub fn try_from_component(
        component: super::Image,
        upload_manager: &Mutex<upload::UploadManager>,
    ) -> Result<Self, DrawerCreationError> {
        let upload_manager = upload_manager.lock().unwrap();
        if let Some(image_data) = upload_manager.retrieve(&component.source) {
            Ok(Self {
                component,
                image_data,
                frame_num: 0,
            })
        } else {
            Err(DrawerCreationError::BadComponentSpec(
                "Could not find filename in database".to_string(),
            ))
        }
    }
}

pub struct RectDrawer {
    component: super::Rectangle,
    border_color: ColorDrawState,
    fill_color: Option<ColorDrawState>,
}

impl RectDrawer {
    pub fn draw_next_frame<T>(&mut self, target: &mut T)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        let top_left = eg_geo::Point::new(self.component.x, self.component.y);
        let extent = eg_geo::Size::new(self.component.width, self.component.height);
        trace!("Drawing Rect(({}), size ({}))", top_left, extent);

        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_width(self.component.border_width)
            .stroke_color(self.border_color.get_next());
        if let Some(ref mut color) = self.fill_color {
            style_builder = style_builder.fill_color(color.get_next());
        }
        eg_prim::Rectangle::new(top_left, extent)
            .into_styled(style_builder.build())
            .draw(target)
            .unwrap();
    }

    pub fn get_cloned_component(&self) -> super::Component {
        super::Component::Rectangle(self.component.clone())
    }

    pub fn is_static(&self) -> bool {
        if let ColorDrawState::Animated(..) = self.border_color {
            return false;
        }
        if let Some(ColorDrawState::Animated(..)) = self.fill_color {
            return false;
        }
        true
    }
}

impl From<super::Rectangle> for RectDrawer {
    fn from(component: super::Rectangle) -> Self {
        let fill_color = component.fill_color.clone().map(ColorDrawState::from);
        let border_color = ColorDrawState::from(component.border_color.clone());
        Self {
            component,
            border_color,
            fill_color,
        }
    }
}
