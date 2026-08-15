use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::{
    Drawable, geometry as eg_geo, geometry::Dimensions, image as eg_image, mono_font as eg_mono,
    prelude::Primitive, primitives as eg_prim, text as eg_text, transform::Transform,
};
use log::trace;
use thiserror::Error;

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::upload;
use color_state::ColorDrawState;

mod color_state;

mod movement;

/// A struct responsible for drawing a component and keeping track of its movement (e.g, scrolling) if configured.
///
/// Each component is turned into a ComponentDrawer which manages rendering for that component instance.
///
/// If scrolling is enabled for a particular component, there may be need to render multiple of them in one
/// frmae. For example, if the text "HELLO" is scrolling to the left (with a periodicity configured to match the
/// screen width), we might need to render the "ELLO" on the left edge of the screen as well as the "H"
/// on the right edge as it wraps around. This is handled by using a movement tracker to keep track of
/// how many copies of a component should be rendered, and where. This logic is kept separate from the
/// ComponentDrawer - which just knows how to render itself in a given location.
///
pub struct MovableComponentDrawer {
    drawer: ComponentDrawer,
    movement_tracker: Option<movement::ScrollingMovementTracker>,
}

impl MovableComponentDrawer {
    pub fn from_component(
        comp: super::Component,
        upload_manager: &Mutex<upload::UploadManager>,
        canvas_size: (u32, u32),
    ) -> Result<Self, DrawerCreationError> {
        let opt_motion_config = comp.get_motion_config().cloned();

        let drawer = ComponentDrawer::from_component(comp, upload_manager)?;

        let movement_tracker = if let Some(motion_config) = opt_motion_config {
            let bbox = drawer.get_bbox();
            let rendered_component_size = (bbox.size.width, bbox.size.height);
            let initial_position = (bbox.top_left.x, bbox.top_left.y);

            Some(
                movement::ScrollingMovementTracker::new(
                    movement::ScrollingMovementConfig::from_parsed_motion_config(
                        &motion_config,
                        canvas_size,
                        rendered_component_size,
                        initial_position,
                    ),
                )
                .map_err(DrawerCreationError::_GeneralError)?,
            )
        } else {
            None
        };

        Ok(Self {
            drawer,
            movement_tracker,
        })
    }

    pub fn is_static(&self) -> bool {
        self.movement_tracker.is_none() && self.drawer.is_static()
    }

