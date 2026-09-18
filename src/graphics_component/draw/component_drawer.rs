use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::{
    Drawable, geometry as eg_geo, geometry::Dimensions, image as eg_image, prelude::Primitive,
    primitives as eg_prim, text as eg_text, transform::Transform,
};
use log::trace;

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::DrawerCreationError;
use super::color_state::ColorDrawState;
use crate::graphics_component::{Component, Image, Line, Plot, Rectangle, Text, text_string};
use crate::upload;

/* Keeping this in case it's useful
fn bbox_union(a: eg_prim::Rectangle, b: eg_prim::Rectangle) -> eg_prim::Rectangle {
    use eg_geo::{AnchorX, AnchorY};
    eg_prim::Rectangle::with_corners(
        (
            a.anchor_x(AnchorX::Left).min(b.anchor_x(AnchorX::Left)),
            a.anchor_y(AnchorY::Top).min(b.anchor_y(AnchorY::Top)),
        )
            .into(),
        (
            a.anchor_x(AnchorX::Right).max(b.anchor_x(AnchorX::Right)),
            a.anchor_y(AnchorY::Bottom).max(b.anchor_y(AnchorY::Bottom)),
        )
            .into(),
    )
}

*/

// Draw-er as in "one who draws", not the furniture
pub enum ComponentDrawer {
    Image(ImageDrawer),
    Line(LineDrawer),
    Text(TextDrawer),
    Rect(RectDrawer),
    Plot(PlotDrawer),
}

impl ComponentDrawer {
    pub fn from_component(
        comp: Component,
        upload_manager: &Mutex<upload::UploadManager>,
    ) -> Result<Self, DrawerCreationError> {
        match comp {
            Component::Image(image) => Ok(Self::Image(ImageDrawer::try_from_component(
                image,
                upload_manager,
            )?)),
            Component::Text(text) => Ok(Self::Text(TextDrawer::from(text))),
            Component::Line(line) => Ok(Self::Line(LineDrawer::from(line))),
            Component::Rectangle(rectangle) => Ok(Self::Rect(RectDrawer::from(rectangle))),
            Component::Plot(plot) => Ok(Self::Plot(PlotDrawer::from(plot))),
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
            Self::Plot(drawer) => drawer.draw(target, offset_pt),
        };
    }

    pub fn advance_frame(&mut self) {
        match self {
            Self::Image(drawer) => drawer.advance_frame(),
            Self::Line(drawer) => drawer.advance_frame(),
            Self::Text(drawer) => drawer.advance_frame(),
            Self::Rect(drawer) => drawer.advance_frame(),
            Self::Plot(drawer) => drawer.advance_frame(),
        };
    }

    pub fn get_bbox(&self) -> eg_prim::Rectangle {
        match self {
            Self::Image(drawer) => drawer.get_bbox(),
            Self::Line(drawer) => drawer.get_bbox(),
            Self::Text(drawer) => drawer.get_bbox(),
            Self::Rect(drawer) => drawer.get_bbox(),
            Self::Plot(drawer) => drawer.get_bbox(),
        }
    }

    pub fn get_cloned_component(&self) -> Component {
        match self {
            Self::Image(drawer) => drawer.get_cloned_component(),
            Self::Line(drawer) => drawer.get_cloned_component(),
            Self::Text(drawer) => drawer.get_cloned_component(),
            Self::Rect(drawer) => drawer.get_cloned_component(),
            Self::Plot(drawer) => drawer.get_cloned_component(),
        }
    }
}

pub struct LineDrawer {
    component: Line,
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

    pub fn advance_frame(&mut self) {
        self.color.advance_frame();
    }

    pub fn get_cloned_component(&self) -> Component {
        Component::Line(self.component.clone())
    }
}

impl From<Line> for LineDrawer {
    fn from(component: Line) -> Self {
        let color: ColorDrawState = component.color.clone().into();

        Self { component, color }
    }
}

pub struct TextDrawer {
    component: Text,
    color: ColorDrawState,
    formatted_text: String, // need to own this so get_styled_text() can return a reference to it
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

    /// Re-calculate the contents of the text - only useful if it contains dynamic components like a time value
    fn update_formatted_text(&mut self) {
        self.formatted_text =
            text_string::make_string_from_template(&self.component.content, chrono::Local::now);
    }

    fn get_styled_text(&self) -> impl Drawable<Color = Rgb888> + Transform + Dimensions {
        let pos = eg_geo::Point::new(self.component.x, self.component.y);
        trace!("Constructing Text(pos {})", pos);
        let char_style = self.component.font.get_eg_font(self.color.get());
        let text_style = eg_text::TextStyleBuilder::new()
            .alignment(self.component.alignment.into())
            .baseline(eg_text::Baseline::Top)
            .build();

        // Make the formatted string based on the template
        eg_text::Text::with_text_style(&self.formatted_text, pos, char_style, text_style)
    }

