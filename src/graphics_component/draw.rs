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
    fn get(&self) -> Rgb888 {
        match self {
            Self::Static(static_draw_state) => static_draw_state.get(),
            Self::Animated(animated_draw_state) => animated_draw_state.get(),
        }
    }

    fn advance_frame(&mut self) {
        match self {
            Self::Static(_) => (),
            Self::Animated(animated_color_draw_state) => animated_color_draw_state.advance_frame(),
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

mod movement {
    use std::collections::VecDeque;

    fn vec2_from_direction(direction_degrees: u16) -> glam::Vec2 {
        match direction_degrees {
            0 => (1.0, 0.0).into(),
            90 => (0.0, 1.0).into(),
            180 => (-1.0, 0.0).into(),
            270 => (0.0, -1.0).into(),
            _ => glam::Vec2::from_angle(f32::from(direction_degrees).to_radians()),
        }
    }

    pub struct ScrollingMovementConfig {
        distance_per_tick: u32,
        direction_degrees: u16,
        periodicity: u32,
        canvas_size: (u32, u32),
        rendered_component_size: (u32, u32),
        initial_pos: glam::IVec2,
    }

    impl ScrollingMovementConfig {
        /// Arguments:
        /// * `motion_config`: Motion configuration parsed from the API component spec
        /// * `canvas_size`: (width, height)
        /// * `rendered_component_size`: (width, height)
        /// * `initial_pos`: (x, y) of TOP LEFT of object's bounding box for its initial
        ///   placement. All calculations and offsets will be relative to this point.
        pub fn from_parsed_motion_config(
            motion_config: &crate::graphics_component::MotionConfig,
            canvas_size: (u32, u32),
            rendered_component_size: (u32, u32),
            initial_position: (i32, i32),
        ) -> Self {
            Self {
                distance_per_tick: motion_config.distance_per_tick,
                direction_degrees: motion_config.direction_degrees,
                periodicity: motion_config.periodicity,
                canvas_size,
                rendered_component_size,
                initial_pos: initial_position.into(),
            }
        }
    }

    pub struct ScrollingMovementTracker {
        // direction: direction::CardinalDirection,
        translation_per_tick: glam::Vec2,
        /// Relative to the initial instance position
        displayable_x_bounds: (f32, f32),
        /// Relative to the initial instance position
        displayable_y_bounds: (f32, f32),
        repetition_offset: glam::Vec2,

        /// Offsets of all instances (duplicate copies) of the component. These are all relative
        /// to the initial position of the object. There may only be one of these, if the scroll
        /// periodicity is high enough that only one copy of the object is on screen at any time.
        ///
        /// The vec of offsets is kept sorted so that the instaces at the "leading edge"
        /// of the motion are at the end of the vec, and the instances at the "trailing" edge
        /// are at the start.
        current_offsets: VecDeque<glam::Vec2>,
    }

    impl ScrollingMovementTracker {
        pub fn new(config: ScrollingMovementConfig) -> Result<Self, String> {
            if config.distance_per_tick > config.periodicity {
                // not supported by tick() currently
                return Err("Unsupported when distance_per_tick > periodicity".to_string());
            }

            let mut current_offsets: VecDeque<glam::prelude::Vec2> = VecDeque::new();
            let translation_per_tick =
                vec2_from_direction(config.direction_degrees) * config.distance_per_tick as f32;

            let repetition_offset = vec2_from_direction(config.direction_degrees) * config.periodicity as f32;

            // The coordinate plane is shifted to be relative to the initial component position for
            // all calculations.
            let displayable_x_bounds = (
                -(config.rendered_component_size.0 as f32) - config.initial_pos.x as f32,
                config.canvas_size.0 as f32 - config.initial_pos.x as f32,
            );
            let displayable_y_bounds = (
                -(config.rendered_component_size.1 as f32) - config.initial_pos.y as f32,
                config.canvas_size.1 as f32 - config.initial_pos.y as f32,
            );

            // prepend all instances that come "before" the initial position
            for i in 1.. {
                let instance = -repetition_offset * i as f32;
                if is_instance_oob(displayable_x_bounds, displayable_y_bounds, instance) {
                    break; // We've gone past the end
                }
                log::trace!("Pushing offset {} to front", instance);
                current_offsets.push_front(instance);
            }
            // append the initial position and all instances that come "after" it
            for i in 0.. {
                let instance = repetition_offset * i as f32;
                if is_instance_oob(displayable_x_bounds, displayable_y_bounds, instance) {
                    break; // We've gone past the end
                }
                log::trace!("Pushing offset {} to back", instance);
                current_offsets.push_back(instance);
            }
            log::debug!("ScrollingMovementTracker offsets to start: {:?}", current_offsets);
            log::debug!(
                "ScrollingMovementTracker bounds: {:?} / {:?}",
                displayable_x_bounds,
                displayable_y_bounds
            );

            Ok(Self {
                translation_per_tick,
                displayable_x_bounds,
                displayable_y_bounds,
                repetition_offset,
                current_offsets,
            })
        }

        /// Update current offsets
        pub fn tick(&mut self) {
            // Update existing instance offsets
            for v in self.current_offsets.iter_mut() {
                *v += self.translation_per_tick;
            }
            // log::trace!( "ScrollingMovementTracker offsets after update: {:?}", self.current_offsets);

            /*
             * If one has moved off screen, delete it. If it's time to add one on screen, create it.
             *
             * HOWEVER, never delete if it's the only instance left. We need at least one coordinate so
             * we have a reference point for creating the next one.
             * This means if the periodicity is large enough, there may be zero instances on the
             * screen for some frames, but self.current_offsets will always have at least one element.
             *
             * This assumes that only one will go off screen in a given tick - which is a safe
             * assumption iff the repetition separation is bigger than the distance per tick.
             */
            assert!(self.translation_per_tick.length_squared() < self.repetition_offset.length_squared());
            if self.current_offsets.len() > 1
                && is_instance_oob(
                    self.displayable_x_bounds,
                    self.displayable_y_bounds,
                    *self.current_offsets.back().unwrap(),
                )
            {
                log::trace!("ScrollingMovementTracker - Destroy instance!");
                self.current_offsets.pop_back();
            }

            let potential_new_spawn = self.current_offsets.front().unwrap() - self.repetition_offset;
            // log::trace!("ScrollingMovementTracker - testing potential new spawn {}", potential_new_spawn);
            if !is_instance_oob(
                self.displayable_x_bounds,
                self.displayable_y_bounds,
                potential_new_spawn,
            ) {
                log::trace!("ScrollingMovementTracker - Spawn new!");
                self.current_offsets.push_front(potential_new_spawn);
            }
        }

        /// Gets offsets for each instance of the component.
        /// These are rounded to integer coordinates for rendering.
        pub fn get_offsets(&self) -> Vec<glam::IVec2> {
            self.current_offsets
                .iter()
                .map(|v| v.round().as_ivec2())
                .collect()
        }

        /// Calls the provided function `f` once for each current component instance.
        /// `f` takes a single argument, the offset (relative to initial component position)
        /// of the instance.
        pub fn for_each_instance<F>(&self, mut f: F)
        where
            F: FnMut(glam::IVec2),
        {
            for pos in self.get_offsets() {
                f(pos)
            }
        }
    }

    /// Check if an instance of the drawn component is out of bounds
    fn is_instance_oob(
        drawable_x_bounds: (f32, f32),
        drawable_y_bounds: (f32, f32),
        instance_pos: glam::Vec2,
    ) -> bool {
        instance_pos.x < drawable_x_bounds.0
            || instance_pos.x > drawable_x_bounds.1
            || instance_pos.y < drawable_y_bounds.0
            || instance_pos.y > drawable_y_bounds.1
    }
}

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
