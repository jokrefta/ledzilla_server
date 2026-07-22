use std::thread;

use display::GraphicsDisplay;
use renderer::Renderer;

use crate::renderer::Command;

pub mod display;
mod graphics_component;
mod renderer;
mod server;

pub fn run_server<Disp, F>(display_provider: F)
where
    F: Send + FnMut() -> Disp,
    Disp: GraphicsDisplay + std::fmt::Debug,
{
    let (snd, rcv) = std::sync::mpsc::sync_channel::<Command>(2);

    thread::scope(|s| {
        s.spawn(|| {
            let mut renderer = Renderer::new(display_provider);
            renderer.run(rcv);
        });

        rouille::start_server("127.0.0.1:8080", move |req| {
            server::handle_request(snd.clone(), req)
        });
    });
}
