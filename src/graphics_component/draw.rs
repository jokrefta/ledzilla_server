use anyhow::Result;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::{
    Drawable, geometry as eg_geo, image as eg_image, mono_font as eg_mono, prelude::Primitive,
    primitives as eg_prim, text as eg_text,
};
use log::trace;
use thiserror::Error;

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::upload;

#[derive(Debug, Error)]
pub enum DrawerCreationError {
    #[error("Failed to construct drawer due to bad component spec: {0}")]
    BadComponentSpec(String),
    #[error("{0}")]
    _GeneralError(String),
}

// Draw-er as in "one who draws", not the furniture
pub trait ComponentDrawer<T>
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    fn draw_next_frame(&mut self, target: &mut T);
    fn get_cloned_component(&self) -> super::Component;
}

pub fn into_drawer<T>(
    comp: super::Component,
    upload_manager: &Mutex<upload::UploadManager>,
) -> Result<Box<dyn ComponentDrawer<T>>, DrawerCreationError>
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    match comp {
        super::Component::Image(image) => Ok(Box::new(ImageDrawer::try_from_component(image, upload_manager)?)),
        super::Component::Text(text) => Ok(Box::new(TextDrawer::from(text))),
        super::Component::Line(line) => Ok(Box::new(LineDrawer::from(line))),
    }
}

pub struct LineDrawer {
    component: super::Line,
}

impl<T> ComponentDrawer<T> for LineDrawer
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    fn draw_next_frame(&mut self, target: &mut T) {
        let start = eg_geo::Point::new(
            self.component.common_properties.x,
            self.component.common_properties.y,
        );
        let delta = eg_geo::Point::new(self.component.delta_x, self.component.delta_y);
        trace!("Drawing Line({}, {})", start, delta);
        eg_prim::Line::with_delta(start, delta)
            .into_styled(eg_prim::PrimitiveStyle::with_stroke(
                Rgb888::from(self.component.color),
                self.component.stroke_width,
            ))
            .draw(target)
            .unwrap();
    }

    fn get_cloned_component(&self) -> super::Component {
        super::Component::Line(self.component.clone())
    }
}

impl From<super::Line> for LineDrawer {
    fn from(component: super::Line) -> Self {
        Self { component }
    }
}

pub struct TextDrawer {
    component: super::Text,
}

impl<T> ComponentDrawer<T> for TextDrawer
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    fn draw_next_frame(&mut self, target: &mut T) {
        let pos = eg_geo::Point::new(
            self.component.common_properties.x,
            self.component.common_properties.y,
        );
        trace!("Drawing Text(pos {})", pos);
        let style = eg_mono::MonoTextStyle::new(
            self.component.font.get_eg_font(),
            Rgb888::from(self.component.color),
        );
        eg_text::Text::with_alignment(
            &self.component.content,
            pos,
            style,
            self.component.alignment.into(),
        )
        .draw(target)
        .unwrap();
    }

    fn get_cloned_component(&self) -> super::Component {
        super::Component::Text(self.component.clone())
    }
}

impl From<super::Text> for TextDrawer {
    fn from(component: super::Text) -> Self {
        Self { component }
    }
}

pub struct ImageDrawer {
    component: super::Image,
    image_data: Arc<upload::UploadedAsset>,
}

impl<T> ComponentDrawer<T> for ImageDrawer
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    fn draw_next_frame(&mut self, target: &mut T) {
        let pos = eg_geo::Point::new(
            self.component.common_properties.x,
            self.component.common_properties.y,
        );

        let raw_image = match &*self.image_data {
            upload::UploadedAsset::Image(image_buf) => {
                eg_image::ImageRaw::<Rgb888>::new(image_buf.get_rgb_raw(), image_buf.get_width())
            }
            upload::UploadedAsset::AnimatedImage(_animated_image_buf) => todo!(),
        };

        let image = eg_image::Image::new(&raw_image, pos);

        image.draw(target).unwrap();
    }

    fn get_cloned_component(&self) -> super::Component {
        super::Component::Image(self.component.clone())
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
            })
        } else {
            Err(DrawerCreationError::BadComponentSpec(
                "Could not find filename in database".to_string(),
            ))
        }
    }
}
