use embedded_graphics::geometry;
#[cfg(feature = "simulator")]
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use ledzilla_server::*;
use log::info;

fn main() {
    simple_logger::SimpleLogger::new()
        .env()
        .with_local_timestamps()
        .with_timestamp_format(time::macros::format_description!("[hour]:[minute]:[second]"))
        .init()
        .unwrap();

    log::info!("Hello, world!");

    let use_sim = true;
    if use_sim {
        run_server_sim();
    } else {
        todo!();
    }

    return;
}

fn run_server_sim() {
    #[cfg(not(feature = "simulator"))]
    panic!("Not compiled with simulator support!");

    #[cfg(feature = "simulator")]
    {
        let create_sim = || {
            use ledzilla_server::display::GraphicsDisplay;

            type Color = embedded_graphics::pixelcolor::Rgb888;
            let canvas: SimulatorDisplay<Color> = SimulatorDisplay::new(geometry::Size::new(64 * 4, 64));

            let output_settings = OutputSettingsBuilder::new().scale(4).pixel_spacing(1).build();

            info!("Creating simulator window");
            let mut window = Window::new("Test", &output_settings);
            window.set_max_fps(60);

            let mut sim = display::SimulatorDisplayWrapper::new(window, canvas);
            // Call update once so that the window actually appears
            sim = sim.update_display();
            sim
        };

        run_server(create_sim);
    }
}
