use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::{Drawable, primitives as eg_prim};
use embedded_graphics::{geometry as eg_geo, prelude::Primitive};

use std::fmt::Debug;

pub trait ComponentDrawer<T>
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    fn draw_next_frame(&mut self, target: &mut T);
    // I just need the ability to get a copy for now. If we ever want to be able to 
    fn get_cloned_component(&self) -> super::Component;
}

pub struct LineDrawer {
    component: super::Line,
}

impl<T> ComponentDrawer<T> for LineDrawer
where
    T: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>,
{
    type Comp = super::Line;

    fn draw_next_frame(&mut self, target: &mut T) {
        let start = eg_geo::Point::new(
            self.component.common_properties.x.try_into().unwrap(),
            self.component.common_properties.y.try_into().unwrap(),
        );
        let delta = eg_geo::Point::new(self.component.delta_x, self.component.delta_y);
        eg_prim::Line::new(start, delta)
            .into_styled(eg_prim::PrimitiveStyle::with_stroke(Rgb888::WHITE, 2))
            .draw(target);
    }
}

impl From<super::Line> for LineDrawer {
    fn from(component: super::Line) -> Self {
        Self { component }
    }
}
