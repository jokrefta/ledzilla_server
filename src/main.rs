#[cfg(feature = "simulator")]
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

use ledzilla_server::*;
use log::info;

use crate::config_parse::ConfigParser;

mod config_parse;

fn main() {
    simple_logger::SimpleLogger::new()
        .env()
        .with_local_timestamps()
        .with_timestamp_format(time::macros::format_description!("[hour]:[minute]:[second]"))
        .init()
        .unwrap();

    log::info!("Hello, world!");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config filename>", &args[0]);
        return;
    }

    let config_parser = ConfigParser::from_file(args[1].clone()).unwrap();
    let gen_params = config_parser.get_general_config();

    if gen_params.use_sim {
        #[cfg(not(feature = "simulator"))]
        panic!("Not compiled with simulator support!");
        #[cfg(feature = "simulator")]
        run_server(mk_sim_display_provider(&config_parser), gen_params.server_port);
    } else {
        #[cfg(not(feature = "led"))]
        panic!("Not compiled with led support!");
        #[cfg(feature = "led")]
        run_server(mk_led_display_provider(&config_parser), gen_params.server_port);
    }

    return;
}

/// Create the closure which the renderer may invoke as needed to generate a new simulator display.
/// The returned closure captures a reference to `config`, so may only be used while `config` is valid.
#[cfg(feature = "simulator")]
fn mk_sim_display_provider(config: &ConfigParser) -> impl FnMut() -> display::SimulatorDisplayWrapper {
    {
        || {
            use ledzilla_server::display::GraphicsDisplay;
            type Color = embedded_graphics::pixelcolor::Rgb888;

            let sim_config = config.get_sim_config();
            let canvas: SimulatorDisplay<Color> = SimulatorDisplay::new(
                embedded_graphics::geometry::Size::new(sim_config.width, sim_config.height),
            );

            let output_settings = OutputSettingsBuilder::new().scale(4).pixel_spacing(1).build();

            info!("Creating simulator window");
            let mut window = Window::new("Test", &output_settings);
            window.set_max_fps(sim_config.target_fps);

            let mut sim = display::SimulatorDisplayWrapper::new(window, canvas);
            // Call update once so that the window actually appears
            sim = sim.update_display();
            sim
        }
    }
}

/// Create the closure which the renderer may invoke as needed to generate a new LED matrix display.
/// The returned closure captures a reference to `config`, so may only be used while `config` is valid.
#[cfg(feature = "led")]
fn mk_led_display_provider(config: &ConfigParser) -> impl FnMut() -> display::LedDisplayWrapper {
    #[cfg(not(feature = "led"))]
    panic!("Not compiled with led support!");

    #[cfg(feature = "led")]
    {
        || {
            use ledzilla_server::display::LedDisplayWrapper;

            info!("Creating led display");
            let led_config = config.get_led_config().unwrap();
            let (matrix, canvas) =
                rpi_led_panel::RGBMatrix::new(led_config, 0).expect("Matrix initialization failed");

            LedDisplayWrapper::new(matrix, canvas)
        }
    }
}