    /// Draw frame and advance internal state by one frame.
    pub fn draw_next_frame<T>(&mut self, target: &mut T)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        if let Some(tracker) = &mut self.movement_tracker {
            tracker.for_each_instance(|offset| self.drawer.draw(target, offset));
            tracker.tick();
        } else {
            self.drawer.draw(target, (0, 0).into());
        }
        self.drawer.advance_frame();
    }

    pub fn get_cloned_component(&self) -> super::Component {
        self.drawer.get_cloned_component()
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
pub enum ComponentDrawer {
    Image(ImageDrawer),
    Line(LineDrawer),
    Text(TextDrawer),
    Rect(RectDrawer),
}

impl ComponentDrawer {
    pub fn from_component(
        comp: super::Component,
        upload_manager: &Mutex<upload::UploadManager>,
    ) -> Result<Self, DrawerCreationError> {
        match comp {
            super::Component::Image(image) => Ok(Self::Image(ImageDrawer::try_from_component(
                image,
                upload_manager,
            )?)),
            super::Component::Text(text) => Ok(Self::Text(TextDrawer::from(text))),
            super::Component::Line(line) => Ok(Self::Line(LineDrawer::from(line))),
            super::Component::Rectangle(rectangle) => Ok(Self::Rect(RectDrawer::from(rectangle))),
        }
    }

    pub fn draw<T>(&mut self, target: &mut T, offset: glam::IVec2)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        let offset_pt = eg_geo::Point::new(offset.x, offset.y);
        match self {
            Self::Image(drawer) => drawer.draw(target, offset_pt),
            Self::Line(drawer) => drawer.draw(target, offset_pt),
            Self::Text(drawer) => drawer.draw(target, offset_pt),
            Self::Rect(drawer) => drawer.draw(target, offset_pt),
        };
    }

    pub fn advance_frame(&mut self) {
        match self {
            Self::Image(drawer) => drawer.advance_frame(),
            Self::Line(drawer) => drawer.advance_frame(),
            Self::Text(drawer) => drawer.advance_frame(),
            Self::Rect(drawer) => drawer.advance_frame(),
        };
    }

    pub fn get_bbox(&self) -> eg_prim::Rectangle {
        match self {
            Self::Image(drawer) => drawer.get_bbox(),
            Self::Line(drawer) => drawer.get_bbox(),
            Self::Text(drawer) => drawer.get_bbox(),
            Self::Rect(drawer) => drawer.get_bbox(),
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

    /// Is the component static or dynamic (changing from frame to frame)?
    pub fn is_static(&self) -> bool {
        match self {
            Self::Image(drawer) => drawer.is_static(),
            Self::Line(drawer) => drawer.is_static(),
            Self::Text(drawer) => drawer.is_static(),
            Self::Rect(drawer) => drawer.is_static(),
        }
    }
}

pub struct LineDrawer {
    component: super::Line,
    color: ColorDrawState,
}

impl LineDrawer {
    pub fn draw<T>(&mut self, target: &mut T, offset: eg_geo::Point)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        self.get_styled_line().translate(offset).draw(target).unwrap();
    }

    pub fn get_bbox(&self) -> eg_prim::Rectangle {
        self.get_styled_line().bounding_box()
    }

    fn get_styled_line(&self) -> impl Drawable<Color = Rgb888> + Transform + Dimensions + use<> {
        let start = eg_geo::Point::new(self.component.x1, self.component.y1);
        let end = eg_geo::Point::new(self.component.x2, self.component.y2);
        trace!("Constructing Line(({})--({}))", start, end);
        eg_prim::Line::new(start, end).into_styled(eg_prim::PrimitiveStyle::with_stroke(
            self.color.get(),
            self.component.stroke_width,
        ))
    }

    fn advance_frame(&mut self) {
        self.color.advance_frame();
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
    pub fn draw<T>(&mut self, target: &mut T, offset: eg_geo::Point)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        self.get_styled_text().translate(offset).draw(target).unwrap();
    }

    pub fn get_bbox(&self) -> eg_prim::Rectangle {
        self.get_styled_text().bounding_box()
    }

    fn get_styled_text(&self) -> impl Drawable<Color = Rgb888> + Transform + Dimensions {
        let pos = eg_geo::Point::new(self.component.x, self.component.y);
        trace!("Constructing Text(pos {})", pos);
        let style = eg_mono::MonoTextStyle::new(self.component.font.get_eg_font(), self.color.get());
        eg_text::Text::with_alignment(
            &self.component.content,
            pos,
            style,
            self.component.alignment.into(),
        )
    }

    pub fn advance_frame(&mut self) {
        self.color.advance_frame();
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

const DEFAULT_GIF_SLOWDOWN: usize = 5;
impl ImageDrawer {
    pub fn draw<T>(&mut self, target: &mut T, offset: eg_geo::Point)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        let img_raw = self.get_eg_raw_img();
        let img = eg_image::Image::new(&img_raw, eg_geo::Point::new(self.component.x, self.component.y));
        img.translate(offset).draw(target).unwrap();
    }

    pub fn get_bbox(&self) -> eg_prim::Rectangle {
        let img_raw = self.get_eg_raw_img();
        eg_image::Image::new(&img_raw, eg_geo::Point::new(self.component.x, self.component.y)).bounding_box()
    }

    fn get_eg_raw_img(&self) -> eg_image::ImageRaw<'_, Rgb888> {
        match &*self.image_data {
            upload::UploadedAsset::Image(image_buf) => {
                eg_image::ImageRaw::<Rgb888>::new(image_buf.get_rgb_raw(), image_buf.get_width())
            }
            upload::UploadedAsset::AnimatedImage(animated_image_buf) => {
                let slowdown = match self.component.frame_slowdown {
                    Some(0) | None => DEFAULT_GIF_SLOWDOWN,
                    Some(x) => x,
                };

                let frames = animated_image_buf.get_frames();
                let next = &frames[self.frame_num / slowdown];
                eg_image::ImageRaw::<Rgb888>::new(next.get_rgb_raw(), animated_image_buf.get_width())
            }
        }
    }

    pub fn advance_frame(&mut self) {
        if let upload::UploadedAsset::AnimatedImage(animated_image_buf) = &*self.image_data {
            let slowdown = match self.component.frame_slowdown {
                Some(0) | None => DEFAULT_GIF_SLOWDOWN,
                Some(x) => x,
            };
            let led_frame_modulus = animated_image_buf.get_frames().len().saturating_mul(slowdown);

            self.frame_num = (self.frame_num + 1) % led_frame_modulus;
        };
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
    pub fn draw<T>(&mut self, target: &mut T, offset: eg_geo::Point)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        self.get_styled_rect().translate(offset).draw(target).unwrap();
    }

    pub fn get_bbox(&self) -> eg_prim::Rectangle {
        self.get_styled_rect().bounding_box()
    }

    fn get_styled_rect(&self) -> impl Drawable<Color = Rgb888> + Transform + Dimensions + use<> {
        let top_left = eg_geo::Point::new(self.component.x, self.component.y);
        let extent = eg_geo::Size::new(self.component.width, self.component.height);
        trace!("Constructing Rect(({}), size ({}))", top_left, extent);

        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_width(self.component.border_width)
            .stroke_color(self.border_color.get());
        if let Some(ref color) = self.fill_color {
            style_builder = style_builder.fill_color(color.get());
        }
        eg_prim::Rectangle::new(top_left, extent).into_styled(style_builder.build())
    }

    fn advance_frame(&mut self) {
        self.border_color.advance_frame();
        if let Some(ref mut color) = self.fill_color {
            color.advance_frame();
        }
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
