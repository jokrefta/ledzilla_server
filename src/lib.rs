use std::thread;


use display::GraphicsDisplay;
use renderer::run_renderer_thread;

use crate::renderer::RendererCommand;

mod component;
pub mod display;
mod renderer;
mod server;

pub fn run_server<Disp, F>(display_provider: F)
where
    F: Send + FnMut() -> Disp,
    Disp: GraphicsDisplay,
{
    let (snd, rcv) = std::sync::mpsc::sync_channel::<RendererCommand>(2);

    thread::scope(|s| {
        s.spawn(|| {
            run_renderer_thread(display_provider, rcv);
        });

        rouille::start_server("127.0.0.1:8080", move |req| {
            server::handle_request(snd.clone(), req)
        });
    });
}