    pub fn advance_frame(&mut self) {
        self.color.advance_frame();
        // This means that any special values (like time) in the displayed text might be slightly
        // out of date because they are updated when advance_frame() is called rather immediately before drawing.
        // This shouldn't be noticeable though since the frame rate will be many times per second.
        // I may move this into draw() if it becomes an issue for some reason.
        self.update_formatted_text();
    }

    pub fn get_cloned_component(&self) -> Component {
        Component::Text(self.component.clone())
    }
}

impl From<Text> for TextDrawer {
    fn from(component: Text) -> Self {
        let color: ColorDrawState = component.color.clone().into();
        let mut inst = Self {
            component,
            color,
            formatted_text: String::new(),
        };
        inst.update_formatted_text();
        inst
    }
}

pub struct ImageDrawer {
    component: Image,
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

    pub fn get_cloned_component(&self) -> Component {
        Component::Image(self.component.clone())
    }
}

impl ImageDrawer {
    pub fn try_from_component(
        component: Image,
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
    component: Rectangle,
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

    pub fn advance_frame(&mut self) {
        self.border_color.advance_frame();
        if let Some(ref mut color) = self.fill_color {
            color.advance_frame();
        }
    }

    pub fn get_cloned_component(&self) -> Component {
        Component::Rectangle(self.component.clone())
    }
}

impl From<Rectangle> for RectDrawer {
    fn from(component: Rectangle) -> Self {
        let fill_color = component.fill_color.clone().map(ColorDrawState::from);
        let border_color = ColorDrawState::from(component.border_color.clone());
        Self {
            component,
            border_color,
            fill_color,
        }
    }
}

pub struct PlotDrawer {
    component: Plot,
    fill_color: Option<ColorDrawState>, // Expected in bar mode, optional in line mode
    line_color: Option<ColorDrawState>, // Expected in line mode
    axis_color: Option<ColorDrawState>, // Optional
}

impl PlotDrawer {
    fn data_y_to_canvas_y(&self, data_y: f32) -> i32 {
        let min = self.component.plot_y_axis_min as f32;
        let max = self.component.plot_y_axis_max as f32;
        let clamped_y_offset = data_y.clamp(min, max) - min;

        let height_range = self.component.height - 1; // -1 to prevent max data point from being one pixel over the top

        self.get_data_region_bottom_y() - (height_range as f32 * clamped_y_offset / (max - min)).round() as i32
    }

    /// Get the bottom y-coordinate of the actual data portion of the plot
    fn get_data_region_bottom_y(&self) -> i32 {
        self.component.y + self.component.height as i32 - 1
    }

    /// Get the right x-coordinate of the actual data portion of the plot
    fn get_data_region_right_x(&self) -> i32 {
        match &self.component.plot_config {
            crate::graphics_component::PlotConfig::Bar { column_width, .. } => {
                self.component.x + *column_width as i32 * self.component.data.len() as i32 - 1
            }
            crate::graphics_component::PlotConfig::Line { x_spacing, .. } => {
                self.component.x + *x_spacing as i32 * (self.component.data.len() - 1) as i32
            }
        }
    }

    /// Draw the actual data portion of the plot
    fn draw_bar_plot_data<T>(&mut self, target: &mut T, offset: eg_geo::Point, column_width: u32)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        for (i, pt) in self.component.data.iter().enumerate() {
            let bar_top = self.data_y_to_canvas_y(*pt);
            let bar_bot = self.get_data_region_bottom_y();
            let bar_left = self.component.x + column_width as i32 * i as i32;
            let bar_right = bar_left + column_width as i32 - 1;

            let corner_1 = eg_geo::Point::new(bar_left, bar_top);
            let corner_2 = eg_geo::Point::new(bar_right, bar_bot);
            log::trace!("corners ({}) and ({})", corner_1, corner_2);

            if i == self.component.data.len() - 1 {
                assert_eq!(bar_right, self.get_data_region_right_x());
            }

            // draw the rectangle (fill)
            let style_builder =
                PrimitiveStyleBuilder::new().fill_color(self.fill_color.as_ref().unwrap().get());
            let styled = eg_prim::Rectangle::with_corners(corner_1, corner_2)
                .into_styled(style_builder.build())
                .translate(offset);

            // bbox = bbox_union(bbox, styled.bounding_box());
            styled.draw(target).unwrap();
        }
    }

    /// Draw the actual data portion of the plot
    /// Return the width of the drawn region (which unlike the height is dependent on
    /// the number of data points)
    fn draw_line_plot_data<T>(
        &mut self,
        target: &mut T,
        offset: eg_geo::Point,
        line_stroke: u32,
        x_spacing: u32,
    ) where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        let mut drawn_line_vertices: Vec<eg_geo::Point> = vec![];
        // let mut bbox = eg_prim::Rectangle::zero();

