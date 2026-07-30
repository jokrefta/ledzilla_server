use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::{Drawable, mono_font as eg_mono, primitives as eg_prim, text as eg_text};
use embedded_graphics::{geometry as eg_geo, prelude::Primitive};
use log::trace;

use std::fmt::Debug;

pub trait ComponentDrawer<T>
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    fn draw_next_frame(&mut self, target: &mut T);
    fn get_cloned_component(&self) -> super::Component;
}

pub fn into_drawer<T>(comp: super::Component) -> Box<dyn ComponentDrawer<T>>
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    match comp {
        super::Component::Image(_image) => todo!(),
        super::Component::Text(text) => Box::new(TextDrawer::from(text)),
        super::Component::Line(line) => Box::new(LineDrawer::from(line)),
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
            self.component.common_properties.x.try_into().unwrap(),
            self.component.common_properties.y.try_into().unwrap(),
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
            self.component.common_properties.x.try_into().unwrap(),
            self.component.common_properties.y.try_into().unwrap(),
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
