use std::sync::mpsc;

use log::debug;

use crate::display::GraphicsDisplay;

#[derive(Debug)]
pub enum RendererCommand {
    // Will likely mirror the API pretty closely
    Start,
    Stop,
    Todo,
}

pub fn run_renderer_thread<F, Disp>(
    mut display_provider: F,
    command_receiver: mpsc::Receiver<RendererCommand>,
) where
    F: FnMut() -> Disp,
    Disp: GraphicsDisplay,
{
    let mut display: Option<Disp> = None;
    loop {
        let cmd = command_receiver.recv();
        debug!("received {:?}", cmd);
        match cmd {
            Ok(RendererCommand::Start) => {
                display = Some(display_provider());
            }
            Ok(RendererCommand::Stop) => {
                display = None;
            }
            Ok(_) => {
                todo!()
            }
            Err(_) => todo!(),
        }
    }
}