        for i in 0..self.component.data.len() {
            let pt = self.component.data[i];
            let slice_top_left_y = self.data_y_to_canvas_y(pt);
            let slice_bot = self.get_data_region_bottom_y();
            let slice_left = self.component.x + x_spacing as i32 * i as i32;

            drawn_line_vertices.push(eg_geo::Point::new(slice_left, slice_top_left_y));

            if let Some(fill_color) = &self.fill_color {
                let mut draw_col = |x, top, bot| {
                    let style_builder = PrimitiveStyleBuilder::new()
                        .stroke_color(fill_color.get())
                        .stroke_width(1);
                    let styled = eg_prim::Line::new((x, top).into(), (x, bot).into())
                        .into_styled(style_builder.build())
                        .translate(offset);

                    // bbox = bbox_union(bbox, styled.bounding_box());
                    styled.draw(target).unwrap();
                };

                if i == self.component.data.len() - 1 {
                    // Don't fill under/to the right of the last point- just need to draw a single column so the line doesn't overhang by 1 pixel
                    draw_col(slice_left, slice_top_left_y, slice_bot);
                } else {
                    let next_pt = self.component.data[i + 1];
                    let next_slice_top_left_y = self.data_y_to_canvas_y(next_pt);

                    // Just draw a vertical line for each pixel column - easier than splitting into a triangle + rectangle
                    for col_num in 0..(x_spacing) {
                        let delta = next_slice_top_left_y - slice_top_left_y;
                        // Round by adding 0.5 and flooring because we want .5 to round the same direction whether
                        // positive or negative, which the round() method doesn't do.
                        let col_top = slice_top_left_y
                            + (col_num as f32 / x_spacing as f32 * delta as f32 + 0.5).floor() as i32;

                        draw_col(slice_left + col_num as i32, col_top, slice_bot);
                    }
                }
            }
        }

        assert_eq!(
            drawn_line_vertices.last().unwrap().x,
            self.get_data_region_right_x()
        );

        // draw the line
        let style_builder = PrimitiveStyleBuilder::new()
            .stroke_color(self.line_color.as_ref().unwrap().get())
            .stroke_width(line_stroke);
        let styled = eg_prim::Polyline::new(&drawn_line_vertices)
            .into_styled(style_builder.build())
            .translate(offset);
        styled.draw(target).unwrap();
    }

    pub fn draw<T>(&mut self, target: &mut T, offset: eg_geo::Point)
    where
        T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
    {
        match &self.component.plot_config {
            crate::graphics_component::PlotConfig::Bar { column_width, .. } => {
                self.draw_bar_plot_data(target, offset, *column_width)
            }
            crate::graphics_component::PlotConfig::Line {
                line_stroke,
                x_spacing,
                ..
            } => self.draw_line_plot_data(target, offset, *line_stroke, *x_spacing),
        };

        // draw axes
        if let Some(stroke_width) = self.component.axis_stroke {
            let color = self.axis_color.as_ref().map_or(Rgb888::new(255, 255, 255), |s| s.get());

            // draw the line
            let style_builder = PrimitiveStyleBuilder::new()
                .stroke_color(color)
                .stroke_width(stroke_width);

            let y_axis_x_pos = self.component.x - stroke_width as i32 / 2;
            let x_axis_y_pos = self.get_data_region_bottom_y() + stroke_width as i32 / 2;
            let axis_endpoints = vec![
                eg_geo::Point::new(y_axis_x_pos, self.component.y),
                eg_geo::Point::new(y_axis_x_pos, x_axis_y_pos),
                eg_geo::Point::new(self.get_data_region_right_x(), x_axis_y_pos),
            ];
            eg_prim::Polyline::new(&axis_endpoints)
                .into_styled(style_builder.build())
                .translate(offset)
                .draw(target)
                .unwrap();
        }
    }

    pub fn get_bbox(&self) -> eg_prim::Rectangle {
        // Not currently used for Plot
        todo!()
    }

    pub fn get_cloned_component(&self) -> Component {
        Component::Plot(self.component.clone())
    }

    pub fn advance_frame(&mut self) {
        // TODO animated colors
    }
}

impl From<Plot> for PlotDrawer {
    fn from(component: Plot) -> Self {
        let fill_color_state: Option<ColorDrawState>;
        let line_color_state: Option<ColorDrawState>;

        match &component.plot_config {
            crate::graphics_component::PlotConfig::Bar { fill_color, .. } => {
                fill_color_state = Some(ColorDrawState::from(fill_color.clone()));
                line_color_state = None;
            }
            crate::graphics_component::PlotConfig::Line {
                fill_color,
                line_color,
                ..
            } => {
                fill_color_state = fill_color.clone().map(ColorDrawState::from);
                line_color_state = Some(ColorDrawState::from(line_color.clone()));
            }
        };
        let axes_color_state = component.axis_color.clone().map(ColorDrawState::from);

        Self {
            component,
            fill_color: fill_color_state,
            line_color: line_color_state,
            axis_color: axes_color_state,
        }
    }
}
