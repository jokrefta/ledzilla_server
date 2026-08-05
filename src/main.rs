use std::process::ExitCode;

#[cfg(feature = "simulator")]
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

use ledzilla_server::*;

use crate::config_parse::ConfigParser;

mod config_parse;

fn main() -> ExitCode {
    if let Err(e) = simple_logger::SimpleLogger::new()
        .env()
        .with_module_level("multipart", log::LevelFilter::Warn)
        .with_local_timestamps()
        .with_timestamp_format(time::macros::format_description!("[hour]:[minute]:[second]"))
        .init()
    {
        eprintln!("Failed to initialize logger! {}", e);
        return ExitCode::FAILURE;
    }

    log::error!("error logging enabled");
    log::warn!("warn logging enabled");
    log::info!("info logging enabled");
    log::debug!("debug logging enabled");
    log::trace!("trace logging enabled");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config filename>", &args[0]);
        return ExitCode::FAILURE;
    }

    let config_parser = match ConfigParser::from_file(args[1].clone()) {
        Ok(c) => c,
        Err(e) => {
            log::error!("{}", e);
            return ExitCode::FAILURE;
        }
    };
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

    ExitCode::SUCCESS
}

/// Create the closure which the renderer may invoke as needed to generate a new simulator display.
/// The returned closure captures a reference to `config`, so may only be used while `config` is valid.
#[cfg(feature = "simulator")]
fn mk_sim_display_provider(config: &ConfigParser) -> impl FnMut() -> display::SimulatorDisplayWrapper + use<> {
    {
        let sim_config = config.get_sim_config();
        move || {
            use ledzilla_server::display::GraphicsDisplay;
            type Color = embedded_graphics::pixelcolor::Rgb888;

            let canvas: SimulatorDisplay<Color> = SimulatorDisplay::new(
                embedded_graphics::geometry::Size::new(sim_config.width, sim_config.height),
            );

            let output_settings = OutputSettingsBuilder::new().scale(4).pixel_spacing(1).build();

            log::info!("Creating simulator window");
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
fn mk_led_display_provider(config: &ConfigParser) -> impl FnMut() -> display::LedDisplayWrapper + use<> {
    #[cfg(not(feature = "led"))]
    panic!("Not compiled with led support!");

    #[cfg(feature = "led")]
    {
        let config = config.clone(); // Avoid lifetime issues (the returned closure sneeds to be valid for 'static)
        move || {
            use ledzilla_server::display::LedDisplayWrapper;

            log::info!("Creating led display");
            let led_config = config.get_led_config().unwrap();
            let (matrix, canvas) =
                rpi_led_panel::RGBMatrix::new(led_config, 0).expect("Matrix initialization failed");

            LedDisplayWrapper::new(matrix, canvas)
        }
    }
}
