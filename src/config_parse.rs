use serde::Deserialize;

#[cfg(feature = "led")]
use rpi_led_panel::RGBMatrixConfig;

#[derive(Debug, Clone)]
pub struct ConfigParser {
    parsed_config: ParsedConfig,
}

impl ConfigParser {
    pub fn from_file(filename: String) -> Result<Self, String> {
        let contents = std::fs::read_to_string(filename).map_err(|e| e.to_string())?;
        let parsed: ParsedConfig = toml::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(ConfigParser {
            parsed_config: parsed,
        })
    }

    pub fn get_general_config(&self) -> GeneralConfig {
        self.parsed_config.general_config.clone()
    }

    #[cfg(feature = "simulator")]
    pub fn get_sim_config(&self) -> SimConfig {
        self.parsed_config.sim_config.clone()
    }

    #[cfg(feature = "led")]
    pub fn get_led_config(&self) -> Result<RGBMatrixConfig, String> {
        use std::str::FromStr;

use rpi_led_panel::HardwareMapping;
        let mut config = rpi_led_panel::RGBMatrixConfig::default();
        let parsed_led_config = &self.parsed_config.led_config;

        config.rows = parsed_led_config.rows;
        config.cols = parsed_led_config.cols;
        if let Some(v) = parsed_led_config.chain_length {
            config.chain_length = v;
        }
        if let Some(v) = parsed_led_config.num_chains {
            config.parallel = v;
        }
        if let Some(v) = parsed_led_config.pwm_bits {
            config.pwm_bits = v;
        }
        if let Some(v) = parsed_led_config.pwm_lsb_ns {
            config.pwm_lsb_nanoseconds = v;
        }
        if let Some(v) = parsed_led_config.pwm_dither_bits {
            config.dither_bits = v;
        }
        if let Some(v) = parsed_led_config.gpio_slowdown {
            config.slowdown = Some(v);
        }
        if let Some(arr) = parsed_led_config.pixel_mappers.as_ref() {
            for s in arr {
                let mapper = rpi_led_panel::NamedPixelMapperType::from_str(s).map_err(|e| e.to_string());
                config.pixelmapper.push(mapper?);
            }
        }
        if let Some(v) = parsed_led_config.brightness {
            config.led_brightness = v;
        }

        // Some information comes from the main game config section
        config.refresh_rate = parsed_led_config.refresh_rate as usize;

        // In the current rpi_led_panel version this is not defaulted properly so set it explicitly
        config.hardware_mapping = HardwareMapping::regular();

        Ok(config)
    }

    pub fn get_ledzilla_config(&self) -> ledzilla_server::LedzillaServerConfig {
        let general_config = &self.parsed_config.general_config;
        ledzilla_server::LedzillaServerConfig {
            port: general_config.server_port,
            canvas_size: (general_config.canvas_width, general_config.canvas_height),
        }
    }
}

// pub fn load_led_config(led_config_section: Table)

////////////////////////////////////////////

fn default_port() -> u16 {
    8080
}
fn default_fps() -> u32 {
    75
}

#[derive(Debug, Clone, Deserialize)]
struct ParsedConfig {
    general_config: GeneralConfig,

    #[cfg(feature = "led")]
    led_config: ParsedLedConfig,

    #[cfg(feature = "simulator")]
    sim_config: SimConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeneralConfig {
    pub use_sim: bool,

    // TODO add verification that they match up at least for sim mode
    /// Canvas width should match the corresponding parameters in the simulator or led config
    pub canvas_width: u32,
    /// Canvas height should match the corresponding parameters in the simulator or led config
    pub canvas_height: u32,

    #[serde(default = "default_port")]
    pub server_port: u16,
}

// Private as this gets transformed into a RGBMatrixConfig
#[cfg(feature = "led")]
#[derive(Debug, Clone, Deserialize)]
struct ParsedLedConfig {
    rows: usize,
    cols: usize,
    chain_length: Option<usize>,
    num_chains: Option<usize>,
    pwm_bits: Option<usize>,
    pwm_dither_bits: Option<usize>,
    pwm_lsb_ns: Option<u32>,
    gpio_slowdown: Option<u32>,
    pixel_mappers: Option<Vec<String>>,
    brightness: Option<u8>,

    #[serde(default = "default_fps")]
    refresh_rate: u32,
}

#[cfg(feature = "simulator")]
#[derive(Debug, Clone, Deserialize)]
pub struct SimConfig {
    pub width: u32,
    pub height: u32,
    pub scaling_factor: u32,
    #[serde(default = "default_fps")]
    pub target_fps: u32,
}
