use std::fmt::Debug;

use embedded_graphics::pixelcolor::Rgb888;

#[cfg(feature = "led")]
pub use led::LedDisplayWrapper;
#[cfg(feature = "simulator")]
pub use simulator::SimulatorDisplayWrapper;

pub trait GraphicsDisplay {
    type DrawTarget: embedded_graphics::draw_target::DrawTarget<Color = Rgb888, Error: Debug>;

    fn get_draw_target(&mut self) -> &mut Self::DrawTarget;

    fn update_display(self) -> Self
    where
        Self: Sized;

    fn received_exit_event(&self) -> bool {
        false
    }
}

#[cfg(feature = "simulator")]
mod simulator {
    use super::*;
    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
    use embedded_graphics_simulator::{SimulatorDisplay, SimulatorEvent, Window};

    pub struct SimulatorDisplayWrapper {
        window: Window,
        canvas: SimulatorDisplay<Rgb888>,
    }

    impl SimulatorDisplayWrapper {
        pub fn new(window: Window, canvas: SimulatorDisplay<Rgb888>) -> Self {
            Self { canvas, window }
        }
    }

    impl GraphicsDisplay for SimulatorDisplayWrapper {
        type DrawTarget = SimulatorDisplay<Rgb888>;

        fn get_draw_target(&mut self) -> &mut SimulatorDisplay<Rgb888> {
            &mut self.canvas
        }

        fn update_display(mut self) -> Self {
            self.window.update(&self.canvas);
            self.canvas.clear(Rgb888::BLACK).unwrap();
            self
        }

        fn received_exit_event(&self) -> bool {
            self.window.events().any(|e| e == SimulatorEvent::Quit)
        }
    }
}

#[cfg(feature = "led")]
mod led {
    use super::*;
    use rpi_led_panel::{Canvas, RGBMatrix};

    pub struct LedDisplayWrapper {
        matrix: RGBMatrix,
        current_canvas: Box<Canvas>,
    }
    impl LedDisplayWrapper {
        pub fn new(matrix: RGBMatrix, initial_canvas: Box<Canvas>) -> Self {
            Self {
                matrix,
                current_canvas: initial_canvas,
            }
        }
    }

    impl GraphicsDisplay for LedDisplayWrapper {
        type DrawTarget = Canvas;

        fn get_draw_target(&mut self) -> &mut Canvas {
            &mut self.current_canvas
        }

        fn update_display(mut self) -> Self {
            self.current_canvas = self.matrix.update_on_vsync(self.current_canvas);
            self
        }
    }
}
