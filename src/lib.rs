use std::sync::{Arc, Mutex};
use std::thread;

use display::GraphicsDisplay;
use renderer::Command;
use renderer::Renderer;
use upload::UploadManager;

pub mod display;
mod graphics_component;
mod renderer;
mod server;
mod upload;

#[derive(Debug, Clone)]
pub struct LedzillaServerConfig {
    pub ip: String,
    pub port: u16,
    pub content_root: String,
    pub canvas_size: (u32, u32),
}

pub fn run_server<Disp, F>(display_provider: F, config: LedzillaServerConfig) -> !
where
    F: Send + FnMut() -> Disp + 'static,
    Disp: GraphicsDisplay + std::fmt::Debug,
{
    let (snd, rcv) = std::sync::mpsc::sync_channel::<Command>(2);
    let ip_port = format!("{}:{}", config.ip, config.port);

    let upload_manager = Arc::new(Mutex::new(UploadManager::new()));

    {
        let upload_manager_clone = upload_manager.clone();
        thread::spawn(move || {
            let mut renderer = Renderer::new(display_provider, upload_manager_clone, config.canvas_size);
            renderer.run(rcv);
        });

        let upload_manager_clone = upload_manager.clone();
        rouille::start_server(ip_port, move |req| {
            server::handle_request(
                snd.clone(),
                req,
                // Must clone again because this closure must implement Fn,
                // i.e. must be callable many times
                upload_manager_clone.clone(),
                &config,
            )
        });
    }
}
